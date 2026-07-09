//! OpenAPI path documentation (utoipa path stubs — handlers live in `routes/`).

#![allow(dead_code)]

use crate::models::*;

/// Service index and quick links.
#[utoipa::path(
    get,
    path = "/",
    tag = "Root",
    responses((status = 200, description = "Service metadata"))
)]
fn doc_root() {}

/// Liveness probe.
#[utoipa::path(
    get,
    path = "/health",
    tag = "Root",
    responses((status = 200, description = "OK"))
)]
fn doc_health() {}

/// Open-FDD health shape (`service`, `version`, `git_sha`, `poll_running`).
#[utoipa::path(
    get,
    path = "/api/health",
    tag = "Open-FDD compat",
    responses((status = 200, description = "Sidecar health"))
)]
fn doc_api_health() {}

/// Configured field-device point catalog.
#[utoipa::path(
    get,
    path = "/bacnet/points",
    tag = "BACnet",
    security(("BearerAuth" = [])),
    responses((status = 200, description = "Point catalog"))
)]
fn doc_bacnet_points() {}

/// ReadProperty on a field device.
#[utoipa::path(
    post,
    path = "/bacnet/read",
    tag = "BACnet",
    request_body = BacnetReadRequest,
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Property value"),
        (status = 502, description = "BACnet error")
    )
)]
fn doc_bacnet_read() {}

/// WriteProperty with optional dry-run when `approved` is false.
#[utoipa::path(
    post,
    path = "/bacnet/write",
    tag = "BACnet",
    request_body = BacnetWriteRequest,
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Write result"),
        (status = 400, description = "Validation error"),
        (status = 502, description = "BACnet error")
    )
)]
fn doc_bacnet_write() {}

/// Validate and encode a write without touching the BACnet bus.
#[utoipa::path(
    post,
    path = "/bacnet/write-dry-run",
    tag = "BACnet",
    request_body = BacnetWriteRequest,
    security(("BearerAuth" = [])),
    responses((status = 200, description = "Dry-run result"))
)]
fn doc_bacnet_write_dry_run() {}

/// ReadPropertyMultiple batch read.
#[utoipa::path(
    post,
    path = "/bacnet/rpm",
    tag = "BACnet",
    request_body = BacnetRpmRequest,
    security(("BearerAuth" = [])),
    responses((status = 200, description = "RPM results"))
)]
fn doc_bacnet_rpm() {}

/// Who-Is device discovery. Send `{}` to scan all instances (0–4194303); set `low`/`high` to narrow.
#[utoipa::path(
    post,
    path = "/bacnet/whois",
    tag = "BACnet",
    request_body = BacnetWhoisRequest,
    security(("BearerAuth" = [])),
    responses((status = 200, description = "Discovered devices"))
)]
fn doc_bacnet_whois() {}

/// Who-Is router-to-network — discovers MS/TP routers on the BACnet/IP segment (no request body).
#[utoipa::path(
    post,
    path = "/bacnet/whois-router",
    tag = "BACnet",
    security(("BearerAuth" = [])),
    responses((status = 200, description = "Router list"))
)]
fn doc_bacnet_whois_router() {}

/// Point discovery (object-list walk + commandable detection).
#[utoipa::path(
    post,
    path = "/bacnet/discover",
    tag = "BACnet",
    request_body = DeviceInstanceRequest,
    security(("BearerAuth" = [])),
    responses((status = 200, description = "Discovered objects"))
)]
fn doc_bacnet_discover() {}

/// Open-FDD alias for `/bacnet/discover`.
#[utoipa::path(
    post,
    path = "/api/bacnet/point-discovery",
    tag = "Open-FDD compat",
    request_body = DeviceInstanceRequest,
    security(("BearerAuth" = [])),
    responses((status = 200, description = "Discovered objects"))
)]
fn doc_api_point_discovery() {}

/// Read all 16 priority-array slots.
#[utoipa::path(
    post,
    path = "/bacnet/priority-array",
    tag = "BACnet",
    request_body = BacnetObjectRef,
    security(("BearerAuth" = [])),
    responses((status = 200, description = "Priority array slots"))
)]
fn doc_bacnet_priority_array() {}

