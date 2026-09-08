//! Thin adapter over audited rusty-bacnet public APIs.
//!
//! Adapter **types** are integrated at the pinned SHA. Ordinary appliance startup
//! must remain fail-closed: this crate does **not** bind BACnet UDP, open a UART,
//! start transports, or enable forwarding unless an explicit isolated session
//! (tests or a later qualification harness) asks for it.

mod bip_qualify;
mod mstp_qualify;
mod ports;
mod route_session;
mod validate;

pub use bip_qualify::{
    assert_bind_on_interface, encode_bip_mac, BipQualifyCounters, BipQualifySession, BipQualifyTx,
    GOLDEN_NPDU,
};
pub use mstp_qualify::{open_appliance_serial, MstpQualifyCounters, MstpQualifySession};
pub use ports::{
    build_bip_transport, build_heterogeneous_ports, build_mstp_transport, ApplianceRouterPort,
    ApplianceSerial, ApplianceTransport, BipTransportParams, MstpTransportParams,
};
pub use route_session::ApplianceRouterSession;
pub use validate::{
    validate_bip_params, validate_distinct_networks, validate_mstp_params, validate_serial_path,
    AdapterError, SUPPORTED_BAUD, WAVESHARE_AUTO_DIRECTION_PROFILE,
    WAVESHARE_B_AUTO_DIRECTION_PROFILE,
};

use bacnet_network::router::{BACnetRouter, RouterPort};
use bacnet_transport::any::AnyTransport;
use bacnet_transport::loopback::LoopbackTransport;
use bacnet_transport::mstp::NoSerial;
use bacnet_transport::port::TransportPort;
use tokio::sync::mpsc;

/// Full 40-character rusty-bacnet commit pinned by `config/upstream-lock.toml`.
pub const UPSTREAM_REVISION: &str = "24e3439694b7d286e57e0a80cf7f1df4bd39d8ad";

/// Short display form used by management status.
pub const UPSTREAM_REVISION_SHORT: &str = "24e3439";

/// Audited repository URL.
pub const UPSTREAM_REPOSITORY: &str = "https://github.com/jscott3201/rusty-bacnet";

/// Isolated two-port loopback router session for deterministic tests.
///
/// Does not bind UDP or open serial devices.
pub struct LoopbackRouterSession {
    router: BACnetRouter,
    local_rx: mpsc::Receiver<bacnet_network::layer::ReceivedApdu>,
    peer_a: LoopbackTransport,
    peer_b: LoopbackTransport,
    network_a: u16,
    network_b: u16,
}

impl LoopbackRouterSession {
    /// Construct and start a router between two loopback networks.
    pub async fn start(network_a: u16, network_b: u16) -> Result<Self, AdapterError> {
        validate_distinct_networks(network_a, network_b)?;

        let (transport_a, peer_a) = LoopbackTransport::pair(vec![0x01], vec![0x11]);
        let (transport_b, peer_b) = LoopbackTransport::pair(vec![0x02], vec![0x22]);

        let ports = vec![
            RouterPort {
                transport: transport_a,
                network_number: network_a,
            },
            RouterPort {
                transport: transport_b,
                network_number: network_b,
            },
        ];

        let (router, local_rx) = BACnetRouter::start(ports).await?;
        Ok(Self {
            router,
            local_rx,
            peer_a,
            peer_b,
            network_a,
            network_b,
        })
    }

    pub fn network_a(&self) -> u16 {
        self.network_a
    }

    pub fn network_b(&self) -> u16 {
        self.network_b
    }

    pub fn local_rx(&mut self) -> &mut mpsc::Receiver<bacnet_network::layer::ReceivedApdu> {
        &mut self.local_rx
    }

    pub fn peer_a(&mut self) -> &mut LoopbackTransport {
        &mut self.peer_a
    }

    pub fn peer_b(&mut self) -> &mut LoopbackTransport {
        &mut self.peer_b
    }

