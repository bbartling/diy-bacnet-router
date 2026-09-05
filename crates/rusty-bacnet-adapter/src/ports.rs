//! Concrete B/IP and MS/TP transport factories (no OS open in unit paths).

use std::net::Ipv4Addr;

use bacnet_network::router::RouterPort;
use bacnet_transport::any::AnyTransport;
use bacnet_transport::bip::BipTransport;
use bacnet_transport::mstp::{MstpConfig as UpstreamMstpConfig, MstpTransport, SerialPort};
use bacnet_transport::mstp_serial::TokioSerialPort;

use crate::validate::{
    validate_bip_params, validate_distinct_networks, validate_mstp_params, AdapterError,
    BipValidationInput, MstpValidationInput,
};

/// Product serial type for Waveshare auto-direction adapters.
pub type ApplianceSerial = TokioSerialPort;

/// Heterogeneous transport enum bound to the appliance serial type.
pub type ApplianceTransport = AnyTransport<ApplianceSerial>;

/// Router port using concrete B/IP + MS/TP-capable `AnyTransport`.
pub type ApplianceRouterPort = RouterPort<ApplianceTransport>;

#[derive(Debug, Clone)]
pub struct BipTransportParams {
    pub interface_addr: Ipv4Addr,
    pub udp_port: u16,
    pub broadcast_address: Ipv4Addr,
    pub network: u16,
    pub interface_name: String,
}

#[derive(Debug, Clone)]
pub struct MstpTransportParams {
    pub this_station: u8,
    pub max_master: u8,
    pub max_info_frames: u8,
    pub baud_rate: u32,
    pub network: u16,
    pub serial_path: String,
    pub adapter_profile: String,
}

/// Build a `BipTransport` via public `BipTransport::new` **without** calling `start()`.
pub fn build_bip_transport(params: &BipTransportParams) -> Result<BipTransport, AdapterError> {
    validate_bip_params(&BipValidationInput {
        bind_address: params.interface_addr.to_string(),
        broadcast_address: params.broadcast_address.to_string(),
        udp_port: params.udp_port,
        network: params.network,
        interface: params.interface_name.clone(),
    })?;
    Ok(BipTransport::new(
        params.interface_addr,
        params.udp_port,
        params.broadcast_address,
    ))
}

/// Build `MstpTransport` around an **already-open** `SerialPort` (no `TokioSerialPort::open`).
///
/// Product path: open with `TokioSerialPort::open` in an explicitly named `open_*`
/// helper later; unit tests inject a fake `SerialPort`.
pub fn build_mstp_transport<S: SerialPort>(
    serial: S,
    params: &MstpTransportParams,
) -> Result<MstpTransport<S>, AdapterError> {
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
    let config = UpstreamMstpConfig {
        this_station: params.this_station,
        max_master: params.max_master,
        max_info_frames: params.max_info_frames,
        baud_rate: params.baud_rate,
    };
    Ok(MstpTransport::new(serial, config))
}

/// Construct heterogeneous `RouterPort`s for B/IP + MS/TP **without** starting the router.
pub fn build_heterogeneous_ports(
    bip: BipTransport,
    mstp: MstpTransport<ApplianceSerial>,
    bip_network: u16,
    mstp_network: u16,
) -> Result<Vec<ApplianceRouterPort>, AdapterError> {
    validate_distinct_networks(bip_network, mstp_network)?;
    Ok(vec![
        RouterPort {
            transport: AnyTransport::Bip(bip),
            network_number: bip_network,
        },
        RouterPort {
            transport: AnyTransport::Mstp(mstp),
            network_number: mstp_network,
        },
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::WAVESHARE_AUTO_DIRECTION_PROFILE;
    use bacnet_types::error::Error as BacnetError;
    use std::sync::Mutex;

    /// Test double: implements `SerialPort` without opening a tty.
    struct FakeSerial {
        buf: Mutex<Vec<u8>>,
    }

    impl SerialPort for FakeSerial {
        async fn write(&self, data: &[u8]) -> Result<(), BacnetError> {
            self.buf.lock().unwrap().extend_from_slice(data);
            Ok(())
        }

        async fn read(&self, buf: &mut [u8]) -> Result<usize, BacnetError> {
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

    fn sample_bip() -> BipTransportParams {
        BipTransportParams {
            interface_addr: Ipv4Addr::new(192, 0, 2, 1),
            udp_port: 47_808,
            broadcast_address: Ipv4Addr::new(192, 0, 2, 255),
            network: 1,
            interface_name: "eth0".into(),
        }
    }

    fn sample_mstp() -> MstpTransportParams {
        MstpTransportParams {
            this_station: 3,
            max_master: 127,
            max_info_frames: 1,
            baud_rate: 38_400,
            network: 2_000,
            serial_path: "/dev/serial/by-id/usb-Waveshare-if00".into(),
            adapter_profile: WAVESHARE_AUTO_DIRECTION_PROFILE.into(),
        }
    }

    #[test]
    fn bip_factory_constructs_without_start() {
        let transport = build_bip_transport(&sample_bip()).unwrap();
        drop(transport);
    }

    #[test]
    fn mstp_factory_accepts_already_open_serial_without_tty() {
        let serial = FakeSerial {
            buf: Mutex::new(Vec::new()),
        };
        let transport = build_mstp_transport(serial, &sample_mstp()).unwrap();
        drop(transport);
    }

    #[test]
    fn appliance_type_alias_is_any_transport() {
        let _ = WAVESHARE_AUTO_DIRECTION_PROFILE;
        let name = std::any::type_name::<ApplianceTransport>();
        assert!(
            name.contains("AnyTransport") || name.contains("any"),
            "unexpected type name {name}"
        );
    }

    #[test]
    fn heterogeneous_bip_mstp_with_fake_serial() {
        let bip = build_bip_transport(&sample_bip()).unwrap();
        let serial = FakeSerial {
            buf: Mutex::new(Vec::new()),
        };
        let mstp = build_mstp_transport(serial, &sample_mstp()).unwrap();
        let ports = [
            RouterPort {
                transport: AnyTransport::<FakeSerial>::Bip(bip),
                network_number: 1,
            },
            RouterPort {
                transport: AnyTransport::Mstp(mstp),
                network_number: 2_000,
            },
        ];
        assert_eq!(ports.len(), 2);
        assert_ne!(ports[0].network_number, ports[1].network_number);
    }
}
