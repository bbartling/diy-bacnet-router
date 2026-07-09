use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::openapi::Server;
use utoipa::OpenApi;

use crate::models::*;
use crate::openapi_paths::*;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Open-FDD Field-Bus Sidecar (Rust)",
        version = "1.2.0",
        description = "Pure Rust axum API over rusty-bacnet, rusty-modbus, and rusty-haystack.\n\n\
            **Remote access:** bind HTTP to `0.0.0.0` (`OPENFDD_FIELDBUS_HTTP_HOST`) and set \
            `OPENFDD_FIELDBUS_API_KEY`. Send `Authorization: Bearer <key>` on protected routes.\n\n\
            **Open-FDD prefix:** every route below (except `/`, `/health`, `/api/health`, docs) \
            is also available under `/api/*` — e.g. `/api/bacnet/read`, `/api/weather`."
    ),
    paths(
        doc_root,
        doc_health,
        doc_api_health,
        doc_bacnet_points,
        doc_bacnet_read,
        doc_bacnet_write,
        doc_bacnet_write_dry_run,
        doc_bacnet_rpm,
        doc_bacnet_whois,
        doc_bacnet_whois_router,
        doc_bacnet_discover,
        doc_api_point_discovery,
        doc_bacnet_priority_array,
        doc_bacnet_supervisory,
        doc_bacnet_poll_status,
        doc_bacnet_poll_once,
        doc_bacnet_server_objects,
        doc_api_server_points,
        doc_bacnet_server_commandable,
        doc_bacnet_server_update,
        doc_weather,
        doc_weather_refresh,
        doc_modbus_read,
        doc_haystack_about,
        doc_haystack_read,
        doc_haystack_nav,
        doc_haystack_his_read,
    ),
    components(schemas(
        BacnetReadRequest,
        BacnetRpmPropertySpec,
        BacnetRpmObjectSpec,
        BacnetRpmRequest,
        BacnetWhoisRequest,
        BacnetWriteRequest,
        BacnetObjectRef,
        DeviceInstanceRequest,
        ServerUpdatePointsRequest,
        ModbusRegisterOp,
        ModbusReadRequest,
        HaystackReadRequest,
        HaystackNavRequest,
        HaystackHisReadRequest,
        WeatherResponse,
        OkResponse,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "Root", description = "Service metadata"),
        (name = "BACnet", description = "BACnet client + hosted server"),
        (name = "Weather", description = "Open-Meteo weather cache"),
        (name = "Modbus", description = "Modbus TCP reads"),
        (name = "Haystack", description = "Read-only Haystack client"),
        (name = "Open-FDD compat", description = "Open-FDD /api aliases"),
    )
)]
pub struct ApiDoc;

pub struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "BearerAuth",
                SecurityScheme::Http(
                    Http::builder()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("API Key")
                        .description(Some(
                            "Set OPENFDD_FIELDBUS_API_KEY in .env, then send \
                             `Authorization: Bearer <key>` on protected routes. \
                             Use the Authorize button in Swagger UI.",
                        ))
                        .build(),
                ),
            );
        }
        openapi.security = Some(vec![utoipa::openapi::security::SecurityRequirement::new(
            "BearerAuth",
            Vec::<String>::new(),
        )]);

        // Optional public URL for Swagger "Try it out" (e.g. http://192.168.204.55:8080).
        if let Ok(url) = std::env::var("RUSTY_GATEWAY_SWAGGER_SERVERS_URL") {
            let trimmed = url.trim();
            if !trimmed.is_empty() {
                openapi.servers = Some(vec![Server::new(trimmed)]);
            }
        } else if let Ok(url) = std::env::var("OPENFDD_FIELDBUS_SWAGGER_SERVERS_URL") {
            let trimmed = url.trim();
            if !trimmed.is_empty() {
                openapi.servers = Some(vec![Server::new(trimmed)]);
            }
        }
    }
}