    pub async fn route_table_len(&self) -> usize {
        self.router.table().lock().await.len()
    }

    pub async fn stop(mut self) -> Result<(), AdapterError> {
        self.router.stop().await;
        self.peer_a.stop().await?;
        self.peer_b.stop().await?;
        Ok(())
    }
}

/// Prove mixed-port construction with `AnyTransport` without starting sockets.
///
/// Uses loopback-only variants so ordinary compile/test paths stay UDP-free.
pub fn mixed_loopback_ports(
    network_a: u16,
    network_b: u16,
) -> Result<Vec<RouterPort<AnyTransport<NoSerial>>>, AdapterError> {
    validate_distinct_networks(network_a, network_b)?;
    let (transport_a, _peer_a) = LoopbackTransport::pair(vec![0x01], vec![0x11]);
    let (transport_b, _peer_b) = LoopbackTransport::pair(vec![0x02], vec![0x22]);
    Ok(vec![
        RouterPort {
            transport: AnyTransport::<NoSerial>::Loopback(transport_a),
            network_number: network_a,
        },
        RouterPort {
            transport: AnyTransport::<NoSerial>::Loopback(transport_b),
            network_number: network_b,
        },
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use bacnet_encoding::npdu::{encode_npdu, Npdu, NpduAddress};
    use bacnet_types::enums::NetworkPriority;
    use bacnet_types::MacAddr;
    use bytes::{Bytes, BytesMut};
    use tokio::time::{timeout, Duration};

    #[test]
    fn pin_is_full_sha() {
        assert_eq!(UPSTREAM_REVISION.len(), 40);
        assert!(UPSTREAM_REVISION.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn rejects_identical_or_out_of_range_networks() {
        assert!(matches!(
            validate_distinct_networks(1, 1),
            Err(AdapterError::InvalidNetworks(1, 1))
        ));
        assert!(matches!(
            validate_distinct_networks(0, 2),
            Err(AdapterError::InvalidNetworks(0, 2))
        ));
        assert!(matches!(
            validate_distinct_networks(1, 65535),
            Err(AdapterError::InvalidNetworks(1, 65535))
        ));
        assert!(validate_distinct_networks(1000, 2000).is_ok());
    }

    #[tokio::test]
    async fn loopback_router_starts_with_distinct_networks_and_stops_cleanly() {
        let session = LoopbackRouterSession::start(1000, 2000).await.unwrap();
        assert_eq!(session.route_table_len().await, 2);
        assert_eq!(session.network_a(), 1000);
        assert_eq!(session.network_b(), 2000);
        session.stop().await.unwrap();
    }

    #[tokio::test]
    async fn duplicate_networks_are_rejected_before_start() {
        match LoopbackRouterSession::start(42, 42).await {
            Err(AdapterError::InvalidNetworks(42, 42)) => {}
            Err(other) => panic!("expected InvalidNetworks, got {other:?}"),
            Ok(_) => panic!("expected InvalidNetworks, got Ok"),
        }
    }

    #[tokio::test]
    async fn any_transport_mixed_loopback_ports_start_and_stop() {
        let ports = mixed_loopback_ports(10, 20).unwrap();
        let (mut router, _local) = BACnetRouter::start(ports).await.unwrap();
        assert_eq!(router.table().lock().await.len(), 2);
        router.stop().await;
    }

    #[tokio::test]
    async fn local_final_hop_delivers_apdu_on_loopback() {
        let mut session = LoopbackRouterSession::start(100, 200).await.unwrap();

        let npdu = Npdu {
            is_network_message: false,
            expecting_reply: false,
            priority: NetworkPriority::NORMAL,
            destination: Some(NpduAddress {
                network: 100,
                mac_address: MacAddr::from_slice(&[0x01]),
            }),
            source: None,
            hop_count: 255,
            payload: Bytes::from_static(&[0x10, 0x08]),
            ..Npdu::default()
        };
        let mut buf = BytesMut::new();
        encode_npdu(&mut buf, &npdu).unwrap();

        session.peer_a().send_broadcast(&buf).await.unwrap();
        let received = timeout(Duration::from_secs(1), session.local_rx().recv())
            .await
            .expect("timed out waiting for local APDU")
            .expect("channel closed");
        assert_eq!(received.apdu.as_ref(), &[0x10, 0x08]);

        session.stop().await.unwrap();
    }

    /// M3 software evidence: loopback peer A → peer B cross-network APDU (CI-safe).
    #[tokio::test]
    async fn loopback_forwards_unicast_a_to_b() {
        let mut session = LoopbackRouterSession::start(100, 200).await.unwrap();
        let mut rx_b = session.peer_b().start().await.unwrap();
        // Drain startup I-Am-Router / net-message chatter.
        while let Ok(Some(_)) = timeout(Duration::from_millis(20), rx_b.recv()).await {}
        let npdu = Npdu {
            is_network_message: false,
            expecting_reply: false,
            priority: NetworkPriority::NORMAL,
            destination: Some(NpduAddress {
                network: 200,
                mac_address: MacAddr::from_slice(&[0x22]),
            }),
            source: None,
            hop_count: 255,
            payload: Bytes::from_static(&[0x01, 0x02, 0x03]),
            ..Npdu::default()
        };
        let mut buf = BytesMut::new();
        encode_npdu(&mut buf, &npdu).unwrap();
        session.peer_a().send_unicast(&buf, &[0x01]).await.unwrap();
        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        let mut found = false;
        while tokio::time::Instant::now() < deadline {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            match timeout(remaining, rx_b.recv()).await {
                Ok(Some(received)) if received.npdu.windows(3).any(|w| w == [0x01, 0x02, 0x03]) => {
                    found = true;
                    break;
                }
                Ok(Some(_)) => continue,
                Ok(None) => panic!("peer B closed"),
                Err(_) => break,
            }
        }
        assert!(found, "forwarded APDU payload not observed on peer B");
        session.stop().await.unwrap();
    }

    /// M3 software evidence: reverse direction B → A.
    #[tokio::test]
    async fn loopback_forwards_unicast_b_to_a() {
        let mut session = LoopbackRouterSession::start(100, 200).await.unwrap();
        let mut rx_a = session.peer_a().start().await.unwrap();
        while let Ok(Some(_)) = timeout(Duration::from_millis(20), rx_a.recv()).await {}
        let npdu = Npdu {
            is_network_message: false,
            expecting_reply: false,
            priority: NetworkPriority::NORMAL,
            destination: Some(NpduAddress {
                network: 100,
                mac_address: MacAddr::from_slice(&[0x11]),
            }),
            source: None,
            hop_count: 255,
            payload: Bytes::from_static(&[0xaa, 0xbb]),
            ..Npdu::default()
        };
        let mut buf = BytesMut::new();
        encode_npdu(&mut buf, &npdu).unwrap();
        session.peer_b().send_unicast(&buf, &[0x02]).await.unwrap();
        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        let mut found = false;
        while tokio::time::Instant::now() < deadline {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            match timeout(remaining, rx_a.recv()).await {
                Ok(Some(received)) if received.npdu.windows(2).any(|w| w == [0xaa, 0xbb]) => {
                    found = true;
                    break;
                }
                Ok(Some(_)) => continue,
                Ok(None) => panic!("peer A closed"),
                Err(_) => break,
            }
        }
        assert!(found, "forwarded APDU payload not observed on peer A");
        session.stop().await.unwrap();
    }

    /// M4 software: clean stop then immediate restart on same nets.
    #[tokio::test]
    async fn loopback_stop_and_rebind_same_networks() {
        let session = LoopbackRouterSession::start(300, 400).await.unwrap();
        session.stop().await.unwrap();
        let session2 = LoopbackRouterSession::start(300, 400).await.unwrap();
        assert_eq!(session2.route_table_len().await, 2);
        session2.stop().await.unwrap();
    }
}
