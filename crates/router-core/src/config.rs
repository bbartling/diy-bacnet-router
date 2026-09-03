use std::{fs, net::SocketAddr, path::Path};

use serde::{Deserialize, Serialize};
use thiserror::Error;

const SUPPORTED_BAUD: [u32; 6] = [9_600, 19_200, 38_400, 57_600, 76_800, 115_200];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default, deny_unknown_fields)]
pub struct RouterConfig {
    pub identity: IdentityConfig,
    pub management: ManagementConfig,
    pub router: RouterControlConfig,
    pub bacnet_ip: BacnetIpConfig,
    pub mstp: MstpConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct IdentityConfig {
    pub name: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct ManagementConfig {
    pub bind: String,
    pub web_root: String,
    pub metrics_interval_ms: u64,
    /// Bounded concurrent `/api/ws/metrics` upgrades (permits released on disconnect).
    pub max_ws_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default, deny_unknown_fields)]
pub struct RouterControlConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct BacnetIpConfig {
    pub interface: String,
    pub bind_address: String,
    pub udp_port: u16,
    pub network: u16,
    pub bbmd_enabled: bool,
    pub foreign_device_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct MstpConfig {
    pub serial: String,
    pub adapter_profile: String,
    pub termination: String,
    pub baud: u32,
    pub mac: u8,
    pub network: u16,
    pub max_master: u8,
    pub max_info_frames: u8,
}

impl Default for IdentityConfig {
    fn default() -> Self {
        Self {
            name: "diy-bacnet-router-lab".to_owned(),
            location: "test bench".to_owned(),
        }
    }
}

impl Default for ManagementConfig {
    fn default() -> Self {
        Self {
            bind: "127.0.0.1:8080".to_owned(),
            web_root: "frontend/web/dist".to_owned(),
            metrics_interval_ms: 1_000,
            max_ws_connections: 8,
        }
    }
}

impl Default for BacnetIpConfig {
    fn default() -> Self {
        Self {
            interface: "eth0".to_owned(),
            bind_address: "0.0.0.0".to_owned(),
            udp_port: 47_808,
            network: 1,
            bbmd_enabled: false,
            foreign_device_enabled: false,
        }
    }
}

impl Default for MstpConfig {
    fn default() -> Self {
        Self {
            serial: "/dev/serial/by-id/REPLACE_ME".to_owned(),
            adapter_profile: "waveshare-usb-to-rs485-c".to_owned(),
            termination: "onboard-present".to_owned(),
            baud: 38_400,
            mac: 3,
            network: 2_000,
            max_master: 127,
            max_info_frames: 1,
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("cannot read configuration {path}: {source}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("invalid TOML in {path}: {source}")]
    Parse {
        path: String,
        source: toml::de::Error,
    },
    #[error("invalid configuration: {0}")]
    Validation(String),
}

impl RouterConfig {
    /// Load and validate a TOML configuration file.
    pub fn from_path(path: &Path) -> Result<Self, ConfigError> {
        let path_display = path.display().to_string();
        let raw = fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path_display.clone(),
            source,
        })?;
        let config = toml::from_str::<Self>(&raw).map_err(|source| ConfigError::Parse {
            path: path_display,
            source,
        })?;
        config.validate()?;
        Ok(config)
    }

    /// Validate invariants before either BACnet port is opened.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.identity.name.trim().is_empty() {
            return Err(ConfigError::Validation(
                "identity.name must not be empty".into(),
            ));
        }

        self.management
            .bind
            .parse::<SocketAddr>()
            .map_err(|error| {
                ConfigError::Validation(format!(
                    "management.bind must be an IP socket address: {error}"
                ))
            })?;

        if !(250..=5_000).contains(&self.management.metrics_interval_ms) {
            return Err(ConfigError::Validation(
                "management.metrics_interval_ms must be in 250..=5000".into(),
            ));
        }
        if !(1..=32).contains(&self.management.max_ws_connections) {
            return Err(ConfigError::Validation(
                "management.max_ws_connections must be in 1..=32".into(),
            ));
        }

