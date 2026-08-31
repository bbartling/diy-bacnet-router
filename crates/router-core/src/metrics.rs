use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

/// Hot-path counters. Writers never wait for the management plane.
#[derive(Debug, Default)]
pub struct Counters {
    pub bip_rx_packets: AtomicU64,
    pub bip_tx_packets: AtomicU64,
    pub mstp_rx_packets: AtomicU64,
    pub mstp_tx_packets: AtomicU64,
    pub forwarded_bip_to_mstp: AtomicU64,
    pub forwarded_mstp_to_bip: AtomicU64,
    pub dropped_packets: AtomicU64,
    pub invalid_frames: AtomicU64,
    pub header_crc_errors: AtomicU64,
    pub data_crc_errors: AtomicU64,
    pub apdu_timeouts: AtomicU64,
    pub serial_reconnects: AtomicU64,
    pub tx_tokens: AtomicU64,
    pub rx_tokens: AtomicU64,
    pub tx_poll_for_master: AtomicU64,
    pub rx_poll_for_master: AtomicU64,
    pub event_count: AtomicU64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RouterMetrics {
    pub bip_rx_packets: u64,
    pub bip_tx_packets: u64,
    pub mstp_rx_packets: u64,
    pub mstp_tx_packets: u64,
    pub forwarded_bip_to_mstp: u64,
    pub forwarded_mstp_to_bip: u64,
    pub dropped_packets: u64,
    pub invalid_frames: u64,
    pub header_crc_errors: u64,
    pub data_crc_errors: u64,
    pub apdu_timeouts: u64,
    pub serial_reconnects: u64,
    pub tx_tokens: u64,
    pub rx_tokens: u64,
    pub tx_poll_for_master: u64,
    pub rx_poll_for_master: u64,
    pub event_count: u64,
}

impl Counters {
    #[must_use]
    pub fn snapshot(&self) -> RouterMetrics {
        let read = |counter: &AtomicU64| counter.load(Ordering::Relaxed);
        RouterMetrics {
            bip_rx_packets: read(&self.bip_rx_packets),
            bip_tx_packets: read(&self.bip_tx_packets),
            mstp_rx_packets: read(&self.mstp_rx_packets),
            mstp_tx_packets: read(&self.mstp_tx_packets),
            forwarded_bip_to_mstp: read(&self.forwarded_bip_to_mstp),
            forwarded_mstp_to_bip: read(&self.forwarded_mstp_to_bip),
            dropped_packets: read(&self.dropped_packets),
            invalid_frames: read(&self.invalid_frames),
            header_crc_errors: read(&self.header_crc_errors),
            data_crc_errors: read(&self.data_crc_errors),
            apdu_timeouts: read(&self.apdu_timeouts),
            serial_reconnects: read(&self.serial_reconnects),
            tx_tokens: read(&self.tx_tokens),
            rx_tokens: read(&self.rx_tokens),
            tx_poll_for_master: read(&self.tx_poll_for_master),
            rx_poll_for_master: read(&self.rx_poll_for_master),
            event_count: read(&self.event_count),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_observes_atomic_updates() {
        let counters = Counters::default();
        counters.rx_tokens.fetch_add(7, Ordering::Relaxed);
        counters.mstp_rx_packets.fetch_add(12, Ordering::Relaxed);
        let snapshot = counters.snapshot();
        assert_eq!(snapshot.rx_tokens, 7);
        assert_eq!(snapshot.mstp_rx_packets, 12);
    }
}
