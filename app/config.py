"""Application settings and config loaders."""

from __future__ import annotations

import csv
import os
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

try:
    import tomllib
except ImportError:
    import tomli as tomllib  # type: ignore


ROOT = Path(__file__).resolve().parent.parent
CONFIG_DIR = Path(
    os.environ.get("OPENFDD_FIELDBUS_CONFIG_DIR")
    or os.environ.get("RUSTY_GATEWAY_CONFIG_DIR")
    or (ROOT / "config")
)


@dataclass
class BacnetServerSettings:
    device_instance: int = 599999
    device_name: str = "OpenFDD"
    # Bind on all interfaces so broadcast Who-Is (sent to x.x.x.255 /
    # 255.255.255.255 by BMS discovery) is actually delivered to this socket.
    # A socket bound to a specific unicast IP does NOT receive broadcast
    # datagrams on Linux, which makes the device invisible to remote discovery.
    interface: str = "0.0.0.0"
    port: int = 47808
    broadcast: str = "192.168.204.255"


@dataclass
class BacnetClientSettings:
    interface: str = "192.168.204.55"
    broadcast: str = "192.168.204.255"
    whois_bind_port: int = 47808
    read_bind_port: int = 0
    apdu_timeout_ms: int = 6000
    whois_timeout_secs: float = 8.0


@dataclass
class WeatherSettings:
    city: str = "Madison Wisconsin"
    interval_secs: int = 1200
    http_timeout_secs: float = 20.0
    fallback_temp_f: float = 70.0
    fallback_humidity: float = 50.0
    fallback_wind_mph: float = 0.0
    mirror_interval_secs: float = 2.0


@dataclass
class ModbusSettings:
    default_host: str = "127.0.0.1"
    default_port: int = 5502
    default_unit_id: int = 1
    default_timeout_secs: float = 5.0


@dataclass
class HaystackSettings:
    base_url: str = "http://127.0.0.1:8081"
    username: str = "admin"
    password: str = "admin"


@dataclass
class PollSettings:
    """Background BACnet poll loop over configured field-device points."""

    enabled: bool = True
    interval_secs: float = 60.0
    startup_delay_secs: float = 5.0
    max_samples: int = 5000


@dataclass
class FieldPoint:
    object_type: str
    object_instance: int
    point_name: str = ""
    units: str = ""


@dataclass
class FieldDevice:
    name: str
    enabled: bool
    device_instance: int
    host: str
    port: int = 47808
    mstp_network: int | None = None
    mstp_mac: list[int] = field(default_factory=list)
    points: list[FieldPoint] = field(default_factory=list)

    @property
    def address(self) -> str:
        return f"{self.host}:{self.port}"

    @property
    def is_routed(self) -> bool:
        return self.mstp_network is not None and bool(self.mstp_mac)


@dataclass
class HostedObjectRow:
    name: str
    point_type: str
    units: str
    commandable: bool
    default: str
    instance: int


@dataclass
class Settings:
    bacnet_server: BacnetServerSettings = field(default_factory=BacnetServerSettings)
    bacnet_client: BacnetClientSettings = field(default_factory=BacnetClientSettings)
    weather: WeatherSettings = field(default_factory=WeatherSettings)
    modbus: ModbusSettings = field(default_factory=ModbusSettings)
    haystack: HaystackSettings = field(default_factory=HaystackSettings)
    poll: PollSettings = field(default_factory=PollSettings)
    http_host: str = "0.0.0.0"
    http_port: int = 8080
    objects_csv: Path = field(default_factory=lambda: CONFIG_DIR / "objects.csv")
    field_devices_toml: Path = field(default_factory=lambda: CONFIG_DIR / "field_devices.toml")


# Env prefixes: the newer OPENFDD_FIELDBUS_* names take precedence, with the
# original RUSTY_GATEWAY_* names kept as fallbacks for compatibility.
def _env(*names: str) -> str | None:
    for n in names:
        v = os.environ.get(n)
        if v is not None and v != "":
            return v
    return None


def _env_bool(value: str | None, default: bool) -> bool:
    if value is None:
        return default
    return value.strip().lower() in ("1", "true", "yes", "on")


def git_sha() -> str:
    return _env("OPENFDD_FIELDBUS_GIT_SHA", "GIT_SHA") or "unknown"


def _subnet_broadcast(ip: str) -> str | None:
    """Directed /24 broadcast for an IPv4 (x.y.z.w -> x.y.z.255)."""
    parts = ip.split(".")
    if len(parts) != 4:
        return None
    try:
        octets = [int(p) for p in parts]
    except ValueError:
        return None
    if not all(0 <= o <= 255 for o in octets):
        return None
    return f"{octets[0]}.{octets[1]}.{octets[2]}.255"


def _merge_dict(base: dict[str, Any], override: dict[str, Any]) -> dict[str, Any]:
    out = dict(base)
    for k, v in override.items():
        if isinstance(v, dict) and isinstance(out.get(k), dict):
            out[k] = _merge_dict(out[k], v)
        else:
            out[k] = v
    return out


