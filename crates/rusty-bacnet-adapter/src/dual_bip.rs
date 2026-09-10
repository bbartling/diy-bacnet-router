//! Opt-in dual B/IP `BACnetRouter` session (CI-safe M3 deepen; no MS/TP).
//!
//! Lab unlock: `--route-bip-bip`. Ordinary boot must not call this.
//! Proves Clause 6 forwarding between two B/IP networks in netns/CI.
//! Product G7/G8 BIP↔MS/TP bench claim remains open.

use std::net::Ipv4Addr;
use std::path::Path;

use bacnet_network::router::{BACnetRouter, RouterPort};
use bacnet_transport::any::AnyTransport;
use bacnet_transport::mstp::NoSerial;
use tracing::info;

use crate::bip_qualify::assert_bind_on_interface;
use crate::local_delivery::LocalDeliveryDrain;
use crate::ports::{build_bip_transport, BipTransportParams};
use crate::validate::{validate_distinct_networks, AdapterError};

/// Two B/IP ports under one half-router (no serial).
pub struct DualBipRouterSession {
    router: BACnetRouter,
    drain: LocalDeliveryDrain,
    network_a: u16,
    network_b: u16,
}

impl DualBipRouterSession {
    pub async fn start(
        a: &BipTransportParams,
        b: &BipTransportParams,
    ) -> Result<Self, AdapterError> {
        validate_distinct_networks(a.network, b.network)?;
        if a.udp_port == b.udp_port && a.interface_addr == b.interface_addr {
            return Err(AdapterError::Validation(
                "dual B/IP ports must not share identical bind address and UDP port".into(),
            ));
        }
        assert_bind_on_interface(&a.interface_name, a.interface_addr)?;
        assert_bind_on_interface(&b.interface_name, b.interface_addr)?;
        let transport_a = build_bip_transport(a)?;
        let transport_b = build_bip_transport(b)?;
        let ports = vec![
            RouterPort {
                transport: AnyTransport::<NoSerial>::Bip(transport_a),
                network_number: a.network,
            },
            RouterPort {
                transport: AnyTransport::<NoSerial>::Bip(transport_b),
                network_number: b.network,
            },
        ];
        let (router, local_rx) = BACnetRouter::start(ports).await?;
        let drain = LocalDeliveryDrain::spawn(local_rx);
        info!(
            net_a = a.network,
            net_b = b.network,
            "dual B/IP router session started (opt-in; BIP↔MS/TP G7/G8 still open)"
        );
        Ok(Self {
            router,
            drain,
            network_a: a.network,
            network_b: b.network,
        })
    }

    pub fn network_a(&self) -> u16 {
        self.network_a
    }

    pub fn network_b(&self) -> u16 {
        self.network_b
    }

    #[must_use]
    pub fn local_delivery_count(&self) -> u64 {
        self.drain.count()
    }

    pub async fn route_table_len(&self) -> usize {
        self.router.table().lock().await.len()
    }

    pub async fn stop(mut self) -> Result<(), AdapterError> {
        self.drain.stop().await;
        self.router.stop().await;
        Ok(())
    }

    /// Atomic JSON report (temp + rename).
    pub fn write_report(&self, path: &Path, detail: &str) -> Result<(), AdapterError> {
        let body = format!(
            "{{\n  \"ready_to_route\": false,\n  \"mode\": \"dual_bip\",\n  \"network_a\": {},\n  \"network_b\": {},\n  \"bacnet_router\": true,\n  \"mstp\": false,\n  \"local_delivery_count\": {},\n  \"detail\": {:?}\n}}\n",
            self.network_a,
            self.network_b,
            self.local_delivery_count(),
            detail
        );
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, body)
            .map_err(|e| AdapterError::Validation(format!("report write: {e}")))?;
        std::fs::rename(&tmp, path)
            .map_err(|e| AdapterError::Validation(format!("report rename: {e}")))?;
        Ok(())
    }
}

