//! Remote bench client — hit a diy-bacnet-server sidecar over HTTP and print all values.
//!
//! Same bench targets as `scripts/smoke_test.sh` / Open-FDD smoke profiles.
//!
//! Windows (PowerShell):
//!   $env:FIELDBUS_BASE = "http://192.168.204.55:8080"
//!   $env:OPENFDD_FIELDBUS_API_KEY = "bench-demo-key-1234567890"
//!   cargo run --release
//!
//! Linux/macOS:
//!   FIELDBUS_BASE=http://192.168.204.55:8080 OPENFDD_FIELDBUS_API_KEY=... cargo run --release

use std::time::Duration;

use clap::Parser;
use reqwest::Client;
use serde_json::{json, Value};

#[derive(Parser, Debug)]
#[command(name = "fieldbus-remote-bench", about = "Remote field-bus sidecar bench client")]
struct Args {
    /// Sidecar base URL (no trailing slash)
    #[arg(long, env = "FIELDBUS_BASE", default_value = "http://127.0.0.1:8080")]
    base: String,

    /// Bearer API key (OPENFDD_FIELDBUS_API_KEY)
    #[arg(long, env = "OPENFDD_FIELDBUS_API_KEY")]
    api_key: Option<String>,

    /// BACnet device instance (bench MSTP router device)
    #[arg(long, env = "BENCH_BACNET_DEVICE", default_value_t = 5007)]
    bacnet_device: u32,

    #[arg(long, env = "BENCH_BACNET_READ_TYPE", default_value = "analog-input")]
    read_type: String,

    #[arg(long, env = "BENCH_BACNET_READ_INST", default_value_t = 1173)]
    read_inst: u32,

    #[arg(long, env = "BENCH_BACNET_OVR_TYPE", default_value = "analog-output")]
    ovr_type: String,

    #[arg(long, env = "BENCH_BACNET_OVR_INST", default_value_t = 2466)]
    ovr_inst: u32,

    #[arg(long, env = "BENCH_BACNET_WRITE_PRIORITY", default_value_t = 10)]
    write_priority: u8,

    #[arg(long, env = "BENCH_MODBUS_HOST", default_value = "192.168.204.14")]
    modbus_host: String,

    #[arg(long, env = "BENCH_MODBUS_PORT", default_value_t = 1502)]
    modbus_port: u16,

    #[arg(long, env = "BENCH_MODBUS_UNIT", default_value_t = 1)]
    modbus_unit: u8,

    #[arg(long, env = "BENCH_MODBUS_REG", default_value_t = 0)]
    modbus_reg: u16,

    #[arg(long, env = "BENCH_HAYSTACK_FILTER", default_value = "point and temp")]
    haystack_filter: String,

    /// Per-request timeout seconds
    #[arg(long, default_value_t = 120)]
    timeout_secs: u64,

    /// Skip BACnet write + release (read-only probe)
    #[arg(long, default_value_t = false)]
    read_only: bool,
}

struct Bench {
    client: Client,
    base: String,
    args: Args,
}

