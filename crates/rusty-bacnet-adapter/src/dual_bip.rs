//! Opt-in dual B/IP `BACnetRouter` session (CI-safe M3 deepen; no MS/TP).
//!
//! Lab unlock: `--route-bip-bip`. Ordinary boot must not call this.
//! Proves Clause 6 forwarding between two B/IP networks in netns/CI.
//! Product G7/G8 BIP↔MS/TP bench claim remains open.

use std::net::Ipv4Addr;
use std::path::Path;

use bacnet_network::router::{BACnetRouter, RouterPort};
use bacnet_transport::any::AnyTransport;
use bacnet_transport::mstp::NoSerial;
use tokio::sync::mpsc;
use tracing::info;

use crate::bip_qualify::assert_bind_on_interface;
use crate::ports::{build_bip_transport, BipTransportParams};
use crate::validate::{validate_distinct_networks, AdapterError};

/// Two B/IP ports under one half-router (no serial).
pub struct DualBipRouterSession {
    router: BACnetRouter,
    local_rx: mpsc::Receiver<bacnet_network::layer::ReceivedApdu>,
    network_a: u16,
    network_b: u16,
}

impl DualBipRouterSession {
    pub async fn start(
        a: &BipTransportParams,
        b: &BipTransportParams,
    ) -> Result<Self, AdapterError> {
        validate_distinct_networks(a.network, b.network)?;
        if a.udp_port == b.udp_port && a.interface_addr == b.interface_addr {
            return Err(AdapterError::Validation(
                "dual B/IP ports must not share identical bind address and UDP port".into(),
            ));
        }
        assert_bind_on_interface(&a.interface_name, a.interface_addr)?;
        assert_bind_on_interface(&b.interface_name, b.interface_addr)?;
        let transport_a = build_bip_transport(a)?;
        let transport_b = build_bip_transport(b)?;
        let ports = vec![
            RouterPort {
                transport: AnyTransport::<NoSerial>::Bip(transport_a),
                network_number: a.network,
            },
            RouterPort {
                transport: AnyTransport::<NoSerial>::Bip(transport_b),
                network_number: b.network,
            },
        ];
        let (router, local_rx) = BACnetRouter::start(ports).await?;
        info!(
            net_a = a.network,
            net_b = b.network,
            "dual B/IP router session started (opt-in; BIP↔MS/TP G7/G8 still open)"
        );
        Ok(Self {
            router,
            local_rx,
            network_a: a.network,
            network_b: b.network,
        })
    }

    pub fn network_a(&self) -> u16 {
        self.network_a
    }

    pub fn network_b(&self) -> u16 {
        self.network_b
    }

    pub fn local_rx(&mut self) -> &mut mpsc::Receiver<bacnet_network::layer::ReceivedApdu> {
        &mut self.local_rx
    }

    pub async fn route_table_len(&self) -> usize {
        self.router.table().lock().await.len()
    }

    pub async fn stop(mut self) -> Result<(), AdapterError> {
        self.router.stop().await;
        Ok(())
    }

    /// Atomic JSON report (temp + rename).
    pub fn write_report(&self, path: &Path, detail: &str) -> Result<(), AdapterError> {
        let body = format!(
            "{{\n  \"ready_to_route\": false,\n  \"mode\": \"dual_bip\",\n  \"network_a\": {},\n  \"network_b\": {},\n  \"bacnet_router\": true,\n  \"mstp\": false,\n  \"detail\": {:?}\n}}\n",
            self.network_a, self.network_b, detail
        );
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, body)
            .map_err(|e| AdapterError::Validation(format!("report write: {e}")))?;
        std::fs::rename(&tmp, path)
            .map_err(|e| AdapterError::Validation(format!("report rename: {e}")))?;
        Ok(())
    }
}

