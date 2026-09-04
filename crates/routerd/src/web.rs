use std::{
    path::{Component, Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket},
        Request, State, WebSocketUpgrade,
    },
    http::{header, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{any, get},
    Json, Router,
};
use router_core::{
    BacnetIpConfig, Counters, IdentityConfig, MstpConfig, RouterConfig, RouterControlConfig,
    RouterMetrics, RuntimeSnapshot, RuntimeState,
};
use serde::Serialize;
use serde_json::{json, Value};
use tokio::sync::{watch, OwnedSemaphorePermit, Semaphore};
use tokio::time::{interval, Duration};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use crate::system::{SystemMetrics, SystemSampler};

#[derive(Clone)]
pub struct AppState {
    config: Arc<RouterConfig>,
    runtime: Arc<RuntimeState>,
    metrics_rx: watch::Receiver<MetricsEnvelope>,
    sample_ticks: Arc<AtomicU64>,
    ws_limit: Arc<Semaphore>,
}

/// Deliberately public view of effective configuration (no secrets).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PublicEffectiveConfig {
    pub identity: IdentityConfig,
    pub management: PublicManagementConfig,
    pub router: RouterControlConfig,
    pub bacnet_ip: BacnetIpConfig,
    pub mstp: MstpConfig,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PublicManagementConfig {
    pub bind: String,
    pub web_root: String,
    pub metrics_interval_ms: u64,
    pub max_ws_connections: u32,
}

impl From<&RouterConfig> for PublicEffectiveConfig {
    fn from(config: &RouterConfig) -> Self {
        Self {
            identity: config.identity.clone(),
            management: PublicManagementConfig {
                bind: config.management.bind.clone(),
                web_root: config.management.web_root.clone(),
                metrics_interval_ms: config.management.metrics_interval_ms,
                max_ws_connections: config.management.max_ws_connections,
            },
            router: config.router.clone(),
            bacnet_ip: config.bacnet_ip.clone(),
            mstp: config.mstp.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricsEnvelope {
    schema_version: u8,
    /// Monotonic publisher sequence (starts at 1 once the background sampler runs).
    sequence: u64,
    timestamp_unix_ms: u128,
    /// Age of this sample relative to publisher cadence (ms since previous tick).
    sample_interval_ms: u64,
    /// Whether BACnet data-plane counters are observed (false while adapter absent).
    bacnet_telemetry_available: bool,
    router: RouterMetrics,
    runtime: RuntimeSnapshot,
    system: SystemMetrics,
}

impl AppState {
    pub fn new(config: Arc<RouterConfig>) -> Self {
        let counters = Arc::new(Counters::default());
        let runtime = Arc::new(RuntimeState::default());
        let sample_ticks = Arc::new(AtomicU64::new(0));
        let ws_limit = Arc::new(Semaphore::new(
            config.management.max_ws_connections as usize,
        ));
        let interval_ms = config.management.metrics_interval_ms;

        let initial = MetricsEnvelope {
            schema_version: 1,
            sequence: 0,
            timestamp_unix_ms: now_ms(),
            sample_interval_ms: interval_ms,
            bacnet_telemetry_available: false,
            router: counters.snapshot(),
            runtime: runtime.snapshot(),
            system: SystemMetrics::default(),
        };
        let (tx, rx) = watch::channel(initial);

        let publisher_counters = Arc::clone(&counters);
        let publisher_runtime = Arc::clone(&runtime);
        let publisher_ticks = Arc::clone(&sample_ticks);
        let sampler = Arc::new(Mutex::new(SystemSampler::default()));

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(interval_ms));
            loop {
                ticker.tick().await;
                let system = sampler
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .sample();
                let seq = publisher_ticks.fetch_add(1, Ordering::Relaxed) + 1;
                let envelope = MetricsEnvelope {
                    schema_version: 1,
                    sequence: seq,
                    timestamp_unix_ms: now_ms(),
                    sample_interval_ms: interval_ms,
                    // Adapter not integrated: counters are scaffold zeros, not observed wire data.
                    bacnet_telemetry_available: false,
                    router: publisher_counters.snapshot(),
                    runtime: publisher_runtime.snapshot(),
                    system,
                };
                if tx.send(envelope).is_err() {
                    break;
                }
            }
        });

        Self {
            config,
            runtime,
            metrics_rx: rx,
            sample_ticks,
            ws_limit,
        }
    }

