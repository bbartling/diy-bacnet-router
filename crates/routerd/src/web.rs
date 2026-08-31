use std::{
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use router_core::{Counters, RouterConfig, RouterMetrics, RuntimeSnapshot, RuntimeState};
use serde::Serialize;
use serde_json::{json, Value};
use tokio::time::{interval, Duration};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use crate::system::{SystemMetrics, SystemSampler};

#[derive(Clone)]
pub struct AppState {
    config: Arc<RouterConfig>,
    counters: Arc<Counters>,
    runtime: Arc<RuntimeState>,
    system: Arc<Mutex<SystemSampler>>,
}

#[derive(Debug, Clone, Serialize)]
struct MetricsEnvelope {
    schema_version: u8,
    timestamp_unix_ms: u128,
    router: RouterMetrics,
    runtime: RuntimeSnapshot,
    system: SystemMetrics,
}

impl AppState {
    pub fn new(config: Arc<RouterConfig>) -> Self {
        Self {
            config,
            counters: Arc::new(Counters::default()),
            runtime: Arc::new(RuntimeState::default()),
            system: Arc::new(Mutex::new(SystemSampler::default())),
        }
    }

    fn metrics(&self) -> MetricsEnvelope {
        let system = self
            .system
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .sample();
        MetricsEnvelope {
            schema_version: 1,
            timestamp_unix_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            router: self.counters.snapshot(),
            runtime: self.runtime.snapshot(),
            system,
        }
    }
}

pub fn app(state: AppState) -> Router {
    let web_root = state.config.management.web_root.clone();
    let index = format!("{web_root}/index.html");
    Router::new()
        .route("/healthz", get(health))
        .route("/api/v1/status", get(status))
        .route("/api/v1/capabilities", get(capabilities))
        .route("/api/v1/config/effective", get(effective_config))
        .route("/api/v1/metrics/snapshot", get(metrics_snapshot))
        .route("/api/v1/openapi.json", get(openapi))
        .route("/api/v1/ws/metrics", get(metrics_ws))
        .route("/metrics", get(prometheus))
        .fallback_service(
            ServeDir::new(web_root)
                .append_index_html_on_directories(true)
                .not_found_service(ServeFile::new(index)),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "status": "ok",
        "management_plane": "operational",
        "data_plane": state.runtime.snapshot().data_plane,
        "ready_to_route": false,
    }))
}

async fn status(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "name": state.config.identity.name,
        "location": state.config.identity.location,
        "version": env!("CARGO_PKG_VERSION"),
        "git_sha": option_env!("DBR_GIT_SHA").unwrap_or("development"),
        "rusty_bacnet_rev": option_env!("RUSTY_BACNET_REV").unwrap_or("not-integrated"),
        "runtime": state.runtime.snapshot(),
    }))
}

async fn capabilities() -> Json<Value> {
    Json(json!({ "capabilities": RuntimeState::capabilities() }))
}

async fn effective_config(State(state): State<AppState>) -> Json<RouterConfig> {
    Json((*state.config).clone())
}

async fn metrics_snapshot(State(state): State<AppState>) -> Json<MetricsEnvelope> {
    Json(state.metrics())
}

async fn openapi() -> Response {
    (
        [(header::CONTENT_TYPE, "application/json")],
        include_str!("../../../openapi/openapi.json"),
    )
        .into_response()
}

async fn metrics_ws(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| metrics_socket(socket, state))
}

async fn metrics_socket(mut socket: WebSocket, state: AppState) {
    let mut ticker = interval(Duration::from_millis(
        state.config.management.metrics_interval_ms,
    ));
    loop {
        ticker.tick().await;
        let Ok(payload) = serde_json::to_string(&state.metrics()) else {
            break;
        };
        if socket.send(Message::Text(payload.into())).await.is_err() {
            break;
        }
    }
}

async fn prometheus(State(state): State<AppState>) -> Response {
    let snapshot = state.metrics();
    let mut body = String::new();
    let counters = [
        ("dbr_bip_rx_packets_total", snapshot.router.bip_rx_packets),
        ("dbr_bip_tx_packets_total", snapshot.router.bip_tx_packets),
        ("dbr_mstp_rx_packets_total", snapshot.router.mstp_rx_packets),
        ("dbr_mstp_tx_packets_total", snapshot.router.mstp_tx_packets),
        (
            "dbr_forwarded_bip_to_mstp_total",
            snapshot.router.forwarded_bip_to_mstp,
        ),
        (
            "dbr_forwarded_mstp_to_bip_total",
            snapshot.router.forwarded_mstp_to_bip,
        ),
        ("dbr_dropped_packets_total", snapshot.router.dropped_packets),
        ("dbr_invalid_frames_total", snapshot.router.invalid_frames),
        (
            "dbr_mstp_header_crc_errors_total",
            snapshot.router.header_crc_errors,
        ),
        (
            "dbr_mstp_data_crc_errors_total",
            snapshot.router.data_crc_errors,
        ),
        ("dbr_apdu_timeouts_total", snapshot.router.apdu_timeouts),
        ("dbr_mstp_tx_tokens_total", snapshot.router.tx_tokens),
        ("dbr_mstp_rx_tokens_total", snapshot.router.rx_tokens),
        (
            "dbr_mstp_tx_poll_for_master_total",
            snapshot.router.tx_poll_for_master,
        ),
        (
            "dbr_mstp_rx_poll_for_master_total",
            snapshot.router.rx_poll_for_master,
        ),
    ];
    for (name, value) in counters {
        body.push_str(&format!("# TYPE {name} counter\n{name} {value}\n"));
    }
    body.push_str(&format!(
        "# TYPE dbr_system_cpu_percent gauge\ndbr_system_cpu_percent {:.3}\n",
        snapshot.system.cpu_percent
    ));
    body.push_str(&format!(
        "# TYPE dbr_system_memory_available_bytes gauge\ndbr_system_memory_available_bytes {}\n",
        snapshot.system.memory_available_bytes
    ));

    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

#[cfg(test)]
mod tests {
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn health_is_honest_about_router_readiness() {
        let response = app(AppState::new(Arc::new(RouterConfig::default())))
            .oneshot(
                axum::http::Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let body: Value = serde_json::from_slice(&bytes).expect("JSON");
        assert_eq!(body["ready_to_route"], false);
    }

    #[tokio::test]
    async fn openapi_contract_is_served() {
        let response = app(AppState::new(Arc::new(RouterConfig::default())))
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/openapi.json")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
    }
}