def load_gateway_toml() -> dict[str, Any]:
    path = CONFIG_DIR / "gateway.toml"
    if not path.exists():
        return {}
    with path.open("rb") as f:
        return tomllib.load(f)


def load_settings() -> Settings:
    raw = load_gateway_toml()
    s = Settings()

    if bs := raw.get("bacnet_server"):
        s.bacnet_server = BacnetServerSettings(**{k: bs[k] for k in BacnetServerSettings.__dataclass_fields__ if k in bs})
    if bc := raw.get("bacnet_client"):
        s.bacnet_client = BacnetClientSettings(**{k: bc[k] for k in BacnetClientSettings.__dataclass_fields__ if k in bc})
    if w := raw.get("weather"):
        s.weather = WeatherSettings(**{k: w[k] for k in WeatherSettings.__dataclass_fields__ if k in w})
    if m := raw.get("modbus"):
        s.modbus = ModbusSettings(**{k: m[k] for k in ModbusSettings.__dataclass_fields__ if k in m})
    if h := raw.get("haystack"):
        s.haystack = HaystackSettings(**{k: h[k] for k in HaystackSettings.__dataclass_fields__ if k in h})

    # Environment overrides. OPENFDD_FIELDBUS_* takes precedence over the
    # original RUSTY_GATEWAY_* names, which remain supported for compatibility.
    if v := _env("OPENFDD_FIELDBUS_BIND", "RUSTY_GATEWAY_BIND"):
        # Client uses the NIC IP as its unicast source; the server stays on
        # 0.0.0.0 so it can receive broadcast Who-Is. Derive the directed
        # subnet broadcast from the NIC IP unless one was set explicitly.
        s.bacnet_client.interface = v
        derived_bcast = _subnet_broadcast(v)
        if derived_bcast:
            s.bacnet_server.broadcast = derived_bcast
            s.bacnet_client.broadcast = derived_bcast
    if v := _env("OPENFDD_FIELDBUS_SERVER_BIND", "RUSTY_GATEWAY_SERVER_BIND"):
        s.bacnet_server.interface = v
    if v := _env("OPENFDD_FIELDBUS_BROADCAST", "RUSTY_GATEWAY_BROADCAST"):
        s.bacnet_server.broadcast = v
        s.bacnet_client.broadcast = v
    if v := _env("OPENFDD_FIELDBUS_HTTP_HOST", "RUSTY_GATEWAY_HTTP_HOST"):
        s.http_host = v
    if v := _env("OPENFDD_FIELDBUS_HTTP_PORT", "RUSTY_GATEWAY_HTTP_PORT"):
        s.http_port = int(v)
    if v := _env("OPENFDD_FIELDBUS_POLL_ENABLED", "RUSTY_GATEWAY_POLL_ENABLED"):
        s.poll.enabled = _env_bool(v, s.poll.enabled)
    if v := _env("OPENFDD_FIELDBUS_POLL_INTERVAL_SECS", "RUSTY_GATEWAY_POLL_INTERVAL_SECS"):
        s.poll.interval_secs = float(v)
    if v := _env("HAYSTACK_BASE_URL"):
        s.haystack.base_url = v
    if v := _env("HAYSTACK_USER"):
        s.haystack.username = v
    if v := _env("HAYSTACK_PASS"):
        s.haystack.password = v
    if v := _env("MODBUS_DEFAULT_HOST"):
        s.modbus.default_host = v

    return s


def load_objects_csv(path: Path | None = None) -> list[HostedObjectRow]:
    csv_path = path or (CONFIG_DIR / "objects.csv")
    rows: list[HostedObjectRow] = []
    with csv_path.open(newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        for row in reader:
            rows.append(
                HostedObjectRow(
                    name=row["Name"].strip(),
                    point_type=row["PointType"].strip(),
                    units=(row.get("Units") or "").strip(),
                    commandable=(row.get("Commandable") or "N").strip().upper() == "Y",
                    default=(row.get("Default") or "").strip(),
                    instance=int(row["Instance"]),
                )
            )
    return rows


def load_field_devices(path: Path | None = None) -> list[FieldDevice]:
    toml_path = path or (CONFIG_DIR / "field_devices.toml")
    with toml_path.open("rb") as f:
        data = tomllib.load(f)
    devices: list[FieldDevice] = []
    for d in data.get("devices", []):
        points = [
            FieldPoint(
                object_type=p["object_type"],
                object_instance=int(p["object_instance"]),
                point_name=p.get("point_name", ""),
                units=p.get("units", ""),
            )
            for p in d.get("points", [])
        ]
        devices.append(
            FieldDevice(
                name=d["name"],
                enabled=bool(d.get("enabled", True)),
                device_instance=int(d["device_instance"]),
                host=d["host"],
                port=int(d.get("port", 47808)),
                mstp_network=d.get("mstp_network"),
                mstp_mac=list(d.get("mstp_mac", [])),
                points=points,
            )
        )
    return devices
