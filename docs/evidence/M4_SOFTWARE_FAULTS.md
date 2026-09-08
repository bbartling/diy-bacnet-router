# M4 software faults — CI covered, G9 OPEN

| Field | Value |
| --- | --- |
| Policy | Expand CI-safe fault/lifecycle cases; **no G9 PASS** |
| Surfaces | `DualBipRouterSession` unit tests; dual-B/IP netns link-down; routing mark clear |

## Covered in software

| Case | Where |
| --- | --- |
| Malformed / truncated BVLL UDP survival | `dual_bip_survives_malformed_udp` |
| Missing iface fail-closed (Linux) | `dual_bip_missing_iface_rejected_on_linux` |
| Identical bind+port rejected | `dual_bip_rejects_identical_bind` |
| Stop + rebind | `dual_bip_stop_rebind_same_ports` |
| Session timeout / inactive clears routing marks | `routing_marks_clear_data_plane_on_inactive` |
| Controlled `ip link set … down` — management stays up | `route-bip-bip-netns.sh` → `health_link_down.json` |

## Explicit non-claims

- G9 remains **Open** (USB unplug, physical NIC loss, duplicate MAC on trunk).
- Ordinary boot fail-closed unchanged.
