use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::OpenApi;

use crate::models::*;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Open-FDD Field-Bus Sidecar (Rust)",
        version = "1.2.0",
        description = "Pure Rust axum API over rusty-bacnet, rusty-modbus, and rusty-haystack."
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
                            "When OPENFDD_FIELDBUS_API_KEY is set, send `Authorization: Bearer <key>` on protected routes.",
                        ))
                        .build(),
                ),
            );
        }
        openapi.security = Some(vec![utoipa::openapi::security::SecurityRequirement::new(
            "BearerAuth",
            Vec::<String>::new(),
        )]);
    }
}