    fn metrics(&self) -> MetricsEnvelope {
        self.metrics_rx.borrow().clone()
    }

    pub fn sample_ticks(&self) -> u64 {
        self.sample_ticks.load(Ordering::Relaxed)
    }

    pub fn available_ws_permits(&self) -> usize {
        self.ws_limit.available_permits()
    }
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

pub fn app(state: AppState) -> Router {
    let web_root = PathBuf::from(&state.config.management.web_root);
    let index = web_root.join("index.html");
    let static_files = ServeDir::new(web_root)
        .append_index_html_on_directories(true)
        .fallback(ServeFile::new(index));

    Router::new()
        .route("/healthz", get(health))
        .route("/api/status", get(status))
        .route("/api/capabilities", get(capabilities))
        .route("/api/config/effective", get(effective_config))
        .route("/api/metrics/snapshot", get(metrics_snapshot))
        .route("/api/openapi.json", get(openapi))
        .route("/api/ws/metrics", get(metrics_ws))
        .route("/metrics", get(prometheus))
        .route("/api/{*rest}", any(api_not_found))
        .layer(middleware::from_fn(reject_unsafe_static_paths))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
        .fallback_service(static_files)
}

async fn reject_unsafe_static_paths(request: Request, next: Next) -> Response {
    let path = request.uri().path();
    if !path.starts_with("/api/")
        && path != "/healthz"
        && path != "/metrics"
        && !static_path_is_safe(path)
    {
        return StatusCode::BAD_REQUEST.into_response();
    }
    next.run(request).await
}

async fn api_not_found() -> Response {
    (StatusCode::NOT_FOUND, Json(json!({ "error": "not found" }))).into_response()
}

/// Reject traversal / odd paths before ServeDir; used by dedicated security tests.
pub fn static_path_is_safe(request_path: &str) -> bool {
    if request_path.contains('\\') || request_path.contains('\0') {
        return false;
    }
    let lower = request_path.to_ascii_lowercase();
    if lower.contains("%2e") || lower.contains("%2f") || lower.contains("%5c") {
        return false;
    }
    let trimmed = request_path.trim_start_matches('/');
    if trimmed.is_empty() {
        return true;
    }
    let path = Path::new(trimmed);
    for component in path.components() {
        match component {
            Component::Normal(part) => {
                let text = part.to_string_lossy();
                if text == ".." || text.contains('\0') {
                    return false;
                }
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return false,
        }
    }
    true
}

async fn health(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("DBR_VERSION"),
        "management_plane": "operational",
        "data_plane": state.runtime.snapshot().data_plane,
        "ready_to_route": false,
    }))
}

async fn status(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "name": state.config.identity.name,
        "location": state.config.identity.location,
        "version": env!("DBR_VERSION"),
        "git_sha": option_env!("DBR_GIT_SHA").unwrap_or("development"),
        "rusty_bacnet_rev": rusty_bacnet_adapter::UPSTREAM_REVISION_SHORT,
        "runtime": state.runtime.snapshot(),
    }))
}

async fn capabilities() -> Json<Value> {
    Json(json!({ "capabilities": RuntimeState::capabilities() }))
}

async fn effective_config(State(state): State<AppState>) -> Json<PublicEffectiveConfig> {
    Json(PublicEffectiveConfig::from(state.config.as_ref()))
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
    let Ok(permit) = state.ws_limit.clone().try_acquire_owned() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "error": "websocket connection limit reached" })),
        )
            .into_response();
    };
    ws.on_upgrade(move |socket| metrics_socket(socket, state, permit))
        .into_response()
}

