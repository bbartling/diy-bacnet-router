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
use rusty_bacnet_adapter::{encode_bip_mac, BipQualifySession, BipTransportParams};
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
        Some(spawn_bip_qualify(state.clone(), config.clone(), args.qualify_secs).await?)
    } else {
        None
    };

    info!(bind = %bind, "management plane listening; BACnet forwarding is disabled");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("management server exited")?;

    if let Some((stop_tx, join)) = qualify_handle {
        let _ = stop_tx.send(());
        let _ = join.await;
    }
    Ok(())
}

async fn spawn_bip_qualify(
    state: web::AppState,
    config: RouterConfig,
    qualify_secs: u64,
) -> Result<(oneshot::Sender<()>, tokio::task::JoinHandle<()>)> {
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
                    // Forwarding must remain exactly zero during qualify.
                    counters.forwarded_bip_to_mstp.store(0, Ordering::Relaxed);
                    counters.forwarded_mstp_to_bip.store(0, Ordering::Relaxed);
                }
            }
        });
        session
            .run_receive_loop(stop_rx, Duration::from_secs(qualify_secs))
            .await;
        mirror.abort();
        if let Err(error) = session.stop().await {
            warn!(%error, "B/IP qualify stop failed");
        }
        info!("B/IP qualify session stopped");
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
    qualify_secs: u64,
    probe_count: u32,
    peer_target: Option<(Ipv4Addr, u16)>,
}

impl CliArgs {
    fn parse() -> Result<Self> {
        let mut args = env::args().skip(1);
        let mut path = env::var_os("DBR_CONFIG")
            .map_or_else(|| PathBuf::from("config/router.toml"), PathBuf::from);
        let mut check_config = false;
        let mut bip_qualify = false;
        let mut bip_qualify_peer = false;
        let mut qualify_secs = 120;
        let mut probe_count = 5;
        let mut peer_target = None;
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
                "--help" | "-h" => {
                    println!(
                        "diy-bacnet-router [options]\n\n\
  --check-config              Validate configuration, then exit without binding\n\
  --config PATH               Configuration file (default: config/router.toml or DBR_CONFIG)\n\
  --bip-qualify               Open one B/IP socket (M2A); management stays up; no forwarding\n\
  --bip-qualify-peer          Peer helper: start B/IP, send probes, exit (no management UI)\n\
  --qualify-secs N            Max qualify session duration (default 120)\n\
  --probe-count N             Peer probe count (default 5)\n\
  --peer-target IP:PORT       Unicast probes to DUT BIP endpoint (else broadcast)"
                    );
                    process::exit(0);
                }
                _ => anyhow::bail!("unknown argument: {arg}"),
            }
        }
        if bip_qualify && bip_qualify_peer {
            anyhow::bail!("--bip-qualify and --bip-qualify-peer are mutually exclusive");
        }
        Ok(Self {
            config_path: path,
            check_config,
            bip_qualify,
            bip_qualify_peer,
            qualify_secs,
            probe_count,
            peer_target,
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
