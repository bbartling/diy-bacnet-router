//! Explicit BACnet/IP port qualification session (one socket, no router, no MS/TP).
//!
//! Ordinary appliance startup must not call this. Qualification is opt-in via
//! `routerd --bip-qualify` / `--bip-qualify-peer` for Linux netns lab and CI.

use std::net::Ipv4Addr;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use bacnet_transport::bip::BipTransport;
use bacnet_transport::port::TransportPort;
use tokio::sync::{mpsc, oneshot};
use tokio::time::{timeout, Instant};
use tracing::{info, warn};

use crate::ports::{build_bip_transport, BipTransportParams};
use crate::validate::AdapterError;

/// Observed B/IP qualify counters (not MS/TP; not forwarding).
#[derive(Debug, Default)]
pub struct BipQualifyCounters {
    pub rx_packets: AtomicU64,
    pub tx_packets: AtomicU64,
}

impl BipQualifyCounters {
    #[must_use]
    pub fn snapshot(&self) -> (u64, u64) {
        (
            self.rx_packets.load(Ordering::Relaxed),
            self.tx_packets.load(Ordering::Relaxed),
        )
    }
}

/// Live qualification session owning exactly one `BipTransport`.
pub struct BipQualifySession {
    transport: BipTransport,
    rx: mpsc::Receiver<bacnet_transport::port::ReceivedNpdu>,
    counters: Arc<BipQualifyCounters>,
    active: Arc<AtomicBool>,
}

impl BipQualifySession {
    /// Construct + `start()` one B/IP socket after validating params and (on Linux)
    /// that `bind_address` appears on the named interface.
    pub async fn start(params: &BipTransportParams) -> Result<Self, AdapterError> {
        assert_bind_on_interface(&params.interface_name, params.interface_addr)?;
        let mut transport = build_bip_transport(params)?;
        let rx = transport.start().await?;
        info!(
            interface = %params.interface_name,
            bind = %params.interface_addr,
            port = params.udp_port,
            "B/IP qualify session started (no BACnetRouter, no MS/TP, no forwarding)"
        );
        Ok(Self {
            transport,
            rx,
            counters: Arc::new(BipQualifyCounters::default()),
            active: Arc::new(AtomicBool::new(true)),
        })
    }

    #[must_use]
    pub fn counters(&self) -> Arc<BipQualifyCounters> {
        Arc::clone(&self.counters)
    }

    #[must_use]
    pub fn active_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.active)
    }

    /// Drain inbound NPDUs until `stop_rx` fires or `max_duration` elapses.
    pub async fn run_receive_loop(
        &mut self,
        mut stop_rx: oneshot::Receiver<()>,
        max_duration: Duration,
    ) {
        let deadline = Instant::now() + max_duration;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }
            tokio::select! {
                _ = &mut stop_rx => break,
                maybe = timeout(remaining, self.rx.recv()) => {
                    match maybe {
                        Ok(Some(_npdu)) => {
                            self.counters.rx_packets.fetch_add(1, Ordering::Relaxed);
                        }
                        Ok(None) => break,
                        Err(_) => break,
                    }
                }
            }
        }
        self.active.store(false, Ordering::Relaxed);
    }

    /// Send a minimal NPDU broadcast (peer helper path).
    pub async fn send_broadcast_probe(&self) -> Result<(), AdapterError> {
        // Minimal NPDU: version=1, control=0 (APDU follows) + dummy APDU octet.
        let npdu = [0x01_u8, 0x00, 0x10];
        self.transport.send_broadcast(&npdu).await?;
        self.counters.tx_packets.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Send a unicast probe to a peer BIP MAC (6 bytes: IPv4 + port BE).
    pub async fn send_unicast_probe(&self, mac: &[u8]) -> Result<(), AdapterError> {
        let npdu = [0x01_u8, 0x00, 0x10];
        self.transport.send_unicast(&npdu, mac).await?;
        self.counters.tx_packets.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    #[must_use]
    pub fn local_mac(&self) -> &[u8] {
        self.transport.local_mac()
    }

    pub async fn stop(&mut self) -> Result<(), AdapterError> {
        self.active.store(false, Ordering::Relaxed);
        self.transport.stop().await?;
        Ok(())
    }
}

/// Encode BIP MAC: 4-byte IPv4 + 2-byte port (big-endian).
#[must_use]
pub fn encode_bip_mac(ip: Ipv4Addr, port: u16) -> [u8; 6] {
    let o = ip.octets();
    let p = port.to_be_bytes();
    [o[0], o[1], o[2], o[3], p[0], p[1]]
}

/// On Linux, require `bind` to appear on `interface` via `ip`. Elsewhere, only
/// document that interface is advisory at construct-only time.
pub fn assert_bind_on_interface(interface: &str, bind: Ipv4Addr) -> Result<(), AdapterError> {
    if interface.trim().is_empty() {
        return Err(AdapterError::Validation(
            "bacnet_ip.interface must not be empty for B/IP qualify".into(),
        ));
    }
    if cfg!(not(target_os = "linux")) {
        warn!(
            interface,
            %bind,
            "interface membership check skipped on non-Linux; construct-only paths treat interface as advisory"
        );
        return Ok(());
    }
    if bind.is_unspecified() {
        return Err(AdapterError::Validation(
            "B/IP qualify refuses bind_address 0.0.0.0; use a concrete interface address".into(),
        ));
    }
    let output = Command::new("ip")
        .args(["-4", "-o", "addr", "show", "dev", interface])
        .output()
        .map_err(|error| {
            AdapterError::Validation(format!(
                "failed to query interface {interface} via ip(8): {error}"
            ))
        })?;
    if !output.status.success() {
        return Err(AdapterError::Validation(format!(
            "interface {interface} not found or not queryable (ip exit {})",
            output.status
        )));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let found = text
        .split_whitespace()
        .any(|tok| tok == bind.to_string() || tok.starts_with(&format!("{bind}/")));
    if !found {
        return Err(AdapterError::Validation(format!(
            "bind_address {bind} is not configured on interface {interface} (fail closed)"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_bip_mac_layout() {
        let mac = encode_bip_mac(Ipv4Addr::new(192, 0, 2, 1), 47_808);
        assert_eq!(mac, [192, 0, 2, 1, 0xBA, 0xC0]);
    }

    #[tokio::test]
    async fn localhost_qualify_start_stop_counts_probe() {
        let params = BipTransportParams {
            interface_addr: Ipv4Addr::LOCALHOST,
            udp_port: 47_818,
            broadcast_address: Ipv4Addr::LOCALHOST,
            network: 1,
            interface_name: "lo".into(),
        };
        // Non-Linux skips iface check; on Linux lo has 127.0.0.1.
        let mut session = BipQualifySession::start(&params).await.unwrap();
        session.send_broadcast_probe().await.unwrap();
        let (_rx, tx) = session.counters().snapshot();
        assert_eq!(tx, 1);
        session.stop().await.unwrap();
        assert!(!session.active_flag().load(Ordering::Relaxed));
    }
}
