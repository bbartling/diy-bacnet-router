//! Domain types for the router appliance.
//!
//! This crate intentionally has no web, serial or BACnet-stack dependency. It
//! owns stable configuration and metrics contracts used by every adapter.

pub mod auth;
pub mod config;
pub mod metrics;
pub mod runtime;

pub use auth::{
    management_writes_enabled, AuditEvent, AuditLog, AUDIT_CAPACITY, WRITES_BLOCKED_DETAIL,
};
pub use config::{
    BacnetIpConfig, ConfigError, IdentityConfig, ManagementConfig, MstpConfig, RouterConfig,
    RouterControlConfig,
};
pub use metrics::{Counters, RouterMetrics};
pub use runtime::{Capability, CapabilityState, DataPlaneState, RuntimeSnapshot, RuntimeState};
