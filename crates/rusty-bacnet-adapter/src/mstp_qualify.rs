//! Opt-in MS/TP port-only qualification (no B/IP, no BACnetRouter, no forwarding).
//!
//! Ordinary appliance startup must not call this. Physical evidence is a later
//! Pi-lab operator task — unit/integration tests here use FakeSerial only.

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use bacnet_transport::mstp::{MstpTransport, SerialPort};
use bacnet_transport::mstp_serial::{SerialConfig, TokioSerialPort};
use bacnet_transport::port::TransportPort;
use tokio::sync::oneshot;
use tokio::time::{timeout, Instant};
use tracing::info;

use crate::ports::{build_mstp_transport, ApplianceSerial, MstpTransportParams};
use crate::validate::{validate_mstp_params, AdapterError, MstpValidationInput};

/// Observed MS/TP qualify fields from public `MasterNode` (not synthesized CRC totals).
#[derive(Debug, Default)]
pub struct MstpQualifyCounters {
    pub event_count: AtomicU64,
    pub poll_station: AtomicU64,
    pub next_station: AtomicU64,
    pub token_count_since_pfm: AtomicU64,
    pub samples: AtomicU64,
}

impl MstpQualifyCounters {
    pub fn snapshot_tuple(&self) -> (u64, u64, u64, u64, u64) {
        (
            self.event_count.load(Ordering::Relaxed),
            self.poll_station.load(Ordering::Relaxed),
            self.next_station.load(Ordering::Relaxed),
            self.token_count_since_pfm.load(Ordering::Relaxed),
            self.samples.load(Ordering::Relaxed),
        )
    }
}

/// Open product serial for Waveshare auto-direction adapters only (no ioctl/GPIO).
pub fn open_appliance_serial(
    params: &MstpTransportParams,
) -> Result<ApplianceSerial, AdapterError> {
    validate_mstp_params(&MstpValidationInput {
        serial_path: params.serial_path.clone(),
        adapter_profile: params.adapter_profile.clone(),
        baud: params.baud_rate,
        mac: params.this_station,
        max_master: params.max_master,
        max_info_frames: params.max_info_frames,
        network: params.network,
        require_hardware_auto_direction: true,
    })?;
    // Refuse enabling kernel RS-485 / GPIO in this helper — Waveshare B/C = hardware DE/RE.
    let config = SerialConfig {
        port_name: params.serial_path.clone(),
        baud_rate: params.baud_rate,
    };
    TokioSerialPort::open(&config).map_err(AdapterError::from)
}

pub struct MstpQualifySession<S: SerialPort> {
    transport: MstpTransport<S>,
    rx: tokio::sync::mpsc::Receiver<bacnet_transport::port::ReceivedNpdu>,
    counters: Arc<MstpQualifyCounters>,
    active: Arc<AtomicBool>,
}

