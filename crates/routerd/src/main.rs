mod system;
mod web;

use std::{
    env,
    net::Ipv4Addr,
    path::PathBuf,
    process,
    sync::{atomic::Ordering, Arc},
    time::Duration,
};

use anyhow::{Context, Result};
use router_core::RouterConfig;
use rusty_bacnet_adapter::{
    encode_bip_mac, open_appliance_serial, ApplianceRouterSession, BipQualifySession,
    BipTransportParams, MstpQualifySession, MstpTransportParams,
};
use tokio::sync::oneshot;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "routerd=info,tower_http=info".into()),
        )
        .init();

    let args = CliArgs::parse()?;
    let mut config = if args.config_path.exists() {
        RouterConfig::from_path(&args.config_path)
            .with_context(|| format!("loading {}", args.config_path.display()))?
    } else if args.check_config {
        anyhow::bail!(
            "configuration file not found: {}",
            args.config_path.display()
        );
    } else {
        warn!(path = %args.config_path.display(), "configuration missing; using fail-closed defaults");
        RouterConfig::default()
    };

    if let Ok(bind) = env::var("DBR_BIND") {
        config.management.bind = bind;
    }
    if let Ok(web_root) = env::var("DBR_WEB_ROOT") {
        config.management.web_root = web_root;
    }
    config.validate()?;

    if args.check_config {
        println!(
            "configuration OK: {} (forwarding disabled, ready_to_route=false)",
            args.config_path.display()
        );
        return Ok(());
    }

    if args.bip_qualify_peer {
        return run_bip_qualify_peer(&config, args.probe_count, args.peer_target).await;
    }

    let bind = config.management.bind.clone();
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .with_context(|| format!("binding management listener {bind}"))?;
    let state = web::AppState::new(Arc::new(config.clone()));
    let app = web::app(state.clone());

    let qualify_handle = if args.bip_qualify {
        Some(
            spawn_bip_qualify(
                state.clone(),
                config.clone(),
                args.qualify_secs,
                args.send_unicast,
                args.send_broadcast,
            )
            .await?,
        )
    } else if args.mstp_qualify {
        Some(
            spawn_mstp_qualify(
                state.clone(),
                config.clone(),
                args.qualify_secs,
                args.mstp_report.clone(),
            )
            .await?,
        )
    } else if args.route_enable {
        Some(spawn_route_session(state.clone(), config.clone(), args.qualify_secs).await?)
    } else {
        None
    };

    if args.route_enable {
        info!(
            bind = %bind,
            "management plane listening; opt-in --route-enable session active (G7/G8 evidence still open)"
        );
    } else {
        info!(bind = %bind, "management plane listening; BACnet forwarding is disabled");
    }
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("management server exited")?;

    if let Some((stop_tx, join)) = qualify_handle {
        let _ = stop_tx.send(());
        match join.await {
            Ok(Ok(())) => {}
            Ok(Err(error)) => return Err(error),
            Err(error) => anyhow::bail!("B/IP qualify task join failed: {error}"),
        }
    }
    Ok(())
}

async fn spawn_bip_qualify(
    state: web::AppState,
    config: RouterConfig,
    qualify_secs: u64,
    send_unicast: Option<(Ipv4Addr, u16, u32)>,
    send_broadcast: Option<u32>,
) -> Result<(oneshot::Sender<()>, tokio::task::JoinHandle<Result<()>>)> {
    let params = bip_params_from_config(&config)?;
    let mut session = BipQualifySession::start(&params)
        .await
        .context("starting B/IP qualify session")?;
    state.mark_bip_qualify_active();
    let counters = state.counters();
    let qualify_counters = session.counters();
    let (stop_tx, stop_rx) = oneshot::channel();
    let join = tokio::spawn(async move {
        let mirror = tokio::spawn({
            let qualify_counters = Arc::clone(&qualify_counters);
            let counters = Arc::clone(&counters);
            async move {
                let mut ticker = tokio::time::interval(Duration::from_millis(200));
                loop {
                    ticker.tick().await;
                    let (rx, tx) = qualify_counters.snapshot();
                    counters.bip_rx_packets.store(rx, Ordering::Relaxed);
                    counters.bip_tx_packets.store(tx, Ordering::Relaxed);
                    counters.forwarded_bip_to_mstp.store(0, Ordering::Relaxed);
                    counters.forwarded_mstp_to_bip.store(0, Ordering::Relaxed);
                }
            }
        });

        let tx = session.tx_handle();
        let delay_ms: u64 = env::var("DBR_QUALIFY_TX_DELAY_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1_500);
        let tx_task = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            if let Some((ip, port, count)) = send_unicast {
                let mac = encode_bip_mac(ip, port);
                tx.send_unicast_burst(&mac, count)
                    .await
                    .context("qualify-send-unicast")?;
            }
            if let Some(count) = send_broadcast {
                tx.send_broadcast_burst(count)
                    .await
                    .context("qualify-send-broadcast")?;
            }
            Ok::<(), anyhow::Error>(())
        });

        session
            .run_receive_loop(stop_rx, Duration::from_secs(qualify_secs))
            .await;
        let tx_res = tx_task.await.context("qualify TX task join")?;
        mirror.abort();

        let stop_res = session.stop().await;
        let reason = match (&tx_res, &stop_res) {
            (Ok(()), Ok(())) => {
                "B/IP qualify session ended; bip_link cleared; forwarding remains disabled"
                    .to_owned()
            }
            (Err(error), _) => format!("B/IP qualify TX failed: {error}"),
            (_, Err(error)) => format!("B/IP qualify stop failed: {error}"),
        };
        state.mark_bip_qualify_inactive(&reason);

        let (rx, txc) = qualify_counters.snapshot();
        counters.bip_rx_packets.store(rx, Ordering::Relaxed);
        counters.bip_tx_packets.store(txc, Ordering::Relaxed);

        tx_res?;
        stop_res.context("stopping B/IP qualify session")?;
        info!(rx, tx = txc, "B/IP qualify session stopped");
        Ok(())
    });
    Ok((stop_tx, join))
}

