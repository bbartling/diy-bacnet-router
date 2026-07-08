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

## Behaviour

- **`Commandable=Y`:** intended for BACnet writes (priority arrays). Do **not** push these via `POST /bacnet/server/update` — the server leaves them to BACnet clients.
- **`Commandable=N`:** server-owned values (sensors, weather feeds, diagnostics). Update them with **`POST /bacnet/server/update`** (`{ "updates": { "<name>": <value> } }`).
- **Instance stability:** explicit `Instance` values keep object identifiers stable across row reordering.
- **Duplicate identifiers:** rows that collide on `(object-type, Instance)` are skipped with an error log so object identity stays deterministic.

## Reading hosted values

- **`GET /bacnet/server/objects`** — every hosted point and its present-value.
- **`GET /bacnet/server/commandable`** — writable / commandable hosted points.

## Weather points

The `OA-WEATHER-*` rows are refreshed from Open-Meteo on an interval and mirrored
into their BACnet objects. See [Home](/) and the `/weather` endpoint.