/// Second B/IP side for `--route-bip-bip` from environment (lab/CI harness).
pub fn bip2_params_from_env(fallback_port: u16) -> Result<BipTransportParams, AdapterError> {
    let interface_addr: Ipv4Addr = std::env::var("DBR_BIP2_BIND")
        .unwrap_or_else(|_| "198.51.100.1".into())
        .parse()
        .map_err(|e| AdapterError::Validation(format!("DBR_BIP2_BIND: {e}")))?;
    let broadcast_address: Ipv4Addr = std::env::var("DBR_BIP2_BROADCAST")
        .unwrap_or_else(|_| "198.51.100.255".into())
        .parse()
        .map_err(|e| AdapterError::Validation(format!("DBR_BIP2_BROADCAST: {e}")))?;
    let udp_port: u16 = std::env::var("DBR_BIP2_PORT")
        .ok()
        .map(|s| s.parse())
        .transpose()
        .map_err(|e| AdapterError::Validation(format!("DBR_BIP2_PORT: {e}")))?
        .unwrap_or(fallback_port);
    let network: u16 = std::env::var("DBR_BIP2_NETWORK")
        .unwrap_or_else(|_| "2000".into())
        .parse()
        .map_err(|e| AdapterError::Validation(format!("DBR_BIP2_NETWORK: {e}")))?;
    let interface_name = std::env::var("DBR_BIP2_IFACE").unwrap_or_else(|_| "eth1".into());
    Ok(BipTransportParams {
        interface_addr,
        udp_port,
        broadcast_address,
        network,
        interface_name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bip_qualify::GOLDEN_NPDU;
    use bacnet_encoding::npdu::{encode_npdu, Npdu, NpduAddress};
    use bacnet_types::enums::NetworkPriority;
    use bacnet_types::MacAddr;
    use bytes::{Bytes, BytesMut};
    use tokio::time::{timeout, Duration};

    fn bvll_original_unicast(npdu: &[u8]) -> Vec<u8> {
        let total = 4 + npdu.len();
        let mut out = Vec::with_capacity(total);
        out.extend_from_slice(&[0x81, 0x0a]);
        out.extend_from_slice(&(total as u16).to_be_bytes());
        out.extend_from_slice(npdu);
        out
    }

    #[tokio::test]
    async fn dual_bip_localhost_start_stop() {
        let a = BipTransportParams {
            interface_addr: Ipv4Addr::LOCALHOST,
            udp_port: 47_881,
            broadcast_address: Ipv4Addr::LOCALHOST,
            network: 1_100,
            interface_name: "lo".into(),
        };
        let b = BipTransportParams {
            interface_addr: Ipv4Addr::LOCALHOST,
            udp_port: 47_882,
            broadcast_address: Ipv4Addr::LOCALHOST,
            network: 2_200,
            interface_name: "lo".into(),
        };
        let session = DualBipRouterSession::start(&a, &b).await.unwrap();
        assert_eq!(session.route_table_len().await, 2);
        session.stop().await.unwrap();
    }

    #[tokio::test]
    async fn dual_bip_rejects_identical_bind() {
        let a = BipTransportParams {
            interface_addr: Ipv4Addr::LOCALHOST,
            udp_port: 47_883,
            broadcast_address: Ipv4Addr::LOCALHOST,
            network: 1,
            interface_name: "lo".into(),
        };
        let b = BipTransportParams {
            interface_addr: Ipv4Addr::LOCALHOST,
            udp_port: 47_883,
            broadcast_address: Ipv4Addr::LOCALHOST,
            network: 2,
            interface_name: "lo".into(),
        };
        assert!(DualBipRouterSession::start(&a, &b).await.is_err());
    }

    #[tokio::test]
    async fn dual_bip_stop_rebind_same_ports() {
        let a = BipTransportParams {
            interface_addr: Ipv4Addr::LOCALHOST,
            udp_port: 47_884,
            broadcast_address: Ipv4Addr::LOCALHOST,
            network: 3_100,
            interface_name: "lo".into(),
        };
        let b = BipTransportParams {
            interface_addr: Ipv4Addr::LOCALHOST,
            udp_port: 47_885,
            broadcast_address: Ipv4Addr::LOCALHOST,
            network: 3_200,
            interface_name: "lo".into(),
        };
        let s1 = DualBipRouterSession::start(&a, &b).await.unwrap();
        s1.stop().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        let s2 = DualBipRouterSession::start(&a, &b).await.unwrap();
        assert_eq!(s2.route_table_len().await, 2);
        s2.stop().await.unwrap();
    }

    #[tokio::test]
    async fn dual_bip_survives_malformed_udp() {
        let port_a = 47_886_u16;
        let port_b = 47_887_u16;
        let a = BipTransportParams {
            interface_addr: Ipv4Addr::LOCALHOST,
            udp_port: port_a,
            broadcast_address: Ipv4Addr::LOCALHOST,
            network: 4_100,
            interface_name: "lo".into(),
        };
        let b = BipTransportParams {
            interface_addr: Ipv4Addr::LOCALHOST,
            udp_port: port_b,
            broadcast_address: Ipv4Addr::LOCALHOST,
            network: 4_200,
            interface_name: "lo".into(),
        };
        let session = DualBipRouterSession::start(&a, &b).await.unwrap();
        let sock = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let _ = sock
            .send_to(b"not-a-bvll-frame", (Ipv4Addr::LOCALHOST, port_a))
            .await;
        let _ = sock
            .send_to(
                &[0x81, 0x0a, 0x00, 0x05, 0xff],
                (Ipv4Addr::LOCALHOST, port_b),
            )
            .await;
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(session.route_table_len().await, 2);
        session.stop().await.unwrap();
    }

    #[tokio::test]
    async fn dual_bip_missing_iface_rejected_on_linux() {
        if !cfg!(target_os = "linux") {
            return;
        }
        let a = BipTransportParams {
            interface_addr: Ipv4Addr::LOCALHOST,
            udp_port: 47_890,
            broadcast_address: Ipv4Addr::LOCALHOST,
            network: 6_100,
            interface_name: "lo".into(),
        };
        let b = BipTransportParams {
            interface_addr: Ipv4Addr::new(192, 0, 2, 99),
            udp_port: 47_891,
            broadcast_address: Ipv4Addr::new(192, 0, 2, 255),
            network: 6_200,
            interface_name: "dbr-m4-missing-iface".into(),
        };
        assert!(DualBipRouterSession::start(&a, &b).await.is_err());
    }

    /// PR-A: drain prevents wedge; >256 local deliveries then forward still works.
    #[tokio::test]
    async fn dual_bip_drain_survives_local_flood_and_forwards() {
        let port_a = 47_900_u16;
        let port_b = 47_901_u16;
        let peer_b_port = 47_902_u16;
        let a = BipTransportParams {
            interface_addr: Ipv4Addr::LOCALHOST,
            udp_port: port_a,
            broadcast_address: Ipv4Addr::LOCALHOST,
            network: 7_100,
            interface_name: "lo".into(),
        };
        let b = BipTransportParams {
            interface_addr: Ipv4Addr::LOCALHOST,
            udp_port: port_b,
            broadcast_address: Ipv4Addr::LOCALHOST,
            network: 7_200,
            interface_name: "lo".into(),
        };
        let session = DualBipRouterSession::start(&a, &b).await.unwrap();

        // Local APDU (no DNET) via Original-Unicast → router local_tx (capacity 256).
        let flood = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let frame = bvll_original_unicast(&GOLDEN_NPDU);
        for _ in 0..300 {
            let _ = flood.send_to(&frame, (Ipv4Addr::LOCALHOST, port_a)).await;
        }
        // Allow drain to catch up.
        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        while tokio::time::Instant::now() < deadline && session.local_delivery_count() < 256 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(
            session.local_delivery_count() >= 256,
            "expected drain to observe >=256 local deliveries, got {}",
            session.local_delivery_count()
        );

        let peer_b = tokio::net::UdpSocket::bind(("127.0.0.1", peer_b_port))
            .await
            .unwrap();
        peer_b.set_broadcast(true).unwrap();

        let dmac = crate::bip_qualify::encode_bip_mac(Ipv4Addr::LOCALHOST, peer_b_port);
        let npdu = Npdu {
            is_network_message: false,
            expecting_reply: false,
            priority: NetworkPriority::NORMAL,
            destination: Some(NpduAddress {
                network: 7_200,
                mac_address: MacAddr::from_slice(&dmac),
            }),
            source: None,
            hop_count: 255,
            payload: Bytes::from_static(&[0xde, 0xad, 0xbe]),
            ..Npdu::default()
        };
        let mut buf = BytesMut::new();
        encode_npdu(&mut buf, &npdu).unwrap();
        let wire = bvll_original_unicast(&buf);
        let sender = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        sender
            .send_to(&wire, (Ipv4Addr::LOCALHOST, port_a))
            .await
            .unwrap();

        let mut recv_buf = [0u8; 512];
        let got = timeout(Duration::from_secs(2), peer_b.recv_from(&mut recv_buf)).await;
        assert!(
            got.is_ok(),
            "forwarded packet not observed on peer B after local flood"
        );
        let (n, _) = got.unwrap().unwrap();
        assert!(
            recv_buf[..n].windows(3).any(|w| w == [0xde, 0xad, 0xbe]),
            "forwarded APDU payload missing"
        );

        session.stop().await.unwrap();
    }
}
