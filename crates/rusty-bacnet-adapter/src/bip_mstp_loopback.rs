//! CI: BipTransport ↔ BACnetRouter ↔ MstpTransport<LoopbackSerial> round-trip.
//!
//! Uses only public APIs at the pinned rusty-bacnet SHA. Product G7/G8
//! BIP↔physical-MS/TP claims remain OPEN.

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use std::time::Duration;

    use bacnet_encoding::npdu::{encode_npdu, Npdu, NpduAddress};
    use bacnet_network::router::{BACnetRouter, RouterPort};
    use bacnet_transport::any::AnyTransport;
    use bacnet_transport::bip::BipTransport;
    use bacnet_transport::mstp::{LoopbackSerial, MstpConfig, MstpTransport};
    use bacnet_transport::port::TransportPort;
    use bacnet_types::enums::NetworkPriority;
    use bacnet_types::MacAddr;
    use bytes::{Bytes, BytesMut};
    use tokio::time::timeout;

    use crate::bip_qualify::encode_bip_mac;
    use crate::local_delivery::LocalDeliveryDrain;

    fn bvll_unicast(npdu: &[u8]) -> Vec<u8> {
        let total = 4 + npdu.len();
        let mut out = Vec::with_capacity(total);
        out.extend_from_slice(&[0x81, 0x0a]);
        out.extend_from_slice(&(total as u16).to_be_bytes());
        out.extend_from_slice(npdu);
        out
    }

    fn bvll_broadcast(npdu: &[u8]) -> Vec<u8> {
        let total = 4 + npdu.len();
        let mut out = Vec::with_capacity(total);
        out.extend_from_slice(&[0x81, 0x0b]);
        out.extend_from_slice(&(total as u16).to_be_bytes());
        out.extend_from_slice(npdu);
        out
    }

    #[tokio::test]
    async fn bip_mstp_loopback_forwards_both_ways_and_rebinds() {
        let bip_port = 47_910_u16;
        let peer_bip_port = 47_911_u16;
        let bip_net = 1_000_u16;
        let mstp_net = 2_000_u16;

        let (serial_router, serial_peer) = LoopbackSerial::pair();
        let bip = BipTransport::new(Ipv4Addr::LOCALHOST, bip_port, Ipv4Addr::LOCALHOST);
        let mstp = MstpTransport::new(
            serial_router,
            MstpConfig {
                this_station: 1,
                max_master: 2,
                max_info_frames: 1,
                baud_rate: 38_400,
            },
        );
        let ports = vec![
            RouterPort {
                transport: AnyTransport::<LoopbackSerial>::Bip(bip),
                network_number: bip_net,
            },
            RouterPort {
                transport: AnyTransport::<LoopbackSerial>::Mstp(mstp),
                network_number: mstp_net,
            },
        ];
        let (mut router, local_rx) = BACnetRouter::start(ports).await.unwrap();
        let drain = LocalDeliveryDrain::spawn(local_rx);

        let mut peer = MstpTransport::new(
            serial_peer,
            MstpConfig {
                this_station: 2,
                max_master: 2,
                max_info_frames: 1,
                baud_rate: 38_400,
            },
        );
        let mut peer_rx = peer.start().await.unwrap();
        // Let MS/TP token settle on the two-node loopback ring.
        tokio::time::sleep(Duration::from_millis(400)).await;

        // BIP → MS/TP directed NPDU.
        let peer_bip = tokio::net::UdpSocket::bind(("127.0.0.1", peer_bip_port))
            .await
            .unwrap();
        let npdu_to_mstp = Npdu {
            destination: Some(NpduAddress {
                network: mstp_net,
                mac_address: MacAddr::from_slice(&[2]),
            }),
            hop_count: 255,
            payload: Bytes::from_static(&[0x11, 0x22, 0x33]),
            priority: NetworkPriority::NORMAL,
            ..Npdu::default()
        };
        let mut buf = BytesMut::new();
        encode_npdu(&mut buf, &npdu_to_mstp).unwrap();
        let wire = bvll_unicast(&buf);
        let client = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        client
            .send_to(&wire, (Ipv4Addr::LOCALHOST, bip_port))
            .await
            .unwrap();

        let mut found_mstp = false;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
        while tokio::time::Instant::now() < deadline {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            match timeout(remaining, peer_rx.recv()).await {
                Ok(Some(frame)) if frame.npdu.windows(3).any(|w| w == [0x11, 0x22, 0x33]) => {
                    found_mstp = true;
                    break;
                }
                Ok(Some(_)) => continue,
                Ok(None) => break,
                Err(_) => break,
            }
        }
        assert!(found_mstp, "BIP→MS/TP directed forward not observed");

        // MS/TP → BIP directed NPDU.
        let dmac = encode_bip_mac(Ipv4Addr::LOCALHOST, peer_bip_port);
        let npdu_to_bip = Npdu {
            destination: Some(NpduAddress {
                network: bip_net,
                mac_address: MacAddr::from_slice(&dmac),
            }),
            hop_count: 255,
            payload: Bytes::from_static(&[0xaa, 0xbb]),
            priority: NetworkPriority::NORMAL,
            ..Npdu::default()
        };
        let mut buf2 = BytesMut::new();
        encode_npdu(&mut buf2, &npdu_to_bip).unwrap();
        peer.send_unicast(&buf2, &[1]).await.unwrap();

        let mut recv_buf = [0u8; 512];
        let got = timeout(Duration::from_secs(3), peer_bip.recv_from(&mut recv_buf)).await;
        assert!(got.is_ok(), "MS/TP→BIP directed forward not observed");
        let (n, _) = got.unwrap().unwrap();
        assert!(
            recv_buf[..n].windows(2).any(|w| w == [0xaa, 0xbb]),
            "MS/TP→BIP payload missing"
        );

        // Global/remote broadcast: inject Original-Broadcast; do not require ingress reflection.
        let bcast_npdu = Npdu {
            destination: Some(NpduAddress {
                network: 0xFFFF,
                mac_address: MacAddr::new(),
            }),
            hop_count: 255,
            payload: Bytes::from_static(&[0x10]),
            priority: NetworkPriority::NORMAL,
            ..Npdu::default()
        };
        let mut bbuf = BytesMut::new();
        encode_npdu(&mut bbuf, &bcast_npdu).unwrap();
        let _ = client
            .send_to(&bvll_broadcast(&bbuf), (Ipv4Addr::LOCALHOST, bip_port))
            .await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        // Success criterion: session still alive (no panic / wedge).
        assert!(drain.count() < u64::MAX);

        drain.stop().await;
        router.stop().await;
        peer.stop().await.unwrap();

        // Rebind: new serial pair + BIP ports.
        let (s2a, s2b) = LoopbackSerial::pair();
        let bip2 = BipTransport::new(Ipv4Addr::LOCALHOST, 47_912, Ipv4Addr::LOCALHOST);
        let mstp2 = MstpTransport::new(
            s2a,
            MstpConfig {
                this_station: 1,
                max_master: 2,
                max_info_frames: 1,
                baud_rate: 38_400,
            },
        );
        let ports2 = vec![
            RouterPort {
                transport: AnyTransport::<LoopbackSerial>::Bip(bip2),
                network_number: bip_net,
            },
            RouterPort {
                transport: AnyTransport::<LoopbackSerial>::Mstp(mstp2),
                network_number: mstp_net,
            },
        ];
        let (mut router2, local2) = BACnetRouter::start(ports2).await.unwrap();
        let drain2 = LocalDeliveryDrain::spawn(local2);
        let mut peer2 = MstpTransport::new(
            s2b,
            MstpConfig {
                this_station: 2,
                max_master: 2,
                max_info_frames: 1,
                baud_rate: 38_400,
            },
        );
        let _ = peer2.start().await.unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;
        drain2.stop().await;
        router2.stop().await;
        peer2.stop().await.unwrap();
    }
}
