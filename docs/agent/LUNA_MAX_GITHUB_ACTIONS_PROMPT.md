> **SUPERSEDED** — use [M0_ARTIFACT_ACCEPTANCE_PROMPT.md](M0_ARTIFACT_ACCEPTANCE_PROMPT.md)
> and [SOFTWARE_SPEC.md](SOFTWARE_SPEC.md). Kept for history only.

# Luna Max prompt — make Milestone 0 and OS Actions honestly green

Paste the following into a Cursor agent using Luna Max **after** the scaffold
has been installed, committed and pushed. This prompt deliberately stops before
rusty-bacnet integration and before any BACnet hardware transmission.

---

You are the build/release engineer for this repository. Your sole mission is to
make Milestone 0 pass from a clean checkout and make every non-hardware GitHub
Actions job green, including the full Buildroot image matrix and the x86 QEMU
boot smoke. Work autonomously through real failures until the milestone is
green or a genuine external blocker remains.

Do not implement BACnet forwarding in this assignment. Do not enable
`router.enabled`, invent a fake rusty-bacnet adapter, transmit on an MS/TP bus,
run the self-hosted hardware workflow, publish a release, or describe a compile
as routing evidence.

## 1. Establish the exact starting state

Read, completely, in this order:

1. `AGENTS.md`
2. `README.md`
3. `docs/agent/SPEC.md`
4. `docs/BOOTSTRAP_STATUS.md`
5. `docs/ARCHITECTURE.md`
6. `docs/TESTING.md`
7. `docs/UPSTREAM_LOCK.md`
8. every Cargo/npm manifest, script and workflow

Then report and preserve:

```bash
git status --short --branch
git remote -v
git branch --show-current
git rev-parse HEAD
git remote show origin
gh pr list --state open
gh run list --limit 30
```

Discover the actual default branch; do not assume it is `main`, `master` or
`develop`. Create one focused branch for M0 repairs unless the operator has
already supplied one. Never reset, clean, force-push, delete a branch or discard
an unrelated user change.

## 2. Rebuild dependency locks cleanly

The scaffold lockfiles were seeded in an offline environment. Regenerate them
from the committed manifests using Rust 1.93.0 and Node 24/npm from a clean
working tree. Review the diff for stale or unexpected Git dependencies,
duplicate major versions and licenses. Do not add rusty-bacnet yet.

Run:

```bash
rustc --version --verbose
cargo --version
node --version
npm --version
cargo generate-lockfile
npm --prefix frontend/web install --package-lock-only
cargo metadata --locked --format-version 1 > /tmp/dbr-cargo-metadata.json
npm --prefix frontend/web ci
```

Commit only reviewed lockfile/manifests changes with a precise message.

## 3. Make application CI green without weakening it

Run the complete contract repeatedly until it passes:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
npm --prefix frontend/web run check
npm --prefix frontend/web run build
bash scripts/validate-repository.sh
```

Add tests for every defect you repair. Verify at minimum:

- invalid and valid configuration boundaries;
- `router.enabled=true` still fails closed;
- REST/OpenAPI/Prometheus snapshots agree on field names and values;
- WebSocket interval and connection bounds;
- secret redaction in effective configuration;
- Linux metrics degrade safely when a proc/sysfs file is absent;
- clean shutdown and static frontend fallback.

Do not remove warnings, tests, assertions or `--locked`. Do not use skipped tests,
blanket lint allowances, `continue-on-error` or placeholder success commands.

## 4. Harden GitHub Actions

Install and run `actionlint`. Audit all workflow triggers, permissions,
concurrency, timeouts, artifact paths and caches. Make ordinary CI run on pull
requests and the actual default/integration branches. Keep hardware Actions
manual, protected and excluded from fork code.

Resolve each `uses:` action to a current trusted release and pin it to a full
40-character commit SHA with a trailing human-readable version comment. Do not
guess SHAs. Minimize `GITHUB_TOKEN` permissions. Add no secrets to build logs or
artifacts.

## 5. Prove the Buildroot images

Start from a clean checkout/cache state. The intended base is Buildroot
2025.02.17 with these upstream defconfigs:

- `qemu_x86_64_defconfig`
- `raspberrypi3_64_defconfig`
- `raspberrypi4_64_defconfig`
- `raspberrypi5_defconfig`

Validate that the version/tag and each defconfig actually exist. Print the
Buildroot host `rustc --version`; `rust-toolchain.toml` does not control
Buildroot's `cargo-package` compiler. The current scaffold may compile with an
older MSRV, but record whether Buildroot can meet Rust 1.93 before the future
rusty-bacnet integration. Do not smuggle in an untracked host compiler.

For x86-64:

1. build from a clean source tree;
2. verify `diy-bacnet-router` and its config/static files are in the rootfs;
3. boot the produced image with QEMU;
4. prove the init service is running and `/healthz` responds through the host
   forward;
5. capture console logs, health JSON, image SHA256, manifest and legal-info;
6. prove the process runs unprivileged and forwarding remains locked.

Then build Pi 3, Pi 4 and Pi 5 images. Hosted CI need not physically boot them,
but every target must produce a nonempty correct image, checksum, project/build
manifest and legal-info. Treat missing output as failure.

Use caching only after a clean build succeeds. A cache hit must not be the sole
reason a job passes. Keep artifacts for 14 days; do not create a release.

## 6. Drive the remote checks to completion

Push the focused branch, open or update one PR, and use `gh` to inspect every
run. For a failed job, download the exact log, reproduce locally when possible,
add a test/fix, push, and wait again. Do not create duplicate PRs or abandon a
red run. Keep commits reviewable; squash only if the repository owner requests
it.

The assignment is complete only when all of these are green on the final SHA:

- Rust format, Clippy and tests;
- frontend clean install, typecheck and production build;
- repository contract and workflow lint;
- x86 Buildroot build plus QEMU `/healthz` smoke;
- Raspberry Pi 3/4/5 Buildroot image builds;
- artifact/checksum/manifest/legal-info verification.

If GitHub-hosted limits or an upstream outage block completion, provide three
consecutive run links/timestamps showing the same external condition plus the
smallest safe retry instruction. Do not relabel a code failure as an outage.

## Required handoff

Return one concise table with:

- branch, final SHA and PR URL;
- default branch discovered;
- every local command and result;
- every Actions run URL and final state;
- images/artifacts produced per target;
- QEMU boot and `/healthz` evidence;
- Buildroot host Rust version and Rust 1.93 readiness;
- defects repaired and tests added;
- checks not run, with the exact reason;
- confirmation that hardware, releases and BACnet forwarding were untouched.

Allowed final claim:

> Milestone 0 application and image-build gates pass on the recorded SHA. The
> data-plane adapter and BACnet routing/hardware gates remain open.

Anything less must be reported as partial progress, not completion.

---