async fn spawn_mstp_qualify(
    state: web::AppState,
    config: RouterConfig,
    qualify_secs: u64,
    report_path: Option<PathBuf>,
) -> Result<(oneshot::Sender<()>, tokio::task::JoinHandle<Result<()>>)> {
    let params = MstpTransportParams {
        this_station: config.mstp.mac,
        max_master: config.mstp.max_master,
        max_info_frames: config.mstp.max_info_frames,
        baud_rate: config.mstp.baud,
        network: config.mstp.network,
        serial_path: config.mstp.serial.clone(),
        adapter_profile: config.mstp.adapter_profile.clone(),
    };
    let serial = open_appliance_serial(&params).context("opening MS/TP serial for qualify")?;
    let mut session = MstpQualifySession::start(serial, &params)
        .await
        .context("starting MS/TP qualify session")?;
    state.mark_mstp_qualify_active();
    let counters = state.counters();
    let qualify_counters = session.counters();
    let (stop_tx, stop_rx) = oneshot::channel();
    let join = tokio::spawn(async move {
        let mirror = tokio::spawn({
            let qualify_counters = Arc::clone(&qualify_counters);
            let counters = Arc::clone(&counters);
            async move {
                let mut ticker = tokio::time::interval(Duration::from_millis(200));
                loop {
                    ticker.tick().await;
                    let (events, poll, next, _token_pfm, _samples) =
                        qualify_counters.snapshot_tuple();
                    // Map only observed MasterNode-derived fields; do not invent CRC totals.
                    counters.event_count.store(events, Ordering::Relaxed);
                    counters.forwarded_bip_to_mstp.store(0, Ordering::Relaxed);
                    counters.forwarded_mstp_to_bip.store(0, Ordering::Relaxed);
                    let _ = (poll, next);
                }
            }
        });
        session
            .run_until(stop_rx, Duration::from_secs(qualify_secs))
            .await;
        mirror.abort();
        let stop_res = session.stop().await;
        let reason = match &stop_res {
            Ok(()) => "MS/TP qualify session ended; mstp_link cleared; forwarding remains disabled"
                .to_owned(),
            Err(error) => format!("MS/TP qualify stop failed: {error}"),
        };
        state.mark_mstp_qualify_inactive(&reason);
        if let Some(path) = report_path {
            session
                .write_report(&path, "mstp-qualify")
                .context("writing mstp qualify report")?;
        }
        stop_res.context("stopping MS/TP qualify session")?;
        info!("MS/TP qualify session stopped");
        Ok(())
    });
    Ok((stop_tx, join))
}

