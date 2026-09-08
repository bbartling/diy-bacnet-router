# M6 auth/session skeleton — writes default-off

| Field | Value |
| --- | --- |
| Policy | Session/password/audit skeleton; **writes stay closed** unless lab unlock |
| Unlock | `DBR_ALLOW_WRITES=1` **and** write-secret file (`DBR_WRITE_SECRET_FILE` or `/etc/diy-bacnet-router/write.secret`) |
| e2e/QEMU | Must **not** set unlock env |

## Surfaces

| Piece | Behavior |
| --- | --- |
| `SessionStore` | Bounded token ring (32) |
| `hash_password_experimental` / `verify_password_experimental` | SHA-256(salt:password) hex — **lab-only**, not argon2/bcrypt |
| `POST /api/config` | Always **403** (blocked, or unlocked-but-unwired — no fake save) |
| `GET /api/audit` | `writes_enabled`, `write_secret_present`, events |
| UI Configuration | Shows writes-blocked status from `/api/audit` |
| Capability `management_writes` | Remains `BlockedByEvidence` |

## Explicit non-claims

- Not production auth/TLS/BBMD.
- Not management_writes Available.
- Password scheme is Experimental / lab-only.
