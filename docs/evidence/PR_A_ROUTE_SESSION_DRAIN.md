# PR-A — Route session drain (software)

## Defect (before)

Upstream `BACnetRouter::start` returns a bounded (capacity **256**) `local_rx`
channel. Local final-hop and global-broadcast APDUs are dispatched with
blocking `local_tx.send().await`. Appliance and dual-B/IP sessions stored
`local_rx` but never read it, so a flood of local deliveries could **wedge
forwarding**.

## Fix (after)

| Surface | Behavior |
| --- | --- |
| `LocalDeliveryDrain` | Spawned on session `start()`; atomic count only (no unbounded queue); abort+join in `stop()` before `router.stop()` |
| `ApplianceRouterSession` | Always drains |
| `DualBipRouterSession` | Always drains; regression floods >256 then asserts A→B forward still completes |
| `LoopbackRouterSession` | Keeps `local_rx()` for unit assertions (no forced drain) |

## CI additions

- `bip_mstp_loopback.rs`: `BipTransport` ↔ `MstpTransport<LoopbackSerial>` directed NPDU both ways + stop/rebind (public pin APIs only).
- `--route-report` allowed with `--route-enable`; finite session shuts down Axum so the process exits without an external SIGTERM.
- Dual-B/IP netns: warmup excluded from scoring; measured bursts once; `observed == expected`.
- Route marks: `data_plane=Starting`, `bacnet_telemetry_available=false` while opt-in session is live.

## Commands

```bash
cargo test -p rusty-bacnet-adapter --all-features --locked
cargo test -p routerd --all-features --locked
# Linux root CI:
sudo scripts/route-bip-bip-netns.sh
```

## Still OPEN (do not claim PASS)

- Product **G4–G11** BIP↔physical-MS/TP bench
- Upstream MS/TP `BACnetRouter::stop()` / serial-release (U1) — first hardware G7/G8 may exit the whole process; restart/G9 waits on U1
- Invented BASRT / `dbr_forwarded_*` counters (still unavailable at pin)

Pin remains `24e3439694b7d286e57e0a80cf7f1df4bd39d8ad`.
