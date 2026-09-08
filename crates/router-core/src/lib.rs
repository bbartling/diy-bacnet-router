//! Domain types for the router appliance.
//!
//! This crate intentionally has no web, serial or BACnet-stack dependency. It
//! owns stable configuration and metrics contracts used by every adapter.

pub mod auth;
pub mod config;
pub mod metrics;
pub mod runtime;

pub use auth::{
    hash_password_experimental, management_writes_enabled, management_writes_enabled_with,
    verify_password_experimental, write_secret_path, write_secret_present, AuditEvent, AuditLog,
    SessionStore, SessionToken, AUDIT_CAPACITY, DEFAULT_WRITE_SECRET_PATH, SESSION_CAPACITY,
    WRITES_BLOCKED_DETAIL,
};
pub use config::{
    BacnetIpConfig, ConfigError, IdentityConfig, ManagementConfig, MstpConfig, RouterConfig,
    RouterControlConfig,
};
pub use metrics::{Counters, RouterMetrics};
pub use runtime::{Capability, CapabilityState, DataPlaneState, RuntimeSnapshot, RuntimeState};
