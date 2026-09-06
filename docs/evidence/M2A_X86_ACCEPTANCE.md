# M2A — x86 Buildroot artifact acceptance (G6 tip)

| Field | Value |
| --- | --- |
| Accepted project SHA | `35fe6182f6192aad72dccf6365d97b46e25fdf0b` |
| Tip `build-os` run | [34063045175](https://github.com/bbartling/diy-bacnet-router/actions/runs/34063045175) |
| x86_64 job | [101566897301](https://github.com/bbartling/diy-bacnet-router/actions/runs/34063045175/job/101566897301) |
| Artifact | `diy-bacnet-router-x86_64-35fe6182f6192aad72dccf6365d97b46e25fdf0b` |
| Local download (Windows) | `C:\Users\ben\Documents\dbr-artifacts\g6-35fe618\` (**not** in git) |
| Buildroot | `2026.05.2` (`72d9d4fa636a371ef9eb99c92a735ce9f6d829d5`) |
| rusty-bacnet pin | `24e3439694b7d286e57e0a80cf7f1df4bd39d8ad` |
| Image VERSION / health | `0.0.1` |

## Claims (exact)

1. **Artifact identity:** downloaded artifact `build-manifest.json` records `project_git_sha` = accepted SHA and rusty-bacnet pin above.
2. **Checksums:** host-verified SHA256 for required payloads match artifact `SHA256SUMS`:

| File | SHA256 |
| --- | --- |
| `bzImage` | `d0b8fe2c622117f292f39a9af5404b18954e3bb02f56ba379caa69a61abf831b` |
| `rootfs.ext2` | `9883740b853d020a47fbf73143325ed2bc792903084d6c79a3844bd287767164` |
| `build-manifest.json` | `46afe7ffa5059f2aeb470cfa24b9064be2aca795676c8d59be7cd46ce1d21d5e` |
| `start-qemu.sh` | `a6a246f0d76adea3e57dfc381846e8e44d7d1567792c4ffe25e663de6f59278d` |

3. **QEMU smoke on that image tree (Actions, same run):** `scripts/qemu-smoke.sh` on `${RUNNER_TEMP}/dbr-buildroot/output/x86_64/images` before packaging:

```text
QEMU management health PASS (data plane disabled; service unprivileged)
Health JSON: {"data_plane":"disabled","management_plane":"operational","ready_to_route":false,"status":"ok","version":"0.0.1"}
QEMU log evidence: Starting diy-bacnet-router: OK (uid=100)
```

Asserted by smoke: `status=ok`, management operational, data plane disabled, `ready_to_route=false`, unprivileged service start. Post-smoke `sha256sum --check --strict SHA256SUMS` on packaged images succeeded in the same job.

## VMware guest path

**Not PASS.** SSH to lab guest `127.0.0.1:2222` refused (guest powered off / not listening). `scripts/vm-accept-artifact.sh` was **not** run. Acceptance is **QEMU (Actions) + local SHA256 verify of the downloaded artifact**, not VMware boot and not `cargo run`.

## Skipped on this accept

- Guest in-image re-check of VERSION / static UI / tty absence after VMware flash (blocked).
- Opt-in G6 netns on the guest (blocked; G6 already evidenced on Actions `bip-qualify` for this tip).
- Committing images or legal-info tarball into git.

## Non-claims

- No NPDU forwarding / BBMD / FDR
- No physical MS/TP (M2B)
- No VMware appliance boot PASS
- No Clause 9 / BTL / production readiness
