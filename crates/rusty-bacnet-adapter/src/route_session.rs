//! Opt-in B/IP + MS/TP `BACnetRouter` session (M3 software path).
//!
//! Ordinary appliance boot must not call this. Lab unlock is `--route-enable`.
//! Product routing gates (G7/G8) remain open until isolated bench evidence exists.
//! Does not enable BBMD/FDR. Does not synthesize forward counters.

use std::path::Path;

use bacnet_network::router::BACnetRouter;
use tracing::info;

use crate::bip_qualify::assert_bind_on_interface;
use crate::local_delivery::LocalDeliveryDrain;
use crate::mstp_qualify::open_appliance_serial;
use crate::ports::{
    build_bip_transport, build_heterogeneous_ports, build_mstp_transport, BipTransportParams,
    MstpTransportParams,
};
use crate::validate::{validate_distinct_networks, AdapterError};
use crate::UPSTREAM_REVISION;

/// Live heterogeneous router session (opens one B/IP socket + one serial-by-id).
pub struct ApplianceRouterSession {
    router: BACnetRouter,
    drain: LocalDeliveryDrain,
    bip_network: u16,
    mstp_network: u16,
    bip: BipTransportParams,
    mstp: MstpTransportParams,
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
        let drain = LocalDeliveryDrain::spawn(local_rx);
        info!(
            bip_net = bip.network,
            mstp_net = mstp.network,
            "appliance router session started (opt-in; G7/G8 evidence still open)"
        );
        Ok(Self {
            router,
            drain,
            bip_network: bip.network,
            mstp_network: mstp.network,
            bip: bip.clone(),
            mstp: mstp.clone(),
        })
    }

    #[must_use]
    pub fn local_delivery_count(&self) -> u64 {
        self.drain.count()
    }

    pub async fn route_table_len(&self) -> usize {
        self.router.table().lock().await.len()
    }

    /// Stop the session, then write an atomic acceptance report when `path` is set.
    pub async fn stop_with_optional_report(
        mut self,
        path: Option<&Path>,
        duration_ms: u64,
        app_version: &str,
    ) -> Result<(), AdapterError> {
        let snap = RouteReportSnapshot {
            verdict: "ok",
            failure_reason: "",
            stop_result: "ok",
            duration_ms,
            app_version,
            bip: &self.bip,
            mstp: &self.mstp,
            bip_network: self.bip_network,
            mstp_network: self.mstp_network,
            local_delivery_count: self.local_delivery_count(),
            table_len: self.route_table_len().await,
        };

        self.drain.stop().await;
        self.router.stop().await;

        if let Some(path) = path {
            write_route_report(path, snap)?;
        }
        Ok(())
    }

    /// Atomic JSON acceptance report (temp + rename). Never invents forward totals.
    pub async fn write_report(
        &self,
        path: &Path,
        verdict: &str,
        failure_reason: &str,
        stop_result: &str,
        duration_ms: u64,
        app_version: &str,
    ) -> Result<(), AdapterError> {
        write_route_report(
            path,
            RouteReportSnapshot {
                verdict,
                failure_reason,
                stop_result,
                duration_ms,
                app_version,
                bip: &self.bip,
                mstp: &self.mstp,
                bip_network: self.bip_network,
                mstp_network: self.mstp_network,
                local_delivery_count: self.local_delivery_count(),
                table_len: self.route_table_len().await,
            },
        )
    }

    pub async fn stop(mut self) -> Result<(), AdapterError> {
        self.drain.stop().await;
        self.router.stop().await;
        Ok(())
    }
}

struct RouteReportSnapshot<'a> {
    verdict: &'a str,
    failure_reason: &'a str,
    stop_result: &'a str,
    duration_ms: u64,
    app_version: &'a str,
    bip: &'a BipTransportParams,
    mstp: &'a MstpTransportParams,
    bip_network: u16,
    mstp_network: u16,
    local_delivery_count: u64,
    table_len: usize,
}

fn write_route_report(path: &Path, s: RouteReportSnapshot<'_>) -> Result<(), AdapterError> {
    let body = format!(
        "{{\n\
           \"verdict\": {:?},\n\
           \"failure_reason\": {:?},\n\
           \"ready_to_route\": false,\n\
           \"mode\": \"bip_mstp_route_enable\",\n\
           \"product_g7_g8_bip_mstp\": \"OPEN\",\n\
           \"bacnet_router\": true,\n\
           \"mstp\": true,\n\
           \"upstream_sha\": {UPSTREAM_REVISION:?},\n\
           \"app_version\": {:?},\n\
           \"duration_ms\": {},\n\
           \"bip\": {{\n\
             \"interface\": {:?},\n\
             \"bind\": {:?},\n\
             \"broadcast\": {:?},\n\
             \"udp_port\": {},\n\
             \"network\": {}\n\
           }},\n\
           \"mstp_port\": {{\n\
             \"serial\": {:?},\n\
             \"adapter_profile\": {:?},\n\
             \"baud\": {},\n\
             \"mac\": {},\n\
             \"max_master\": {},\n\
             \"max_info_frames\": {},\n\
             \"network\": {}\n\
           }},\n\
           \"local_delivery_count\": {},\n\
           \"route_table_len\": {},\n\
           \"stop_result\": {:?},\n\
           \"telemetry_limitations\": \"bacnet counters unavailable at pin; local_delivery_count only; G7/G8 OPEN\"\n\
         }}\n",
        s.verdict,
        s.failure_reason,
        s.app_version,
        s.duration_ms,
        s.bip.interface_name,
        s.bip.interface_addr.to_string(),
        s.bip.broadcast_address.to_string(),
        s.bip.udp_port,
        s.bip_network,
        s.mstp.serial_path,
        s.mstp.adapter_profile,
        s.mstp.baud_rate,
        s.mstp.this_station,
        s.mstp.max_master,
        s.mstp.max_info_frames,
        s.mstp_network,
        s.local_delivery_count,
        s.table_len,
        s.stop_result,
    );
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, body)
        .map_err(|e| AdapterError::Validation(format!("report write: {e}")))?;
    std::fs::rename(&tmp, path)
        .map_err(|e| AdapterError::Validation(format!("report rename: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::WAVESHARE_AUTO_DIRECTION_PROFILE;
    use std::net::Ipv4Addr;

    fn sample_bip() -> BipTransportParams {
        BipTransportParams {
            interface_addr: Ipv4Addr::new(127, 0, 0, 1),
            udp_port: 0,
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