async fn metrics_socket(mut socket: WebSocket, state: AppState, _permit: OwnedSemaphorePermit) {
    const MAX_INCOMING_BYTES: usize = 4_096;
    const WRITE_TIMEOUT: Duration = Duration::from_secs(2);

    let mut rx = state.metrics_rx.clone();
    // Initial snapshot once, then only when the shared publisher advances.
    // Ping/Pong/Close must not trigger extra snapshots.
    {
        let snapshot = rx.borrow_and_update().clone();
        let Ok(payload) = serde_json::to_string(&snapshot) else {
            return;
        };
        if tokio::time::timeout(WRITE_TIMEOUT, socket.send(Message::Text(payload.into())))
            .await
            .is_err()
        {
            return;
        }
    }

    loop {
        tokio::select! {
            changed = rx.changed() => {
                if changed.is_err() {
                    break;
                }
                let snapshot = rx.borrow_and_update().clone();
                let Ok(payload) = serde_json::to_string(&snapshot) else {
                    break;
                };
                match tokio::time::timeout(WRITE_TIMEOUT, socket.send(Message::Text(payload.into()))).await {
                    Ok(Ok(())) => {}
                    Ok(Err(_)) | Err(_) => break, // disconnect or slow reader
                }
            }
            incoming = socket.recv() => {
                match incoming {
                    None | Some(Err(_)) | Some(Ok(Message::Close(_))) => break,
                    Some(Ok(Message::Ping(payload))) => {
                        if payload.len() > MAX_INCOMING_BYTES {
                            break;
                        }
                        if tokio::time::timeout(WRITE_TIMEOUT, socket.send(Message::Pong(payload)))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    Some(Ok(Message::Text(text))) => {
                        if text.len() > MAX_INCOMING_BYTES {
                            break;
                        }
                    }
                    Some(Ok(Message::Binary(bin))) => {
                        if bin.len() > MAX_INCOMING_BYTES {
                            break;
                        }
                    }
                    Some(Ok(_)) => {}
                }
            }
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
        (
            "dbr_serial_reconnects_total",
            snapshot.router.serial_reconnects,
        ),
    ];
    for (name, value) in counters {
        body.push_str(&format!("# TYPE {name} counter\n{name} {value}\n"));
    }
    // EventCount is a resettable FSM gauge, not a lifetime counter.
    body.push_str(&format!(
        "# TYPE dbr_mstp_event_count gauge\ndbr_mstp_event_count {}\n",
        snapshot.router.event_count
    ));
    body.push_str(&format!(
        "# TYPE dbr_bacnet_telemetry_available gauge\ndbr_bacnet_telemetry_available {}\n",
        u64::from(snapshot.bacnet_telemetry_available)
    ));
    body.push_str(&format!(
        "# TYPE dbr_metrics_sequence gauge\ndbr_metrics_sequence {}\n",
        snapshot.sequence
    ));
    body.push_str(&format!(
        "# TYPE dbr_metrics_publisher_ticks_total counter\ndbr_metrics_publisher_ticks_total {}\n",
        state.sample_ticks()
    ));
    body.push_str(&format!(
        "# TYPE dbr_ws_permits_available gauge\ndbr_ws_permits_available {}\n",
        state.available_ws_permits()
    ));
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
    use std::time::Duration;

    use futures_util::StreamExt;
    use http_body_util::BodyExt;
    use tokio::net::TcpListener;
    use tower::ServiceExt;

    use super::*;

    fn test_config_with_web_root(web_root: PathBuf) -> RouterConfig {
        let mut config = RouterConfig::default();
        config.management.web_root = web_root.to_string_lossy().into_owned();
        config.management.metrics_interval_ms = 250;
        config.management.max_ws_connections = 2;
        config
    }

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
    async fn openapi_contract_lists_required_paths() {
        let response = app(AppState::new(Arc::new(RouterConfig::default())))
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/openapi.json")
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
        for path in [
            "/healthz",
            "/api/status",
            "/api/capabilities",
            "/api/config/effective",
            "/api/metrics/snapshot",
            "/api/ws/metrics",
            "/api/openapi.json",
            "/metrics",
        ] {
            assert!(
                body["paths"].get(path).is_some(),
                "OpenAPI missing path {path}"
            );
        }
    }

    #[tokio::test]
    async fn unknown_api_route_returns_json_not_spa_html() {
        let response = app(AppState::new(Arc::new(RouterConfig::default())))
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/does-not-exist")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("");
        assert!(content_type.contains("application/json"));
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let body: Value = serde_json::from_slice(&bytes).expect("JSON error body");
        assert_eq!(body["error"], "not found");
    }

    #[tokio::test]
    async fn spa_fallback_serves_index_html_for_frontend_paths() {
        let temp = std::env::temp_dir().join(format!("dbr-web-test-{}", std::process::id()));
        std::fs::create_dir_all(&temp).expect("tempdir");
        std::fs::write(temp.join("index.html"), "<!doctype html><title>DBR</title>")
            .expect("write index");

        let config = test_config_with_web_root(temp.clone());
        let response = app(AppState::new(Arc::new(config)))
            .oneshot(
                axum::http::Request::builder()
                    .uri("/overview")
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
        let html = String::from_utf8(bytes.to_vec()).expect("utf8");
        assert!(html.contains("DBR"));

        let _ = std::fs::remove_dir_all(temp);
    }

    #[tokio::test]
    async fn effective_config_returns_public_type() {
        let response = app(AppState::new(Arc::new(RouterConfig::default())))
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/config/effective")
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
        assert!(body.get("identity").is_some());
        assert!(body.get("management").is_some());
        assert_eq!(body["management"]["max_ws_connections"], 8);
    }

    #[test]
    fn static_path_rejects_traversal_attempts() {
        assert!(static_path_is_safe("/"));
        assert!(static_path_is_safe("/assets/app.js"));
        assert!(!static_path_is_safe("/../etc/passwd"));
        assert!(!static_path_is_safe("/..\\windows\\system32"));
        assert!(!static_path_is_safe("/%2e%2e/etc/passwd")); // literal encoded segment
        assert!(!static_path_is_safe("/assets/../../etc/passwd"));
    }

    #[tokio::test]
    async fn serve_dir_rejects_dotdot_path() {
        let temp = std::env::temp_dir().join(format!("dbr-web-trav-{}", std::process::id()));
        std::fs::create_dir_all(&temp).expect("tempdir");
        std::fs::write(temp.join("index.html"), "<!doctype html><title>DBR</title>")
            .expect("write index");
        // Sibling secret outside web root
        let secret = temp
            .parent()
            .unwrap()
            .join(format!("dbr-secret-{}", std::process::id()));
        std::fs::write(&secret, "SECRET").expect("secret");

        let config = test_config_with_web_root(temp.clone());
        let response = app(AppState::new(Arc::new(config)))
            .oneshot(
                axum::http::Request::builder()
                    .uri("/../dbr-secret-should-not-matter")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let body = String::from_utf8_lossy(&bytes);
        assert!(!body.contains("SECRET"));

        let _ = std::fs::remove_dir_all(temp);
        let _ = std::fs::remove_file(secret);
    }

    #[tokio::test]
    async fn websocket_limit_releases_permit_after_disconnect() {
        let mut config = RouterConfig::default();
        config.management.metrics_interval_ms = 250;
        config.management.max_ws_connections = 1;
        let state = AppState::new(Arc::new(config));
        assert_eq!(state.available_ws_permits(), 1);

        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let app = app(state.clone());
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve");
        });

        let url = format!("ws://{addr}/api/ws/metrics");
        let (mut ws, _) = tokio_tungstenite::connect_async(&url)
            .await
            .expect("first ws");
        // Limit reached
        let denied = tokio_tungstenite::connect_async(&url).await;
        assert!(
            denied.is_err(),
            "second websocket should be rejected at the connection limit"
        );
        // Close first socket and wait for permit return
        ws.close(None).await.ok();
        drop(ws);
        tokio::time::sleep(Duration::from_millis(300)).await;

        let (mut ws2, _) = tokio_tungstenite::connect_async(&url)
            .await
            .expect("ws after release");
        let msg = ws2.next().await.expect("frame").expect("ok");
        assert!(msg.is_text());
        ws2.close(None).await.ok();
    }

    #[tokio::test]
    async fn shared_publisher_serves_multiple_snapshot_readers() {
        let mut config = RouterConfig::default();
        config.management.metrics_interval_ms = 250;
        let state = AppState::new(Arc::new(config));
        // Wait for at least one publisher tick
        for _ in 0..40 {
            if state.sample_ticks() >= 1 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        assert!(state.sample_ticks() >= 1);
        let a = state.metrics();
        let b = state.metrics();
        assert_eq!(a.timestamp_unix_ms, b.timestamp_unix_ms);
        let before = state.sample_ticks();
        // Reading metrics does not sample again
        let _ = state.metrics();
        assert_eq!(state.sample_ticks(), before);
    }

    #[tokio::test]
    async fn graceful_client_disconnect_does_not_panic() {
        let mut config = RouterConfig::default();
        config.management.metrics_interval_ms = 250;
        let state = AppState::new(Arc::new(config));
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let app = app(state);
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve");
        });
        let url = format!("ws://{addr}/api/ws/metrics");
        let (mut ws, _) = tokio_tungstenite::connect_async(&url).await.expect("ws");
        let _ = ws.next().await;
        drop(ws);
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