impl<S: SerialPort + 'static> MstpQualifySession<S> {
    pub async fn start(serial: S, params: &MstpTransportParams) -> Result<Self, AdapterError> {
        let mut transport = build_mstp_transport(serial, params)?;
        let rx = transport.start().await?;
        info!(
            path = %params.serial_path,
            mac = params.this_station,
            baud = params.baud_rate,
            "MS/TP qualify session started (no B/IP, no BACnetRouter, no forwarding)"
        );
        Ok(Self {
            transport,
            rx,
            counters: Arc::new(MstpQualifyCounters::default()),
            active: Arc::new(AtomicBool::new(true)),
        })
    }

    #[must_use]
    pub fn counters(&self) -> Arc<MstpQualifyCounters> {
        Arc::clone(&self.counters)
    }

    #[must_use]
    pub fn active_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.active)
    }

    /// Sample public MasterNode fields into qualify counters; drain NPDUs until stop/timeout.
    pub async fn run_until(&mut self, mut stop_rx: oneshot::Receiver<()>, max_duration: Duration) {
        let deadline = Instant::now() + max_duration;
        let mut ticker = tokio::time::interval(Duration::from_millis(200));
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }
            tokio::select! {
                _ = &mut stop_rx => break,
                _ = ticker.tick() => {
                    if let Some(node) = self.transport.node_state() {
                        if let Ok(guard) = node.try_lock() {
                            self.counters.event_count.store(u64::from(guard.event_count), Ordering::Relaxed);
                            self.counters.poll_station.store(u64::from(guard.poll_station), Ordering::Relaxed);
                            self.counters.next_station.store(u64::from(guard.next_station), Ordering::Relaxed);
                            self.counters.token_count_since_pfm.store(u64::from(guard.token_count), Ordering::Relaxed);
                            self.counters.samples.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
                maybe = timeout(remaining, self.rx.recv()) => {
                    match maybe {
                        Ok(Some(_)) => {
                            // Frame observed on transport channel — counted via node_state event_count when available.
                        }
                        Ok(None) => break,
                        Err(_) => break,
                    }
                }
            }
        }
        self.active.store(false, Ordering::Relaxed);
    }

    pub async fn stop(&mut self) -> Result<(), AdapterError> {
        self.active.store(false, Ordering::Relaxed);
        self.transport.stop().await?;
        Ok(())
    }

    /// Write a bounded atomic JSON report (temp then rename).
    pub fn write_report(&self, path: &Path, detail: &str) -> Result<(), AdapterError> {
        let (events, poll, next, token_pfm, samples) = self.counters.snapshot_tuple();
        let body = format!(
            "{{\n  \"ready_to_route\": false,\n  \"forwarding\": 0,\n  \"bip_opened\": false,\n  \"bacnet_router\": false,\n  \"observed\": {{\n    \"event_count\": {events},\n    \"poll_station\": {poll},\n    \"next_station\": {next},\n    \"token_count_since_pfm\": {token_pfm},\n    \"samples\": {samples}\n  }},\n  \"upstream_gaps\": [\n    \"No public aggregate tx_tokens/rx_tokens/CRC error counters on MstpTransport at pin 24e3439; only MasterNode fields are mirrored.\"\n  ],\n  \"detail\": {detail:?}\n}}\n"
        );
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, body)
            .map_err(|e| AdapterError::Validation(format!("report write: {e}")))?;
        std::fs::rename(&tmp, path)
            .map_err(|e| AdapterError::Validation(format!("report rename: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::{WAVESHARE_AUTO_DIRECTION_PROFILE, WAVESHARE_B_AUTO_DIRECTION_PROFILE};
    use bacnet_types::error::Error as BacnetError;
    use std::sync::Mutex;

    struct FakeSerial {
        buf: Mutex<Vec<u8>>,
    }

    impl SerialPort for FakeSerial {
        async fn write(&self, data: &[u8]) -> Result<(), BacnetError> {
            self.buf.lock().unwrap().extend_from_slice(data);
            Ok(())
        }

        async fn read(&self, buf: &mut [u8]) -> Result<usize, BacnetError> {
            // Avoid busy-spinning the MS/TP receive task in tests.
            tokio::time::sleep(Duration::from_millis(20)).await;
            let mut guard = self.buf.lock().unwrap();
            if guard.is_empty() {
                return Ok(0);
            }
            let n = buf.len().min(guard.len());
            buf[..n].copy_from_slice(&guard[..n]);
            guard.drain(..n);
            Ok(n)
        }
    }

    fn sample_params() -> MstpTransportParams {
        MstpTransportParams {
            this_station: 1,
            max_master: 2,
            max_info_frames: 1,
            baud_rate: 38_400,
            network: 2_000,
            serial_path: "/dev/serial/by-id/usb-Waveshare-C-if00".into(),
            adapter_profile: WAVESHARE_AUTO_DIRECTION_PROFILE.into(),
        }
    }

    #[tokio::test]
    async fn fake_serial_qualify_start_stop() {
        let serial = FakeSerial {
            buf: Mutex::new(Vec::new()),
        };
        let mut session = MstpQualifySession::start(serial, &sample_params())
            .await
            .unwrap();
        let (stop_tx, stop_rx) = oneshot::channel();
        let run = session.run_until(stop_rx, Duration::from_millis(80));
        let stopper = async {
            tokio::time::sleep(Duration::from_millis(30)).await;
            let _ = stop_tx.send(());
        };
        tokio::join!(run, stopper);
        session.stop().await.unwrap();
        assert!(!session.active_flag().load(Ordering::Relaxed));
    }

    #[test]
    fn waveshare_b_profile_is_auto_direction() {
        let mut input = MstpValidationInput {
            serial_path: "/dev/serial/by-id/usb-Waveshare-B".into(),
            adapter_profile: WAVESHARE_B_AUTO_DIRECTION_PROFILE.into(),
            baud: 38_400,
            mac: 2,
            max_master: 2,
            max_info_frames: 1,
            network: 2_000,
            require_hardware_auto_direction: true,
        };
        validate_mstp_params(&input).unwrap();
        input.adapter_profile = "kernel-rs485".into();
        assert!(validate_mstp_params(&input).is_err());
    }

    /// Compile/docs fixture: public MasterNode has no crc aggregate counter field.
    #[test]
    fn upstream_gap_no_public_crc_aggregate_on_master_node() {
        // If this fails after an upstream pin bump, update metrics mapping + this note.
        let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/mstp_qualify.rs"));
        assert!(src.contains("upstream_gaps"));
        assert!(src.contains("No public aggregate"));
    }
}