        if !(1..=65_534).contains(&self.bacnet_ip.network) {
            return Err(ConfigError::Validation(
                "bacnet_ip.network must be in 1..=65534".into(),
            ));
        }
        if !(1..=65_534).contains(&self.mstp.network) {
            return Err(ConfigError::Validation(
                "mstp.network must be in 1..=65534".into(),
            ));
        }
        if self.bacnet_ip.network == self.mstp.network {
            return Err(ConfigError::Validation(
                "BACnet/IP and MS/TP network numbers must be distinct".into(),
            ));
        }
        if self.bacnet_ip.udp_port == 0 {
            return Err(ConfigError::Validation(
                "bacnet_ip.udp_port must not be zero".into(),
            ));
        }
        if self.bacnet_ip.interface.trim().is_empty() {
            return Err(ConfigError::Validation(
                "bacnet_ip.interface must not be empty".into(),
            ));
        }
        if !SUPPORTED_BAUD.contains(&self.mstp.baud) {
            return Err(ConfigError::Validation(format!(
                "mstp.baud must be one of {SUPPORTED_BAUD:?}"
            )));
        }
        if self.mstp.mac > 127 {
            return Err(ConfigError::Validation(
                "mstp.mac must be in 0..=127".into(),
            ));
        }
        if self.mstp.max_master > 127 || self.mstp.mac > self.mstp.max_master {
            return Err(ConfigError::Validation(
                "mstp.mac must be <= mstp.max_master <= 127".into(),
            ));
        }
        if self.mstp.max_info_frames == 0 {
            return Err(ConfigError::Validation(
                "mstp.max_info_frames must be in 1..=255".into(),
            ));
        }
        if !self.mstp.serial.starts_with("/dev/serial/by-id/") {
            return Err(ConfigError::Validation(
                "mstp.serial must use a stable /dev/serial/by-id path".into(),
            ));
        }
        if ![
            "onboard-present",
            "external",
            "switchable-disabled",
            "unknown",
        ]
        .contains(&self.mstp.termination.as_str())
        {
            return Err(ConfigError::Validation(
                "mstp.termination must be onboard-present, external, switchable-disabled, or unknown"
                    .into(),
            ));
        }
        if self.router.enabled {
            return Err(ConfigError::Validation(
                "router.enabled=true is unavailable until the rusty-bacnet routing gate passes"
                    .into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_safe_and_valid() {
        RouterConfig::default().validate().expect("default config");
        assert!(!RouterConfig::default().router.enabled);
    }

    #[test]
    fn all_supported_baud_rates_validate() {
        for baud in SUPPORTED_BAUD {
            let mut config = RouterConfig::default();
            config.mstp.baud = baud;
            config
                .validate()
                .unwrap_or_else(|error| panic!("baud {baud}: {error}"));
        }
    }

    #[test]
    fn rejects_duplicate_network_numbers() {
        let mut config = RouterConfig::default();
        config.mstp.network = config.bacnet_ip.network;
        assert!(config.validate().is_err());
    }

    #[test]
    fn rejects_unstable_serial_alias() {
        let mut config = RouterConfig::default();
        config.mstp.serial = "/dev/ttyUSB0".into();
        assert!(config.validate().is_err());
    }

    #[test]
    fn forwarding_is_fail_closed_during_scaffold() {
        let mut config = RouterConfig::default();
        config.router.enabled = true;
        assert!(config.validate().is_err());
    }

    #[test]
    fn unknown_toml_fields_are_rejected() {
        let raw = "[router]\nenabled = false\nsurprise = true\n";
        assert!(toml::from_str::<RouterConfig>(raw).is_err());
    }

    #[test]
    fn example_router_toml_is_valid() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../config/router.example.toml");
        RouterConfig::from_path(&path).expect("example router.toml");
    }

    #[test]
    fn metrics_interval_bounds_are_enforced() {
        let mut config = RouterConfig::default();
        config.management.metrics_interval_ms = 249;
        assert!(config.validate().is_err());
        config.management.metrics_interval_ms = 5_001;
        assert!(config.validate().is_err());
    }

    #[test]
    fn max_ws_connections_bounds_are_enforced() {
        let mut config = RouterConfig::default();
        config.management.max_ws_connections = 0;
        assert!(config.validate().is_err());
        config.management.max_ws_connections = 33;
        assert!(config.validate().is_err());
    }

    /// Top-level RouterConfig keys must stay on the public allowlist. Secret-ish
    /// field names are rejected so `/api/config/effective` cannot silently grow
    /// credentials without an intentional PublicEffectiveConfig update.
    #[test]
    fn router_config_fields_stay_public_allowlisted() {
        const ALLOWED_TOP_LEVEL: &[&str] =
            &["identity", "management", "router", "bacnet_ip", "mstp"];
        const FORBIDDEN_SUBSTRINGS: &[&str] = &[
            "password",
            "secret",
            "token",
            "api_key",
            "credential",
            "private_key",
        ];

        let value = serde_json::to_value(RouterConfig::default()).expect("json");
        let obj = value.as_object().expect("object");
        for key in obj.keys() {
            assert!(
                ALLOWED_TOP_LEVEL.contains(&key.as_str()),
                "unexpected RouterConfig top-level field `{key}` — update PublicEffectiveConfig"
            );
            let lower = key.to_ascii_lowercase();
            for needle in FORBIDDEN_SUBSTRINGS {
                assert!(
                    !lower.contains(needle),
                    "RouterConfig field `{key}` looks secret; keep it out of the public API"
                );
            }
        }
        walk_forbid_secret_keys(&value, FORBIDDEN_SUBSTRINGS);
    }

    fn walk_forbid_secret_keys(value: &serde_json::Value, needles: &[&str]) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, child) in map {
                    let lower = key.to_ascii_lowercase();
                    for needle in needles {
                        assert!(
                            !lower.contains(needle),
                            "config key `{key}` looks secret; redact before exposing"
                        );
                    }
                    walk_forbid_secret_keys(child, needles);
                }
            }
            serde_json::Value::Array(items) => {
                for child in items {
                    walk_forbid_secret_keys(child, needles);
                }
            }
            _ => {}
        }
    }
}
