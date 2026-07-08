---
title: CSV point model
nav_order: 4
---

# CSV point model

The hosted BACnet server's objects are defined in **`config/objects.csv`**.

## Columns

- **`Name`** — friendly / API name for the point.
- **`PointType`** — BACnet object type code (see below).
- **`Units`** — engineering units for analog types (e.g. `degreesFahrenheit`, `percent`).
- **`Commandable`** — `Y` or `N`.
- **`Default`** — startup present-value.
- **`Instance`** — BACnet object instance number (`0..4194303`).

## Supported `PointType` codes

`AI`, `AO`, `AV`, `BI`, `BO`, `BV`, `CSV` (CharacterString Value), and the
multi-state types `MSI`, `MSO`, `MSV`.

## Read / write split

To avoid a REST-vs-BACnet **data race**, the two point classes are handled differently:

- **`Commandable=Y` (BACnet-writable):** a field or supervisory BACnet device may command these at any time. The REST API is **read-only** for them — `POST /bacnet/server/update` **rejects** writes to a commandable point. You can still *observe* whatever a BACnet client wrote via the read endpoints.
- **`Commandable=N` (server-owned):** sensors, weather feeds, diagnostics. This process owns them (weather loop, FDD updates) and they are the **only** points the API may write with **`POST /bacnet/server/update`** (`{ "updates": { "<name>": <value> } }`).

Other rules:

- **Instance stability:** explicit `Instance` values keep object identifiers stable across row reordering.
- **Duplicate identifiers:** rows that collide on `(object-type, Instance)` are skipped with an error log so object identity stays deterministic.
- **Naming:** point names use a lowercase-hyphenated convention (e.g. `outside-air-temperature`, `openfdd-optimization-enabled`).

## Reading hosted values

- **`GET /bacnet/server/objects`** — every hosted point with `present_value`, `commandable`, and `api_writable`.
- **`GET /bacnet/server/commandable`** — the commandable (BACnet-writable) points and their current present-values, so the API can see what BACnet clients wrote.

## Weather points

The `OA-WEATHER-*` rows are refreshed from Open-Meteo on an interval and mirrored
into their BACnet objects. See [Home](/) and the `/weather` endpoint.
