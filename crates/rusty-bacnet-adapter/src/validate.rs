//! Product policy validation without opening sockets or ttys.

use std::net::Ipv4Addr;

use thiserror::Error;

/// Baud rates allowed for appliance MS/TP operation.
pub const SUPPORTED_BAUD: [u32; 6] = [9_600, 19_200, 38_400, 57_600, 76_800, 115_200];

/// Waveshare USB TO RS485 (C) uses hardware automatic DE/RE — no Linux RS-485 ioctl/GPIO.
pub const WAVESHARE_AUTO_DIRECTION_PROFILE: &str = "waveshare-usb-to-rs485-c";
/// Waveshare USB TO RS485 (B) also uses hardware automatic DE/RE (peer / Vibe13 path).
pub const WAVESHARE_B_AUTO_DIRECTION_PROFILE: &str = "waveshare-usb-to-rs485-b";

fn is_hardware_auto_direction_profile(profile: &str) -> bool {
    profile == WAVESHARE_AUTO_DIRECTION_PROFILE
        || profile == WAVESHARE_B_AUTO_DIRECTION_PROFILE
        || profile.contains("auto")
}

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("BACnet networks must be distinct and in 1..=65534 (got {0} and {1})")]
    InvalidNetworks(u16, u16),
    #[error("{0}")]
    Validation(String),
    #[error(transparent)]
    Upstream(#[from] bacnet_types::error::Error),
}

pub fn validate_distinct_networks(a: u16, b: u16) -> Result<(), AdapterError> {
    let ok = |n: u16| (1..=65_534).contains(&n);
    if a == b || !ok(a) || !ok(b) {
        return Err(AdapterError::InvalidNetworks(a, b));
    }
    Ok(())
}

pub fn validate_serial_path(path: &str) -> Result<(), AdapterError> {
    if !path.starts_with("/dev/serial/by-id/") || path.len() <= "/dev/serial/by-id/".len() {
        return Err(AdapterError::Validation(
            "MS/TP serial path must be a stable /dev/serial/by-id/... path".into(),
        ));
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct MstpValidationInput {
    pub serial_path: String,
    pub adapter_profile: String,
    pub baud: u32,
    pub mac: u8,
    pub max_master: u8,
    pub max_info_frames: u8,
    pub network: u16,
    /// When true, refuse kernel RS-485 ioctl / GPIO direction modes.
    pub require_hardware_auto_direction: bool,
}

pub fn validate_mstp_params(input: &MstpValidationInput) -> Result<(), AdapterError> {
    if !(1..=65_534).contains(&input.network) {
        return Err(AdapterError::Validation(
            "MS/TP network must be in 1..=65534".into(),
        ));
    }
    validate_serial_path(&input.serial_path)?;
    if !SUPPORTED_BAUD.contains(&input.baud) {
        return Err(AdapterError::Validation(format!(
            "MS/TP baud must be one of {SUPPORTED_BAUD:?} (default 38400)"
        )));
    }
    if input.mac > 127 {
        return Err(AdapterError::Validation("MS/TP MAC must be <= 127".into()));
    }
    if input.max_master > 127 || input.mac > input.max_master {
        return Err(AdapterError::Validation(
            "MS/TP MAC must be <= Max_Master <= 127".into(),
        ));
    }
    if input.max_info_frames == 0 {
        return Err(AdapterError::Validation(
            "Max_Info_Frames must be in 1..=255".into(),
        ));
    }
    if input.require_hardware_auto_direction
        && !is_hardware_auto_direction_profile(&input.adapter_profile)
    {
        return Err(AdapterError::Validation(
            "this profile requires hardware auto-direction (no simultaneous Linux RS-485 ioctl/RTS/GPIO)"
                .into(),
        ));
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct BipValidationInput {
    pub bind_address: String,
    pub broadcast_address: String,
    pub udp_port: u16,
    pub network: u16,
    pub interface: String,
}

pub fn validate_bip_params(input: &BipValidationInput) -> Result<(), AdapterError> {
    if !(1..=65_534).contains(&input.network) {
        return Err(AdapterError::Validation(
            "B/IP network must be in 1..=65534".into(),
        ));
    }
    if input.udp_port == 0 {
        return Err(AdapterError::Validation(
            "B/IP UDP port must not be zero".into(),
        ));
    }
    if input.interface.trim().is_empty() {
        return Err(AdapterError::Validation(
            "B/IP interface name must not be empty".into(),
        ));
    }
    let bind: Ipv4Addr = input.bind_address.parse().map_err(|_| {
        AdapterError::Validation("B/IP bind_address must be a valid IPv4 address".into())
    })?;
    let bcast: Ipv4Addr = input.broadcast_address.parse().map_err(|_| {
        AdapterError::Validation("B/IP broadcast_address must be a valid IPv4 address".into())
    })?;
    // Construction-only check: addresses parse. Interface↔address ownership is
    // enforced later by M2A qualification, not here (no netlink/socket).
    let _ = (bind, bcast);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serial_path_requires_by_id() {
        assert!(validate_serial_path("/dev/ttyUSB0").is_err());
        assert!(validate_serial_path("/dev/serial/by-id/").is_err());
        assert!(validate_serial_path("/dev/serial/by-id/usb-FTDI-if00").is_ok());
    }

    #[test]
    fn mstp_defaults_validate() {
        let input = MstpValidationInput {
            serial_path: "/dev/serial/by-id/usb-Waveshare".into(),
            adapter_profile: WAVESHARE_AUTO_DIRECTION_PROFILE.into(),
            baud: 38_400,
            mac: 3,
            max_master: 127,
            max_info_frames: 1,
            network: 2_000,
            require_hardware_auto_direction: true,
        };
        validate_mstp_params(&input).unwrap();
    }

    #[test]
    fn mstp_rejects_bad_baud_and_mac() {
        let mut input = MstpValidationInput {
            serial_path: "/dev/serial/by-id/usb-Waveshare".into(),
            adapter_profile: WAVESHARE_AUTO_DIRECTION_PROFILE.into(),
            baud: 38_400,
            mac: 3,
            max_master: 127,
            max_info_frames: 1,
            network: 2_000,
            require_hardware_auto_direction: true,
        };
        input.baud = 1200;
        assert!(validate_mstp_params(&input).is_err());
        input.baud = 38_400;
        input.mac = 10;
        input.max_master = 5;
        assert!(validate_mstp_params(&input).is_err());
    }

    #[test]
    fn bip_parses_addresses_without_socket() {
        validate_bip_params(&BipValidationInput {
            bind_address: "192.0.2.1".into(),
            broadcast_address: "192.0.2.255".into(),
            udp_port: 47_808,
            network: 1,
            interface: "eth0".into(),
        })
        .unwrap();
        assert!(validate_bip_params(&BipValidationInput {
            bind_address: "not-an-ip".into(),
            broadcast_address: "192.0.2.255".into(),
            udp_port: 47_808,
            network: 1,
            interface: "eth0".into(),
        })
        .is_err());
    }
}