/// Supervisory override audit across commandable points.
#[utoipa::path(
    post,
    path = "/bacnet/supervisory",
    tag = "BACnet",
    request_body = DeviceInstanceRequest,
    security(("BearerAuth" = [])),
    responses((status = 200, description = "Override audit"))
)]
fn doc_bacnet_supervisory() {}

/// Background poll engine status and last values.
#[utoipa::path(
    get,
    path = "/bacnet/poll/status",
    tag = "BACnet",
    security(("BearerAuth" = [])),
    responses((status = 200, description = "Poll status"))
)]
fn doc_bacnet_poll_status() {}

/// Run one poll cycle immediately.
#[utoipa::path(
    post,
    path = "/bacnet/poll/once",
    tag = "BACnet",
    security(("BearerAuth" = [])),
    responses((status = 200, description = "Poll cycle result"))
)]
fn doc_bacnet_poll_once() {}

/// List all hosted server objects (device 599999).
#[utoipa::path(
    get,
    path = "/bacnet/server/objects",
    tag = "BACnet",
    security(("BearerAuth" = [])),
    responses((status = 200, description = "Hosted objects"))
)]
fn doc_bacnet_server_objects() {}

/// Open-FDD alias for hosted server points.
#[utoipa::path(
    get,
    path = "/api/bacnet/server/points",
    tag = "Open-FDD compat",
    security(("BearerAuth" = [])),
    responses((status = 200, description = "Hosted objects"))
)]
fn doc_api_server_points() {}

/// List commandable hosted points (BACnet-writable, API read-only).
#[utoipa::path(
    get,
    path = "/bacnet/server/commandable",
    tag = "BACnet",
    security(("BearerAuth" = [])),
    responses((status = 200, description = "Commandable points"))
)]
fn doc_bacnet_server_commandable() {}

/// Update server-owned points (rejects commandable points).
#[utoipa::path(
    post,
    path = "/bacnet/server/update",
    tag = "BACnet",
    request_body = ServerUpdatePointsRequest,
    security(("BearerAuth" = [])),
    responses((status = 200, description = "Update results"))
)]
fn doc_bacnet_server_update() {}

/// Cached Open-Meteo weather + BACnet mirror status.
#[utoipa::path(
    get,
    path = "/weather",
    tag = "Weather",
    security(("BearerAuth" = [])),
    responses((status = 200, body = WeatherResponse))
)]
fn doc_weather() {}

/// Force an immediate weather refresh.
#[utoipa::path(
    post,
    path = "/weather/refresh",
    tag = "Weather",
    security(("BearerAuth" = [])),
    responses((status = 200, body = WeatherResponse))
)]
fn doc_weather_refresh() {}

/// Modbus TCP batch register read.
#[utoipa::path(
    post,
    path = "/modbus/read",
    tag = "Modbus",
    request_body = ModbusReadRequest,
    security(("BearerAuth" = [])),
    responses(
        (status = 200, description = "Register readings"),
        (status = 400, description = "Validation error"),
        (status = 502, description = "Modbus transport error")
    )
)]
fn doc_modbus_read() {}

/// Haystack server about.
#[utoipa::path(
    get,
    path = "/haystack/about",
    tag = "Haystack",
    security(("BearerAuth" = [])),
    responses((status = 200, description = "Haystack about grid"))
)]
fn doc_haystack_about() {}

/// Haystack read (allowlisted filters only).
#[utoipa::path(
    post,
    path = "/haystack/read",
    tag = "Haystack",
    request_body = HaystackReadRequest,
    security(("BearerAuth" = [])),
    responses((status = 200, description = "Haystack grid"))
)]
fn doc_haystack_read() {}

/// Haystack nav tree.
#[utoipa::path(
    post,
    path = "/haystack/nav",
    tag = "Haystack",
    request_body = HaystackNavRequest,
    security(("BearerAuth" = [])),
    responses((status = 200, description = "Haystack nav grid"))
)]
fn doc_haystack_nav() {}

/// Haystack historical read.
#[utoipa::path(
    post,
    path = "/haystack/his-read",
    tag = "Haystack",
    request_body = HaystackHisReadRequest,
    security(("BearerAuth" = [])),
    responses((status = 200, description = "Historical grids by id"))
)]
fn doc_haystack_his_read() {}
