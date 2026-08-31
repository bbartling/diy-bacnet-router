# Architecture

## Trust boundaries

The appliance has three deliberately separated areas:

1. **BACnet data plane** — owns BACnet/IP, one MS/TP token master, the network
   routing table and NPDU forwarding. It must keep operating if the browser is
   slow or disconnected.
2. **Management plane** — read-mostly REST, OpenAPI, Prometheus and bounded
   WebSocket snapshots. Configuration writes are a later authenticated and
   audited feature.
3. **Operating system** — Buildroot image, Linux interfaces, SSH, service
   supervision, persistent configuration and update/recovery mechanisms.

```text
                ┌────────────────────────────────────┐
UDP/47808  ───► │ B/IP port                          │
                │       ┌────────────────────┐       │
                │       │ BACnet network     │       │
                │       │ layer + route table│       │
RS-485      ◄──►│ MS/TP └────────────────────┘       │
                │ port        │ atomic counters      │
                └─────────────┼──────────────────────┘
                              ▼
                 REST / WebSocket / Prometheus
                              ▼
                       React static assets
```

## Rust workspace

- `router-core`: stable configuration, capability and metrics contracts. No
  Axum, serial or BACnet dependency.
- `routerd`: process entry point, Linux metrics, HTTP/WebSocket endpoints and
  static React serving.
- future `rusty-bacnet-adapter`: the only crate allowed to translate upstream
  transport/network events into the stable router-core interfaces.

The adapter must use the actual pinned `bacnet-transport` and `bacnet-network`
APIs. It must not reimplement CRC, MS/TP state machines, BVLC or NPDU codecs.

## Packet path requirements

- B/IP and MS/TP have distinct configured network numbers.
- Forwarding is based on decoded NPDUs and BACnet network-layer behavior, not
  application-service proxying.
- Hop count, source/destination network addressing and network messages are
  preserved or updated according to the network-layer procedure.
- Global and remote broadcasts are bounded and deduplicated; they must not form
  loops.
- An NPDU that cannot be represented by the active MS/TP capability is rejected
  or reported; it is never silently truncated.
- A browser disconnect, metrics backpressure or slow disk may not delay the
  MS/TP state machine.

## Configuration ownership

`router.toml` owns application and BACnet settings. Linux owns NIC addresses,
routes, DNS and hostname. The web UI initially shows effective configuration
but cannot modify it. The write path is intentionally deferred until it has:

- authenticated administration;
- schema validation;
- atomic temporary-file + fsync + rename persistence;
- automatic backup and rollback;
- audit records without secrets;
- pre-apply duplicate network/MAC checks;
- an explicit restart/apply boundary.

## Frontend delivery

The React build is static. Rust serves it from the configured directory. The
browser uses a one-second aggregate WebSocket stream and falls back to a
five-second REST poll. The stream rate is bounded to 250–5000 ms.

## Operating-system targets

The initial image matrix is x86-64 QEMU/EFI and Raspberry Pi 3/4/5 64-bit. All
targets consume one application configuration and expose the same API. Board
differences remain in Buildroot and Linux configuration, not in BACnet logic.

