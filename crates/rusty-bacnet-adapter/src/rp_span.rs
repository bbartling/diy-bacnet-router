//! Bounded confirmed-ReadProperty timing spans for issue #66 diagnosis.
//!
//! Records peer+invoke+DNET/MAC+object/property keys without property values.
//! Hot path uses atomics + a small mutex ring; unavailable fields stay `null`
//! in JSON (never zero-filled as if measured).

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use serde::Serialize;

const RING_CAP: usize = 256;

/// One completed (or timed-out) confirmed-service timing sample.
#[derive(Debug, Clone, Serialize)]
pub struct RpSpanSample {
    pub started_ms: u64,
    pub peer: String,
    pub invoke_id: Option<u8>,
    pub dnet: Option<u16>,
    pub dmac: Option<u8>,
    pub object: String,
    pub property: String,
    /// Monotonic ms from span open → BIP ingress mark (unknown if None).
    pub bip_ingress_ms: Option<u64>,
    pub route_decision_ms: Option<u64>,
    pub mstp_enqueue_ms: Option<u64>,
    pub mstp_tx_ms: Option<u64>,
    pub mstp_rx_ms: Option<u64>,
    pub bip_egress_ms: Option<u64>,
    pub total_ms: Option<u64>,
    pub outcome: &'static str,
}

/// Live counters + ring buffer.
#[derive(Debug)]
pub struct RpSpanStore {
    pub opened: AtomicU64,
    pub completed_ok: AtomicU64,
    pub completed_timeout: AtomicU64,
    pub completed_error: AtomicU64,
    pub queue_full_drops: AtomicU64,
    ring: Mutex<VecDeque<RpSpanSample>>,
    epoch: Instant,
}

impl Default for RpSpanStore {
    fn default() -> Self {
        Self::new()
    }
}

