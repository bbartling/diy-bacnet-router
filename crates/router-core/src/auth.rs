//! M6 management-write scaffolding (disabled until auth/audit gates pass).
//!
//! Browser configuration writes remain refused. This module owns the bounded
//! audit ring and the explicit allow-list check used by the management API.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// Hard ceiling for in-memory audit events (no unbounded growth).
pub const AUDIT_CAPACITY: usize = 256;

/// Stable reason returned when writes are refused.
pub const WRITES_BLOCKED_DETAIL: &str =
    "management writes blocked until M6 auth/audit evidence gates pass";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditEvent {
    pub timestamp_unix_ms: u128,
    pub actor: String,
    pub action: String,
    pub outcome: String,
    pub detail: String,
}

#[derive(Debug)]
pub struct AuditLog {
    events: Mutex<VecDeque<AuditEvent>>,
    capacity: usize,
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new(AUDIT_CAPACITY)
    }
}

impl AuditLog {
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            events: Mutex::new(VecDeque::with_capacity(capacity.min(AUDIT_CAPACITY))),
            capacity: capacity.clamp(1, AUDIT_CAPACITY),
        }
    }

    pub fn record(&self, actor: &str, action: &str, outcome: &str, detail: &str) {
        let event = AuditEvent {
            timestamp_unix_ms: now_ms(),
            actor: actor.to_owned(),
            action: action.to_owned(),
            outcome: outcome.to_owned(),
            detail: detail.to_owned(),
        };
        let mut guard = self
            .events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if guard.len() >= self.capacity {
            guard.pop_front();
        }
        guard.push_back(event);
    }

    #[must_use]
    pub fn snapshot(&self) -> Vec<AuditEvent> {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .cloned()
            .collect()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Product policy: management mutating APIs stay closed until M6 evidence.
#[must_use]
pub fn management_writes_enabled() -> bool {
    false
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_remain_disabled() {
        assert!(!management_writes_enabled());
    }

    #[test]
    fn audit_ring_is_bounded() {
        let log = AuditLog::new(4);
        for i in 0..10 {
            log.record("test", "probe", "ok", &format!("{i}"));
        }
        assert_eq!(log.len(), 4);
        let snap = log.snapshot();
        assert_eq!(snap.first().unwrap().detail, "6");
        assert_eq!(snap.last().unwrap().detail, "9");
    }
}
