# fieldbus-remote-bench

Standalone Rust HTTP client for exercising a **remote** `diy-bacnet-server` sidecar from your Windows (or any) machine. No path dependencies on `rusty-bacnet` — only `reqwest`.

Same bench targets as `scripts/smoke_test.sh` / Open-FDD smoke profiles:

| Protocol | Target |
|----------|--------|
| BACnet | device **5007**, AI:1173, AO:2466 (P8=55% override) |
| Modbus | **192.168.204.14:1502** unit 1 |
| Haystack | whatever the sidecar `.env` points at (Niagara or local demo) |

## Windows (PowerShell)

```powershell
cd diy-bacnet-server\remote-bench
$env:FIELDBUS_BASE = "http://192.168.204.55:8080"
$env:OPENFDD_FIELDBUS_API_KEY = "bench-demo-key-1234567890"
cargo run --release
```

Read-only (no BACnet writes):

```powershell
cargo run --release -- --read-only
```

## Remote Swagger

Open in a browser (same LAN or VPN):

- **Swagger UI:** `http://192.168.204.55:8080/docs`
- **OpenAPI JSON:** `http://192.168.204.55:8080/openapi.json`

Click **Authorize** and paste the same Bearer API key.

## CLI options

```
--base / FIELDBUS_BASE              Sidecar URL
--api-key / OPENFDD_FIELDBUS_API_KEY
--bacnet-device                     default 5007
--modbus-host                       default 192.168.204.14
--modbus-port                       default 1502
--read-only                         skip write/release
```