/// Second B/IP side for `--route-bip-bip` from environment (lab/CI harness).
pub fn bip2_params_from_env(fallback_port: u16) -> Result<BipTransportParams, AdapterError> {
    let interface_addr: Ipv4Addr = std::env::var("DBR_BIP2_BIND")
        .unwrap_or_else(|_| "198.51.100.1".into())
        .parse()
        .map_err(|e| AdapterError::Validation(format!("DBR_BIP2_BIND: {e}")))?;
    let broadcast_address: Ipv4Addr = std::env::var("DBR_BIP2_BROADCAST")
        .unwrap_or_else(|_| "198.51.100.255".into())
        .parse()
        .map_err(|e| AdapterError::Validation(format!("DBR_BIP2_BROADCAST: {e}")))?;
    let udp_port: u16 = std::env::var("DBR_BIP2_PORT")
        .ok()
        .map(|s| s.parse())
        .transpose()
        .map_err(|e| AdapterError::Validation(format!("DBR_BIP2_PORT: {e}")))?
        .unwrap_or(fallback_port);
    let network: u16 = std::env::var("DBR_BIP2_NETWORK")
        .unwrap_or_else(|_| "2000".into())
        .parse()
        .map_err(|e| AdapterError::Validation(format!("DBR_BIP2_NETWORK: {e}")))?;
    let interface_name = std::env::var("DBR_BIP2_IFACE").unwrap_or_else(|_| "eth1".into());
    Ok(BipTransportParams {
        interface_addr,
        udp_port,
        broadcast_address,
        network,
        interface_name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn dual_bip_localhost_start_stop() {
        let a = BipTransportParams {
            interface_addr: Ipv4Addr::LOCALHOST,
            udp_port: 47_881,
            broadcast_address: Ipv4Addr::LOCALHOST,
            network: 1_100,
            interface_name: "lo".into(),
        };
        let b = BipTransportParams {
            interface_addr: Ipv4Addr::LOCALHOST,
            udp_port: 47_882,
            broadcast_address: Ipv4Addr::LOCALHOST,
            network: 2_200,
            interface_name: "lo".into(),
        };
        let session = DualBipRouterSession::start(&a, &b).await.unwrap();
        assert_eq!(session.route_table_len().await, 2);
        session.stop().await.unwrap();
    }

    #[tokio::test]
    async fn dual_bip_rejects_identical_bind() {
        let a = BipTransportParams {
            interface_addr: Ipv4Addr::LOCALHOST,
            udp_port: 47_883,
            broadcast_address: Ipv4Addr::LOCALHOST,
            network: 1,
            interface_name: "lo".into(),
        };
        let b = BipTransportParams {
            interface_addr: Ipv4Addr::LOCALHOST,
            udp_port: 47_883,
            broadcast_address: Ipv4Addr::LOCALHOST,
            network: 2,
            interface_name: "lo".into(),
        };
        assert!(DualBipRouterSession::start(&a, &b).await.is_err());
    }

    #[tokio::test]
    async fn dual_bip_stop_rebind_same_ports() {
        let a = BipTransportParams {
            interface_addr: Ipv4Addr::LOCALHOST,
            udp_port: 47_884,
            broadcast_address: Ipv4Addr::LOCALHOST,
            network: 3_100,
            interface_name: "lo".into(),
        };
        let b = BipTransportParams {
            interface_addr: Ipv4Addr::LOCALHOST,
            udp_port: 47_885,
            broadcast_address: Ipv4Addr::LOCALHOST,
            network: 3_200,
            interface_name: "lo".into(),
        };
        let s1 = DualBipRouterSession::start(&a, &b).await.unwrap();
        s1.stop().await.unwrap();
        // Brief pause for OS to release UDP ports.
        tokio::time::sleep(Duration::from_millis(50)).await;
        let mut s2 = DualBipRouterSession::start(&a, &b).await.unwrap();
        assert_eq!(s2.route_table_len().await, 2);
        let _ = timeout(Duration::from_millis(10), s2.local_rx().recv()).await;
        s2.stop().await.unwrap();
    }
}
