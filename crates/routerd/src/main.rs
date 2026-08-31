mod system;
mod web;

use std::{env, path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use router_core::RouterConfig;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "routerd=info,tower_http=info".into()),
        )
        .init();

    let config_path = config_path_from_args()?;
    let mut config = if config_path.exists() {
        RouterConfig::from_path(&config_path)
            .with_context(|| format!("loading {}", config_path.display()))?
    } else {
        warn!(path = %config_path.display(), "configuration missing; using fail-closed defaults");
        RouterConfig::default()
    };

    if let Ok(bind) = env::var("DBR_BIND") {
        config.management.bind = bind;
    }
    if let Ok(web_root) = env::var("DBR_WEB_ROOT") {
        config.management.web_root = web_root;
    }
    config.validate()?;

    let bind = config.management.bind.clone();
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .with_context(|| format!("binding management listener {bind}"))?;
    let state = web::AppState::new(Arc::new(config));
    let app = web::app(state);

    info!(bind = %bind, "management plane listening; BACnet forwarding is disabled");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("management server exited")
}

fn config_path_from_args() -> Result<PathBuf> {
    let mut args = env::args().skip(1);
    let mut path = env::var_os("DBR_CONFIG")
        .map_or_else(|| PathBuf::from("config/router.toml"), PathBuf::from);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" => {
                path = PathBuf::from(args.next().context("--config requires a path")?);
            }
            "--help" | "-h" => {
                println!("diy-bacnet-router [--config PATH]");
                std::process::exit(0);
            }
            _ => anyhow::bail!("unknown argument: {arg}"),
        }
    }
    Ok(path)
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
