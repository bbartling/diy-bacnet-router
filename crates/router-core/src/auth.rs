//! M6 management-write scaffolding (disabled until auth/audit gates pass).
//!
//! Browser configuration writes remain refused by default. Lab unlock requires
//! **both** `DBR_ALLOW_WRITES=1` and a present write-secret file. e2e/QEMU must
//! never set the env. Capability `management_writes` stays `BlockedByEvidence`.

use std::collections::VecDeque;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Hard ceiling for in-memory audit events (no unbounded growth).
pub const AUDIT_CAPACITY: usize = 256;

/// Hard ceiling for in-memory session tokens.
pub const SESSION_CAPACITY: usize = 32;

/// Stable reason returned when writes are refused.
pub const WRITES_BLOCKED_DETAIL: &str =
    "management writes blocked until M6 auth/audit evidence gates pass";

/// Default secret path when `DBR_WRITE_SECRET_FILE` is unset.
pub const DEFAULT_WRITE_SECRET_PATH: &str = "/etc/diy-bacnet-router/write.secret";

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

/// Bounded session token ring (lab skeleton; not production auth).
#[derive(Debug)]
pub struct SessionStore {
    tokens: Mutex<VecDeque<SessionToken>>,
    capacity: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionToken {
    pub token: String,
    pub expires_unix_ms: u128,
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new(SESSION_CAPACITY)
    }
}

impl SessionStore {
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            tokens: Mutex::new(VecDeque::with_capacity(capacity.min(SESSION_CAPACITY))),
            capacity: capacity.clamp(1, SESSION_CAPACITY),
        }
    }

    pub fn issue(&self, token: impl Into<String>, ttl_ms: u128) -> SessionToken {
        let entry = SessionToken {
            token: token.into(),
            expires_unix_ms: now_ms().saturating_add(ttl_ms),
        };
        let mut guard = self
            .tokens
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if guard.len() >= self.capacity {
            guard.pop_front();
        }
        guard.push_back(entry.clone());
        entry
    }

    #[must_use]
    pub fn validate(&self, token: &str) -> bool {
        let now = now_ms();
        let mut guard = self
            .tokens
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.retain(|t| t.expires_unix_ms > now);
        guard.iter().any(|t| t.token == token)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.tokens
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// **Experimental / lab-only** password digest: SHA-256(salt || ":" || password) hex.
/// Not argon2/bcrypt; not for production appliances.
#[must_use]
pub fn hash_password_experimental(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(b":");
    hasher.update(password.as_bytes());
    hex_encode(&hasher.finalize())
}

/// Verify against [`hash_password_experimental`] (constant-time-ish byte compare).
#[must_use]
pub fn verify_password_experimental(password: &str, salt: &str, expected_hex: &str) -> bool {
    let got = hash_password_experimental(password, salt);
    if got.len() != expected_hex.len() {
        return false;
    }
    got.bytes()
        .zip(expected_hex.bytes())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

/// Resolve write-secret path (env override for tests/lab).
#[must_use]
pub fn write_secret_path() -> PathBuf {
    env::var("DBR_WRITE_SECRET_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_WRITE_SECRET_PATH))
}

#[must_use]
pub fn write_secret_present() -> bool {
    let path = write_secret_path();
    Path::new(&path).is_file()
}

/// Product policy: writes stay closed unless **explicit** lab unlock.
///
/// Requires `DBR_ALLOW_WRITES=1` **and** a present secret file. Never set in e2e/QEMU.
#[must_use]
pub fn management_writes_enabled() -> bool {
    management_writes_enabled_with(
        env::var("DBR_ALLOW_WRITES").ok().as_deref() == Some("1"),
        write_secret_present(),
    )
}

/// Testable core of [`management_writes_enabled`] (no process env mutation).
#[must_use]
pub fn management_writes_enabled_with(allow_writes_env: bool, secret_file_present: bool) -> bool {
    allow_writes_env && secret_file_present
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0xf) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn writes_default_disabled_without_unlock() {
        assert!(!management_writes_enabled_with(false, false));
        assert!(!management_writes_enabled_with(true, false));
        assert!(!management_writes_enabled_with(false, true));
    }

    #[test]
    fn writes_require_env_and_secret_file() {
        assert!(management_writes_enabled_with(true, true));
        let dir = std::env::temp_dir().join(format!("dbr-m6-secret-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let secret = dir.join("write.secret");
        fs::write(&secret, b"lab-only\n").unwrap();
        assert!(secret.is_file());
        assert!(management_writes_enabled_with(true, secret.is_file()));
        fs::remove_file(&secret).unwrap();
        assert!(!management_writes_enabled_with(true, secret.is_file()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn audit_ring_is_bounded() {
        let log = AuditLog::new(4);
        for i in 0..10 {
            log.record("test", "probe", "ok", &format!("{i}"));
        }
        assert_eq!(log.len(), 4);
        let snap = log.snapshot();
        assert_eq!(snap.first().map(|e| e.detail.as_str()), Some("6"));
    }

    #[test]
    fn session_ring_validates_and_bounds() {
        let store = SessionStore::new(2);
        store.issue("a", 60_000);
        store.issue("b", 60_000);
        store.issue("c", 60_000);
        assert_eq!(store.len(), 2);
        assert!(!store.validate("a"));
        assert!(store.validate("b"));
        assert!(store.validate("c"));
    }

    #[test]
    fn experimental_password_roundtrip() {
        let hash = hash_password_experimental("lab-pass", "salt1");
        assert!(verify_password_experimental("lab-pass", "salt1", &hash));
        assert!(!verify_password_experimental("wrong", "salt1", &hash));
        assert_eq!(hash.len(), 64);
    }
}
