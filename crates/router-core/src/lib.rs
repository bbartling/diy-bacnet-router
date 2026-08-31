//! Domain types for the router appliance.
//!
//! This crate intentionally has no web, serial or BACnet-stack dependency. It
//! owns stable configuration and metrics contracts used by every adapter.

pub mod config;
pub mod metrics;
pub mod runtime;

pub use config::{ConfigError, RouterConfig};
pub use metrics::{Counters, RouterMetrics};
pub use runtime::{Capability, CapabilityState, DataPlaneState, RuntimeSnapshot, RuntimeState};
