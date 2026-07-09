//! Hard-coded test-bench request bodies for Swagger UI "Try it out".
//!
//! Values match `scripts/bench.env.example`, `scripts/smoke_test.sh`, and the
//! BENS-BENCHTEST-BOX / Modbus / Niagara targets on 192.168.204.x.

use std::collections::BTreeMap;

use serde_json::{json, Value};
use utoipa::openapi::{OpenApi, RefOr, Schema};

/// Default bench API key when env is unset (local/docker demo only).
pub const DEFAULT_BENCH_API_KEY: &str = "bench-demo-key-1234567890";

pub fn swagger_api_key() -> String {
    for key in [
        "OPENFDD_FIELDBUS_SWAGGER_API_KEY",
        "OPENFDD_FIELDBUS_API_KEY",
        "RUSTY_GATEWAY_API_KEY",
    ] {
        if let Ok(v) = std::env::var(key) {
            let trimmed = v.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    DEFAULT_BENCH_API_KEY.to_string()
}

pub fn bacnet_read_example() -> Value {
    json!({
        "device_instance": 5007,
        "object_type": "analog-input",
        "object_instance": 1173,
        "property_id": "present-value"
    })
}

pub fn bacnet_rpm_example() -> Value {
    json!({
        "device_instance": 5007,
        "objects": [{
            "object_type": "analog-input",
            "object_instance": 1173,
            "properties": [{ "property_id": "present-value" }]
        }, {
            "object_type": "analog-output",
            "object_instance": 2466,
            "properties": [{ "property_id": "present-value" }]
        }]
    })
}

pub fn bacnet_whois_example() -> Value {
    json!({ "low": 5007, "high": 5007 })
}

pub fn bacnet_write_example() -> Value {
    json!({
        "device_instance": 5007,
        "object_type": "analog-output",
        "object_instance": 2466,
        "property_id": "present-value",
        "value": 42.0,
        "priority": 10,
        "approved": true
    })
}

/// Alternate body for `/bacnet/write-dry-run` (set `approved: false`).
#[allow(dead_code)]
pub fn bacnet_write_dry_run_example() -> Value {
    json!({
        "device_instance": 5007,
        "object_type": "analog-output",
        "object_instance": 2466,
        "property_id": "present-value",
        "value": null,
        "priority": 10,
        "approved": false
    })
}

pub fn bacnet_discover_example() -> Value {
    json!({ "device_instance": 5007 })
}

pub fn bacnet_priority_array_example() -> Value {
    json!({
        "device_instance": 5007,
        "object_type": "analog-output",
        "object_instance": 2466
    })
}

pub fn bacnet_server_update_example() -> Value {
    json!({
        "updates": {
            "openfdd-active-fault-count": 0.0,
            "outside-air-temperature": 72.0
        }
    })
}

pub fn modbus_read_example() -> Value {
    json!({
        "host": "192.168.204.14",
        "port": 1502,
        "unit_id": 1,
        "timeout": 5.0,
        "registers": [{
            "address": 0,
            "count": 1,
            "function": "input",
            "decode": "uint16",
            "label": "bench-reg-0"
        }]
    })
}

pub fn haystack_read_example() -> Value {
    json!({ "filter": "point and temp" })
}

pub fn haystack_nav_example() -> Value {
    json!({ "nav_id": null })
}

pub fn haystack_his_read_example() -> Value {
    json!({
        "ids": ["@demo:point"],
        "range_start": "yesterday",
        "range_end": "today"
    })
}

/// Patch component schemas so Swagger UI pre-fills Try-it-out bodies.
pub fn apply_bench_examples(openapi: &mut OpenApi) {
    let Some(components) = openapi.components.as_mut() else {
        return;
    };

    set_schema_example(
        &mut components.schemas,
        "BacnetReadRequest",
        bacnet_read_example(),
    );
    set_schema_example(
        &mut components.schemas,
        "BacnetRpmRequest",
        bacnet_rpm_example(),
    );
    set_schema_example(
        &mut components.schemas,
        "BacnetWhoisRequest",
        bacnet_whois_example(),
    );
    set_schema_example(
        &mut components.schemas,
        "BacnetWriteRequest",
        bacnet_write_example(),
    );
    set_schema_example(
        &mut components.schemas,
        "BacnetObjectRef",
        bacnet_priority_array_example(),
    );
    set_schema_example(
        &mut components.schemas,
        "DeviceInstanceRequest",
        bacnet_discover_example(),
    );
    set_schema_example(
        &mut components.schemas,
        "ServerUpdatePointsRequest",
        bacnet_server_update_example(),
    );
    set_schema_example(
        &mut components.schemas,
        "ModbusReadRequest",
        modbus_read_example(),
    );
    set_schema_example(
        &mut components.schemas,
        "HaystackReadRequest",
        haystack_read_example(),
    );
    set_schema_example(
        &mut components.schemas,
        "HaystackNavRequest",
        haystack_nav_example(),
    );
    set_schema_example(
        &mut components.schemas,
        "HaystackHisReadRequest",
        haystack_his_read_example(),
    );
}

fn set_schema_example(schemas: &mut BTreeMap<String, RefOr<Schema>>, name: &str, example: Value) {
    let Some(RefOr::T(Schema::Object(obj))) = schemas.get_mut(name) else {
        return;
    };
    obj.example = Some(example);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openapi::ApiDoc;
    use utoipa::OpenApi;

    use utoipa::openapi::Object;

    #[test]
    fn bench_examples_applied_to_openapi_schemas() {
        let mut doc = ApiDoc::openapi();
        apply_bench_examples(&mut doc);
        let schemas = &doc.components.as_ref().unwrap().schemas;
        let read = schemas.get("BacnetReadRequest").unwrap();
        if let RefOr::T(Schema::Object(Object {
            example: Some(ex), ..
        })) = read
        {
            assert_eq!(ex["device_instance"], 5007);
            assert_eq!(ex["object_instance"], 1173);
        } else {
            panic!("missing BacnetReadRequest example");
        }
    }

    #[test]
    fn default_bench_api_key_constant() {
        assert_eq!(DEFAULT_BENCH_API_KEY, "bench-demo-key-1234567890");
    }

    #[test]
    fn bench_write_dry_run_example_shape() {
        assert!(bacnet_write_dry_run_example()["approved"].is_boolean());
    }
}