impl Bench {
    fn new(args: Args) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(args.timeout_secs))
            .build()
            .expect("http client");
        Self {
            base: args.base.trim_end_matches('/').to_string(),
            client,
            args,
        }
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(key) = &self.args.api_key {
            req.bearer_auth(key)
        } else {
            req
        }
    }

    async fn get(&self, path: &str) -> Result<Value, String> {
        let url = format!("{}{}", self.base, path);
        let resp = self
            .auth(self.client.get(&url))
            .send()
            .await
            .map_err(|e| format!("GET {path}: {e}"))?;
        let status = resp.status();
        let body: Value = resp.json().await.map_err(|e| format!("GET {path} json: {e}"))?;
        if !status.is_success() {
            return Err(format!("GET {path} HTTP {status}: {body}"));
        }
        Ok(body)
    }

    async fn post(&self, path: &str, body: Value) -> Result<Value, String> {
        let url = format!("{}{}", self.base, path);
        let resp = self
            .auth(self.client.post(&url).json(&body))
            .send()
            .await
            .map_err(|e| format!("POST {path}: {e}"))?;
        let status = resp.status();
        let out: Value = resp.json().await.map_err(|e| format!("POST {path} json: {e}"))?;
        if !status.is_success() {
            return Err(format!("POST {path} HTTP {status}: {out}"));
        }
        Ok(out)
    }

    fn section(&self, title: &str) {
        println!();
        println!("======== {title} ========");
    }

    fn print_ok(&self, label: &str, v: &Value) {
        println!("--- {label} ---");
        println!("{}", serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string()));
    }

    fn print_err(&self, label: &str, e: &str) {
        eprintln!("--- {label} FAILED ---");
        eprintln!("{e}");
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let b = Bench::new(args);

    println!("fieldbus-remote-bench");
    println!("base={}", b.base);
    println!("bacnet device={} read={}:{} override={}:{}", 
        b.args.bacnet_device, b.args.read_type, b.args.read_inst,
        b.args.ovr_type, b.args.ovr_inst);
    println!("modbus {}:{} unit={}", b.args.modbus_host, b.args.modbus_port, b.args.modbus_unit);
    println!();
    println!("Swagger UI:  {}/docs", b.base);
    println!("OpenAPI JSON: {}/openapi.json", b.base);
    println!("(Authorize in Swagger with the same Bearer API key)");

    // ---- Liveness ----
    b.section("Liveness");
    match b.get("/health").await {
        Ok(v) => b.print_ok("GET /health", &v),
        Err(e) => b.print_err("GET /health", &e),
    }
    match b.get("/api/health").await {
        Ok(v) => b.print_ok("GET /api/health", &v),
        Err(e) => b.print_err("GET /api/health", &e),
    }

    // ---- BACnet client ----
    b.section("BACnet client");
    let dev = b.args.bacnet_device;
    match b.get("/bacnet/points").await {
        Ok(v) => b.print_ok("GET /bacnet/points", &v),
        Err(e) => b.print_err("GET /bacnet/points", &e),
    }
    match b.post("/bacnet/whois", json!({"low": dev, "high": dev})).await {
        Ok(v) => b.print_ok("POST /bacnet/whois", &v),
        Err(e) => b.print_err("POST /bacnet/whois", &e),
    }
    match b.post("/bacnet/read", json!({
        "device_instance": dev,
        "object_type": b.args.read_type,
        "object_instance": b.args.read_inst
    })).await {
        Ok(v) => b.print_ok("POST /bacnet/read", &v),
        Err(e) => b.print_err("POST /bacnet/read", &e),
    }
    match b.post("/bacnet/rpm", json!({
        "device_instance": dev,
        "objects": [{
            "object_type": b.args.read_type,
            "object_instance": b.args.read_inst,
            "properties": [{"property_id": "present-value"}]
        }]
    })).await {
        Ok(v) => b.print_ok("POST /bacnet/rpm", &v),
        Err(e) => b.print_err("POST /bacnet/rpm", &e),
    }
    match b.post("/bacnet/discover", json!({"device_instance": dev})).await {
        Ok(v) => b.print_ok("POST /bacnet/discover", &v),
        Err(e) => b.print_err("POST /bacnet/discover", &e),
    }
    match b.post("/bacnet/priority-array", json!({
        "device_instance": dev,
        "object_type": b.args.ovr_type,
        "object_instance": b.args.ovr_inst
    })).await {
        Ok(v) => b.print_ok("POST /bacnet/priority-array", &v),
        Err(e) => b.print_err("POST /bacnet/priority-array", &e),
    }
    match b.post("/bacnet/supervisory", json!({"device_instance": dev})).await {
        Ok(v) => b.print_ok("POST /bacnet/supervisory", &v),
        Err(e) => b.print_err("POST /bacnet/supervisory", &e),
    }

    if !b.args.read_only {
        match b.post("/bacnet/write", json!({
            "device_instance": dev,
            "object_type": b.args.ovr_type,
            "object_instance": b.args.ovr_inst,
            "value": 42.0,
            "priority": b.args.write_priority,
            "approved": true
        })).await {
            Ok(v) => b.print_ok("POST /bacnet/write (P10 test)", &v),
            Err(e) => b.print_err("POST /bacnet/write", &e),
        }
        match b.post("/bacnet/write", json!({
            "device_instance": dev,
            "object_type": b.args.ovr_type,
            "object_instance": b.args.ovr_inst,
            "value": null,
            "priority": b.args.write_priority,
            "approved": true
        })).await {
            Ok(v) => b.print_ok("POST /bacnet/write (null release)", &v),
            Err(e) => b.print_err("POST /bacnet/write release", &e),
        }
    }

    match b.post("/bacnet/write-dry-run", json!({
        "device_instance": dev,
        "object_type": b.args.ovr_type,
        "object_instance": b.args.ovr_inst,
        "value": null,
        "priority": b.args.write_priority
    })).await {
        Ok(v) => b.print_ok("POST /bacnet/write-dry-run", &v),
        Err(e) => b.print_err("POST /bacnet/write-dry-run", &e),
    }
    match b.post("/bacnet/poll/once", json!({})).await {
        Ok(v) => b.print_ok("POST /bacnet/poll/once", &v),
        Err(e) => b.print_err("POST /bacnet/poll/once", &e),
    }
    match b.get("/bacnet/poll/status").await {
        Ok(v) => b.print_ok("GET /bacnet/poll/status", &v),
        Err(e) => b.print_err("GET /bacnet/poll/status", &e),
    }

    // ---- Hosted server + weather ----
    b.section("Hosted BACnet server + weather");
    match b.get("/bacnet/server/objects").await {
        Ok(v) => b.print_ok("GET /bacnet/server/objects", &v),
        Err(e) => b.print_err("GET /bacnet/server/objects", &e),
    }
    match b.get("/weather").await {
        Ok(v) => b.print_ok("GET /weather", &v),
        Err(e) => b.print_err("GET /weather", &e),
    }

    // ---- Modbus ----
    b.section("Modbus TCP");
    match b.post("/modbus/read", json!({
        "host": b.args.modbus_host,
        "port": b.args.modbus_port,
        "unit_id": b.args.modbus_unit,
        "timeout": 5.0,
        "registers": [{
            "address": b.args.modbus_reg,
            "count": 1,
            "function": "input",
            "decode": "uint16",
            "label": "bench"
        }]
    })).await {
        Ok(v) => b.print_ok("POST /modbus/read", &v),
        Err(e) => b.print_err("POST /modbus/read", &e),
    }

    // ---- Haystack ----
    b.section("Haystack (via sidecar upstream config)");
    match b.get("/haystack/about").await {
        Ok(v) => b.print_ok("GET /haystack/about", &v),
        Err(e) => b.print_err("GET /haystack/about", &e),
    }
    match b.post("/haystack/read", json!({"filter": b.args.haystack_filter})).await {
        Ok(v) => b.print_ok("POST /haystack/read", &v),
        Err(e) => b.print_err("POST /haystack/read", &e),
    }
    match b.post("/haystack/nav", json!({})).await {
        Ok(v) => b.print_ok("POST /haystack/nav", &v),
        Err(e) => b.print_err("POST /haystack/nav", &e),
    }

    // ---- OpenAPI summary ----
    b.section("OpenAPI");
    match b.get("/openapi.json").await {
        Ok(v) => {
            let paths = v.get("paths").and_then(|p| p.as_object()).map(|p| p.len()).unwrap_or(0);
            println!("paths={paths}  full spec at {}/openapi.json", b.base);
            if let Some(info) = v.get("info") {
                println!("info: {}", serde_json::to_string_pretty(info).unwrap_or_default());
            }
        }
        Err(e) => b.print_err("GET /openapi.json", &e),
    }

    println!();
    println!("Done.");
}
