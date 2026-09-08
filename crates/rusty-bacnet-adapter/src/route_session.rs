//! Opt-in B/IP + MS/TP `BACnetRouter` session (M3 software path).
//!
//! Ordinary appliance boot must not call this. Lab unlock is `--route-enable`.
//! Product routing gates (G7/G8) remain open until isolated bench evidence exists.
//! Does not enable BBMD/FDR. Does not synthesize forward counters.

use bacnet_network::router::BACnetRouter;
use tokio::sync::mpsc;
use tracing::info;

use crate::bip_qualify::assert_bind_on_interface;
use crate::mstp_qualify::open_appliance_serial;
use crate::ports::{
    build_bip_transport, build_heterogeneous_ports, build_mstp_transport, BipTransportParams,
    MstpTransportParams,
};
use crate::validate::{validate_distinct_networks, AdapterError};

/// Live heterogeneous router session (opens one B/IP socket + one serial-by-id).
pub struct ApplianceRouterSession {
    router: BACnetRouter,
    local_rx: mpsc::Receiver<bacnet_network::layer::ReceivedApdu>,
}

impl ApplianceRouterSession {
    /// Validate, open ports, and start Clause 6 half-router forwarding.
    pub async fn start(
        bip: &BipTransportParams,
        mstp: &MstpTransportParams,
    ) -> Result<Self, AdapterError> {
        validate_distinct_networks(bip.network, mstp.network)?;
        assert_bind_on_interface(&bip.interface_name, bip.interface_addr)?;
        let serial = open_appliance_serial(mstp)?;
        let mstp_transport = build_mstp_transport(serial, mstp)?;
        let bip_transport = build_bip_transport(bip)?;
        let ports =
            build_heterogeneous_ports(bip_transport, mstp_transport, bip.network, mstp.network)?;
        let (router, local_rx) = BACnetRouter::start(ports).await?;
        info!(
            bip_net = bip.network,
            mstp_net = mstp.network,
            "appliance router session started (opt-in; G7/G8 evidence still open)"
        );
        Ok(Self { router, local_rx })
    }

    pub fn local_rx(&mut self) -> &mut mpsc::Receiver<bacnet_network::layer::ReceivedApdu> {
        &mut self.local_rx
    }

    pub async fn route_table_len(&self) -> usize {
        self.router.table().lock().await.len()
    }

    pub async fn stop(mut self) -> Result<(), AdapterError> {
        self.router.stop().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::WAVESHARE_AUTO_DIRECTION_PROFILE;
    use std::net::Ipv4Addr;

    fn sample_bip() -> BipTransportParams {
        BipTransportParams {
            interface_addr: Ipv4Addr::new(127, 0, 0, 1),
            udp_port: 0, // rejected by validate — used for pre-open failure path
            broadcast_address: Ipv4Addr::new(127, 0, 0, 1),
            network: 1_000,
            interface_name: "lo".into(),
        }
    }

    fn sample_mstp() -> MstpTransportParams {
        MstpTransportParams {
            this_station: 1,
            max_master: 2,
            max_info_frames: 1,
            baud_rate: 38_400,
            network: 2_000,
            serial_path: "/dev/serial/by-id/usb-Waveshare-C-if00".into(),
            adapter_profile: WAVESHARE_AUTO_DIRECTION_PROFILE.into(),
        }
    }

    #[test]
    fn duplicate_networks_rejected_before_open() {
        let bip = sample_bip();
        let mut mstp = sample_mstp();
        mstp.network = bip.network;
        // start is async and would open devices; validate path is sync via helper.
        assert!(matches!(
            validate_distinct_networks(bip.network, mstp.network),
            Err(AdapterError::InvalidNetworks(_, _))
        ));
    }

    #[test]
    fn zero_udp_port_rejected_by_bip_validate() {
        let bip = sample_bip();
        assert!(build_bip_transport(&bip).is_err());
    }
}