async fn run_bip_qualify_peer(
    config: &RouterConfig,
    probe_count: u32,
    peer_target: Option<(Ipv4Addr, u16)>,
) -> Result<()> {
    let params = bip_params_from_config(config)?;
    let mut session = BipQualifySession::start(&params)
        .await
        .context("starting B/IP qualify peer")?;
    for i in 0..probe_count {
        if let Some((ip, port)) = peer_target {
            let mac = encode_bip_mac(ip, port);
            session
                .send_unicast_probe(&mac)
                .await
                .with_context(|| format!("unicast probe {i}"))?;
        } else {
            session
                .send_broadcast_probe()
                .await
                .with_context(|| format!("broadcast probe {i}"))?;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let (rx, tx) = session.counters().snapshot();
    info!(rx, tx, "B/IP qualify peer finished");
    session.stop().await.context("stopping B/IP qualify peer")?;
    Ok(())
}

async fn spawn_route_session(
    state: web::AppState,
    config: RouterConfig,
    qualify_secs: u64,
) -> Result<(oneshot::Sender<()>, tokio::task::JoinHandle<Result<()>>)> {
    let bip = bip_params_from_config(&config)?;
    let mstp = MstpTransportParams {
        this_station: config.mstp.mac,
        max_master: config.mstp.max_master,
        max_info_frames: config.mstp.max_info_frames,
        baud_rate: config.mstp.baud,
        network: config.mstp.network,
        serial_path: config.mstp.serial.clone(),
        adapter_profile: config.mstp.adapter_profile.clone(),
    };
    let session = ApplianceRouterSession::start(&bip, &mstp)
        .await
        .context("starting opt-in appliance router session")?;
    state.mark_routing_active();
    let (stop_tx, stop_rx) = oneshot::channel();
    let join = tokio::spawn(async move {
        let _ = tokio::time::timeout(Duration::from_secs(qualify_secs), stop_rx).await;
        let stop_res = session.stop().await;
        let reason = match &stop_res {
            Ok(()) => {
                "opt-in route session ended; ports closed; ready_to_route product claim remains false"
                    .to_owned()
            }
            Err(error) => format!("opt-in route session stop failed: {error}"),
        };
        state.mark_routing_inactive(&reason);
        stop_res.context("stopping opt-in appliance router session")?;
        info!("opt-in appliance router session stopped");
        Ok(())
    });
    Ok((stop_tx, join))
}

fn bip_params_from_config(config: &RouterConfig) -> Result<BipTransportParams> {
    let interface_addr: Ipv4Addr = config
        .bacnet_ip
        .bind_address
        .parse()
        .context("bacnet_ip.bind_address")?;
    let broadcast_address: Ipv4Addr = config
        .bacnet_ip
        .broadcast_address
        .parse()
        .context("bacnet_ip.broadcast_address")?;
    Ok(BipTransportParams {
        interface_addr,
        udp_port: config.bacnet_ip.udp_port,
        broadcast_address,
        network: config.bacnet_ip.network,
        interface_name: config.bacnet_ip.interface.clone(),
    })
}

#[derive(Debug)]
struct CliArgs {
    config_path: PathBuf,
    check_config: bool,
    bip_qualify: bool,
    bip_qualify_peer: bool,
    mstp_qualify: bool,
    route_enable: bool,
    qualify_secs: u64,
    probe_count: u32,
    peer_target: Option<(Ipv4Addr, u16)>,
    send_unicast: Option<(Ipv4Addr, u16, u32)>,
    send_broadcast: Option<u32>,
    mstp_report: Option<PathBuf>,
}

impl CliArgs {
    fn parse() -> Result<Self> {
        let mut args = env::args().skip(1);
        let mut path = env::var_os("DBR_CONFIG")
            .map_or_else(|| PathBuf::from("config/router.toml"), PathBuf::from);
        let mut check_config = false;
        let mut bip_qualify = false;
        let mut bip_qualify_peer = false;
        let mut mstp_qualify = false;
        let mut route_enable = false;
        let mut qualify_secs = 120_u64;
        let mut probe_count = 5_u32;
        let mut peer_target = None;
        let mut send_unicast = None;
        let mut send_broadcast = None;
        let mut mstp_report = None;
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--config" => {
                    path = PathBuf::from(args.next().context("--config requires a path")?);
                }
                "--check-config" => {
                    check_config = true;
                }
                "--bip-qualify" => {
                    bip_qualify = true;
                }
                "--bip-qualify-peer" => {
                    bip_qualify_peer = true;
                }
                "--mstp-qualify" => {
                    mstp_qualify = true;
                }
                "--route-enable" => {
                    route_enable = true;
                }
                "--mstp-report" => {
                    mstp_report = Some(PathBuf::from(
                        args.next().context("--mstp-report requires a path")?,
                    ));
                }
                "--qualify-secs" => {
                    qualify_secs = args
                        .next()
                        .context("--qualify-secs requires a value")?
                        .parse()
                        .context("--qualify-secs")?;
                }
                "--probe-count" => {
                    probe_count = args
                        .next()
                        .context("--probe-count requires a value")?
                        .parse()
                        .context("--probe-count")?;
                }
                "--peer-target" => {
                    let value = args.next().context("--peer-target requires IP:PORT")?;
                    let (ip, port) = value
                        .rsplit_once(':')
                        .context("--peer-target must be IP:PORT")?;
                    peer_target = Some((
                        ip.parse().context("peer IP")?,
                        port.parse().context("peer port")?,
                    ));
                }
                "--qualify-send-unicast" => {
                    let value = args
                        .next()
                        .context("--qualify-send-unicast requires IP:PORT:N")?;
                    let parts: Vec<&str> = value.rsplitn(3, ':').collect();
                    if parts.len() != 3 {
                        anyhow::bail!("--qualify-send-unicast must be IP:PORT:N");
                    }
                    let count: u32 = parts[0].parse().context("unicast count")?;
                    let port: u16 = parts[1].parse().context("unicast port")?;
                    let ip: Ipv4Addr = parts[2].parse().context("unicast IP")?;
                    send_unicast = Some((ip, port, count));
                }
                "--qualify-send-broadcast" => {
                    let value = args.next().context("--qualify-send-broadcast requires N")?;
                    send_broadcast = Some(value.parse().context("--qualify-send-broadcast")?);
                }
                "--help" | "-h" => {
                    println!(
                        "diy-bacnet-router [options]\n\n\
  --check-config                 Validate configuration, then exit without binding\n\
  --config PATH                  Configuration file (default: config/router.toml or DBR_CONFIG)\n\
  --bip-qualify                  Open one B/IP socket (G6); management stays up; no forwarding\n\
  --bip-qualify-peer             Peer helper: start B/IP, send probes, exit (no management UI)\n\
  --qualify-secs N               Max qualify session duration (1..=600, default 120)\n\
  --probe-count N                Peer probe count (1..=64, default 5)\n\
  --peer-target IP:PORT          Unicast probes to DUT BIP endpoint (else broadcast)\n\
  --qualify-send-unicast IP:PORT:N  DUT scheduled unicast TX during --bip-qualify\n\
  --qualify-send-broadcast N     DUT scheduled directed-broadcast TX during --bip-qualify\n\
  --mstp-qualify                 Open one MS/TP port (M2B software); no B/IP; no forwarding\n\
  --mstp-report PATH             Write atomic MS/TP qualify JSON report\n\
  --route-enable                 Opt-in BACnetRouter B/IP+MS/TP session (M3 software; G7/G8 open)"
                    );
                    process::exit(0);
                }
                _ => anyhow::bail!("unknown argument: {arg}"),
            }
        }
        if bip_qualify && bip_qualify_peer {
            anyhow::bail!("--bip-qualify and --bip-qualify-peer are mutually exclusive");
        }
        if mstp_qualify && (bip_qualify || bip_qualify_peer) {
            anyhow::bail!("--mstp-qualify cannot combine with B/IP qualify flags");
        }
        if route_enable && (bip_qualify || bip_qualify_peer || mstp_qualify) {
            anyhow::bail!("--route-enable cannot combine with qualify flags");
        }
        if mstp_report.is_some() && !mstp_qualify {
            anyhow::bail!("--mstp-report requires --mstp-qualify");
        }
        if !(1..=600).contains(&qualify_secs) {
            anyhow::bail!("--qualify-secs must be in 1..=600 (got {qualify_secs})");
        }
        if !(1..=64).contains(&probe_count) {
            anyhow::bail!("--probe-count must be in 1..=64 (got {probe_count})");
        }
        if let Some((_, _, count)) = send_unicast {
            if !(1..=64).contains(&count) {
                anyhow::bail!("--qualify-send-unicast count must be in 1..=64");
            }
            if !bip_qualify {
                anyhow::bail!("--qualify-send-unicast requires --bip-qualify");
            }
        }
        if let Some(count) = send_broadcast {
            if !(1..=64).contains(&count) {
                anyhow::bail!("--qualify-send-broadcast must be in 1..=64");
            }
            if !bip_qualify {
                anyhow::bail!("--qualify-send-broadcast requires --bip-qualify");
            }
        }
        Ok(Self {
            config_path: path,
            check_config,
            bip_qualify,
            bip_qualify_peer,
            mstp_qualify,
            route_enable,
            qualify_secs,
            probe_count,
            peer_target,
            send_unicast,
            send_broadcast,
            mstp_report,
        })
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            warn!(%error, "failed to install Ctrl+C handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => warn!(%error, "failed to install SIGTERM handler"),
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