impl RpSpanStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            opened: AtomicU64::new(0),
            completed_ok: AtomicU64::new(0),
            completed_timeout: AtomicU64::new(0),
            completed_error: AtomicU64::new(0),
            queue_full_drops: AtomicU64::new(0),
            ring: Mutex::new(VecDeque::with_capacity(RING_CAP)),
            epoch: Instant::now(),
        }
    }

    fn now_ms(&self) -> u64 {
        self.epoch.elapsed().as_millis() as u64
    }

    /// Open a span; returns a handle id (= started_ms for correlation).
    pub fn open(
        &self,
        peer: impl Into<String>,
        invoke_id: Option<u8>,
        dnet: Option<u16>,
        dmac: Option<u8>,
        object: impl Into<String>,
        property: impl Into<String>,
    ) -> u64 {
        self.opened.fetch_add(1, Ordering::Relaxed);
        let started = self.now_ms();
        let sample = RpSpanSample {
            started_ms: started,
            peer: peer.into(),
            invoke_id,
            dnet,
            dmac,
            object: object.into(),
            property: property.into(),
            bip_ingress_ms: Some(0),
            route_decision_ms: None,
            mstp_enqueue_ms: None,
            mstp_tx_ms: None,
            mstp_rx_ms: None,
            bip_egress_ms: None,
            total_ms: None,
            outcome: "open",
        };
        if let Ok(mut ring) = self.ring.lock() {
            if ring.len() >= RING_CAP {
                ring.pop_front();
            }
            ring.push_back(sample);
        }
        started
    }

    pub fn mark(&self, started_ms: u64, field: SpanField, elapsed_from_start_ms: u64) {
        let Ok(mut ring) = self.ring.lock() else {
            return;
        };
        if let Some(s) = ring.iter_mut().rev().find(|s| s.started_ms == started_ms) {
            match field {
                SpanField::RouteDecision => s.route_decision_ms = Some(elapsed_from_start_ms),
                SpanField::MstpEnqueue => s.mstp_enqueue_ms = Some(elapsed_from_start_ms),
                SpanField::MstpTx => s.mstp_tx_ms = Some(elapsed_from_start_ms),
                SpanField::MstpRx => s.mstp_rx_ms = Some(elapsed_from_start_ms),
                SpanField::BipEgress => s.bip_egress_ms = Some(elapsed_from_start_ms),
            }
        }
    }

    pub fn finish(&self, started_ms: u64, outcome: &'static str) {
        match outcome {
            "ok" => {
                self.completed_ok.fetch_add(1, Ordering::Relaxed);
            }
            "timeout" => {
                self.completed_timeout.fetch_add(1, Ordering::Relaxed);
            }
            _ => {
                self.completed_error.fetch_add(1, Ordering::Relaxed);
            }
        }
        let total = self.now_ms().saturating_sub(started_ms);
        let Ok(mut ring) = self.ring.lock() else {
            return;
        };
        if let Some(s) = ring.iter_mut().rev().find(|s| s.started_ms == started_ms) {
            s.total_ms = Some(total);
            s.outcome = outcome;
        }
    }

    pub fn note_queue_full_drop(&self) {
        self.queue_full_drops.fetch_add(1, Ordering::Relaxed);
    }

    #[must_use]
    pub fn snapshot(&self) -> RpSpanSnapshot {
        let samples: Vec<RpSpanSample> = self
            .ring
            .lock()
            .map(|r| r.iter().cloned().collect())
            .unwrap_or_default();
        let totals: Vec<u64> = samples.iter().filter_map(|s| s.total_ms).collect();
        RpSpanSnapshot {
            opened: self.opened.load(Ordering::Relaxed),
            completed_ok: self.completed_ok.load(Ordering::Relaxed),
            completed_timeout: self.completed_timeout.load(Ordering::Relaxed),
            completed_error: self.completed_error.load(Ordering::Relaxed),
            queue_full_drops: self.queue_full_drops.load(Ordering::Relaxed),
            sample_count: samples.len(),
            latency_p50_ms: percentile(&totals, 50),
            latency_p95_ms: percentile(&totals, 95),
            latency_p99_ms: percentile(&totals, 99),
            latency_max_ms: totals.iter().copied().max(),
            recent: samples.into_iter().rev().take(32).collect(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SpanField {
    RouteDecision,
    MstpEnqueue,
    MstpTx,
    MstpRx,
    BipEgress,
}

#[derive(Debug, Clone, Serialize)]
pub struct RpSpanSnapshot {
    pub opened: u64,
    pub completed_ok: u64,
    pub completed_timeout: u64,
    pub completed_error: u64,
    pub queue_full_drops: u64,
    pub sample_count: usize,
    pub latency_p50_ms: Option<u64>,
    pub latency_p95_ms: Option<u64>,
    pub latency_p99_ms: Option<u64>,
    pub latency_max_ms: Option<u64>,
    pub recent: Vec<RpSpanSample>,
}

fn percentile(sorted_src: &[u64], pct: u8) -> Option<u64> {
    if sorted_src.is_empty() {
        return None;
    }
    let mut v = sorted_src.to_vec();
    v.sort_unstable();
    let idx = ((pct as usize) * (v.len().saturating_sub(1))) / 100;
    v.get(idx).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_lifecycle_and_percentiles() {
        let store = RpSpanStore::new();
        let id = store.open(
            "192.168.204.11:47808",
            Some(7),
            Some(2001),
            Some(2),
            "AI:1",
            "PV",
        );
        store.mark(id, SpanField::RouteDecision, 1);
        store.mark(id, SpanField::MstpEnqueue, 2);
        store.finish(id, "ok");
        let snap = store.snapshot();
        assert_eq!(snap.opened, 1);
        assert_eq!(snap.completed_ok, 1);
        assert_eq!(snap.sample_count, 1);
        assert!(snap.latency_p50_ms.is_some());
        assert_eq!(snap.recent[0].dnet, Some(2001));
        assert_eq!(snap.recent[0].outcome, "ok");
    }

    #[test]
    fn ring_bounds_memory() {
        let store = RpSpanStore::new();
        for i in 0..(RING_CAP + 50) {
            let id = store.open("p", None, None, None, format!("o{i}"), "pv");
            store.finish(id, "ok");
        }
        assert!(store.snapshot().sample_count <= RING_CAP);
    }

    #[test]
    fn unmatched_late_still_records_timeout() {
        let store = RpSpanStore::new();
        let id = store.open("p", Some(1), Some(2001), Some(2), "AI:1", "PV");
        store.finish(id, "timeout");
        let snap = store.snapshot();
        assert_eq!(snap.completed_timeout, 1);
        assert_eq!(snap.recent[0].outcome, "timeout");
    }
}
