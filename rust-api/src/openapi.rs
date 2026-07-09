use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::openapi::Server;
use utoipa::{Modify, OpenApi};

use crate::models::*;
use crate::openapi_bench;
use crate::openapi_paths::*;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Open-FDD Field-Bus Sidecar (Rust)",
        version = "1.2.0",
        description = "Open-FDD field-bus REST API.\n\n\
            **Auth:** set `OPENFDD_FIELDBUS_API_KEY` and send `Authorization: Bearer <key>` \
            on protected routes.\n\n\
            **Routes:** also under `/api/*` (e.g. `/api/bacnet/read`, `/api/weather`). \
            Public: `/`, `/health`, `/api/health`, docs.\n\n\
            **Who-Is:** `POST /bacnet/whois` with `{}` scans all device instances \
            (0–4194303); set `low`/`high` to narrow. **Routed MS/TP:** \
            `POST /bacnet/whois-router` (no body)."
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
    modifiers(&SecurityAddon, &SwaggerExamplesAddon),
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

/// Build OpenAPI spec with runtime server URL and Swagger examples.
pub fn build_openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let bench = openapi_bench::swagger_bench_enabled();
        let auth_desc = if bench && openapi_bench::swagger_may_reveal_demo_key() {
            format!(
                "Authorize with `Bearer <key>`. Bench demo key: `{}`",
                openapi_bench::DEFAULT_BENCH_API_KEY
            )
        } else {
            "Authorize with `Bearer <key>` using your configured `OPENFDD_FIELDBUS_API_KEY`."
                .to_string()
        };

        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "BearerAuth",
                SecurityScheme::Http(
                    Http::builder()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("API Key")
                        .description(Some(auth_desc))
                        .build(),
                ),
            );
        }
        openapi.security = Some(vec![utoipa::openapi::security::SecurityRequirement::new(
            "BearerAuth",
            Vec::<String>::new(),
        )]);

        if bench {
            if let Some(info) = openapi.info.description.as_mut() {
                info.push_str(
                    "\n\n**Bench mode:** POST bodies pre-fill test-bench targets (device 5007, \
                     Modbus 192.168.204.14:1502, Haystack filters). Set \
                     `OPENFDD_FIELDBUS_SWAGGER_BENCH=0` or use a non-demo API key to hide them.",
                );
            }
        }

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

pub struct SwaggerExamplesAddon;

impl Modify for SwaggerExamplesAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi_bench::apply_swagger_examples(openapi);
    }
}
