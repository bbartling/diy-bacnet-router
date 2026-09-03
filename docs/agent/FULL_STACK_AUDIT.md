# Full-stack audit and refactor contract — DIY BACnet Router

This document is the agent checklist for production-grade audits and refactors
of this repository. Read it with [SOFTWARE_SPEC.md](SOFTWARE_SPEC.md),
[AGENTS.md](../../AGENTS.md), and [ARCHITECTURE.md](../ARCHITECTURE.md).

Use it when asked to clean up technical debt, harden the stack, or prepare a
milestone for merge. **Preserve existing behavior unless behavior is demonstrably
incorrect.** Do not perform cosmetic churn merely because code could theoretically
be written differently.

---

## Project-specific stack (non-negotiable)

This is **not** a generic web app. Agents must respect these boundaries:

| Layer | Technology | Notes |
|-------|------------|-------|
| BACnet data plane | **rusty-bacnet** (pinned upstream) | Rust only. **No Python** in the router data plane, Buildroot rootfs runtime, or production CI paths. External Vibe13 prototype may reference Python for lab history — do not import it here. |
| Application | **Rust** (`router-core`, `routerd`, future `rusty-bacnet-adapter`) | Workspace forbids `unsafe` at root. Prefer pinned upstream APIs over copied stack internals. |
| Management UI | **React + TypeScript + Vite** | Static build embedded in the appliance image. Management only — must not block MS/TP token handling. |
| HTTP server | **Axum** | REST, OpenAPI, Prometheus, bounded WebSocket metrics, static asset serving. |
| OS images | **Buildroot** | x86_64, rpi3_64, rpi4_64, rpi5_64 via `scripts/build-image.sh` and `.github/workflows/build-os.yml`. |
| x86 verification | **QEMU** | `scripts/qemu-smoke.sh` with `-snapshot` so post-boot checksums remain valid. |
| ARM64 images | Build in CI | Raspberry Pi 64-bit targets build and publish checksums/manifests; QEMU smoke is the x86_64 gate unless a Pi-specific smoke job exists with evidence. |
| Host networking | **Linux + SSH** | IP addresses, routes, DNS, firewall, and hostname are configured over **SSH** with normal Linux tools (`ip`, `systemd-networkd`, etc.). Not browser-writable until M6 auth gates pass. |
| Serial / MS/TP | **Waveshare USB TO RS485 (C)** | Persist `/dev/serial/by-id/...` only. 8N1, allowed baud set, no RTS/GPIO direction control on this adapter. |
| Lab debug | **VMware Ubuntu guest** (`ubuntu2` @ `127.0.0.1:2222`) | Artifact-first workflow per [M0_ARTIFACT_ACCEPTANCE_PROMPT.md](M0_ARTIFACT_ACCEPTANCE_PROMPT.md). **Do not use WSL** for Buildroot lab work when the host WSL environment is unavailable or corrupt. VirtualBox remains an optional `vm-ensure.ps1 -Hypervisor virtualbox` path. |
| Dependency lock | `Cargo.lock`, `package-lock.json`, `config/upstream-lock.toml` | CI and Buildroot use `--locked`. Never commit moving `dev` branches. |

**Hard product boundaries** (also in AGENTS.md):

- Router forwards **NPDUs** between distinct BACnet networks — not an AI/BI/AV/BV object database.
- Default config is **fail-closed**: forwarding disabled, management on loopback, no default password, no write API.
- Do not claim BTL, Clause 9 conformance, BBMD, FDR, routing, or tested baud without gate evidence.

---

## 1. Repository-wide audit scope

Review the entire stack as one deployable appliance, not isolated subprojects:

- Rust workspace crates and manifests
- React/TypeScript source, Vite config, `frontend/web/dist` integration
- OpenAPI contract (`openapi/openapi.json`) and frontend types (`frontend/web/src/types.ts`)
- Buildroot external tree (`buildroot-external/`)
- CI (`.github/workflows/ci.yml`, `build-os.yml`, `hardware-mstp.yml`)
- Scripts (`scripts/build-image.sh`, `qemu-smoke.sh`, `validate-repository.sh`, VM helpers)
- Configuration samples (`config/`), upstream lock, evidence docs
- Documentation that affects build, run, or agent behavior

Look for dead code, duplicate logic, unused dependencies, stale docs, and broken build practices across **Rust, React, and Buildroot** together.

---

## 2. Rust dependency hygiene

- Remove unused dependencies, dev-dependencies, build-dependencies, and feature flags.
- Run `cargo tree -d` and investigate duplicate crate versions.
- Consolidate when practical; keep multiple versions when the graph legitimately requires them.
- Prefer `[workspace.dependencies]` when multiple members share a crate.
- Disable unnecessary default features where it meaningfully reduces build size or attack surface.
- **rusty-bacnet / bacnet-transport / bacnet-network**: changes belong in upstream PRs with failing tests first; pin full 40-character SHAs in `config/upstream-lock.toml` and `docs/UPSTREAM_LOCK.md`.

---

## 3. Rust dead code and duplicate code

Remove unjustified dead functions, structs, modules, imports, commented-out legacy code, and duplicate helpers.

Do **not** broadly use `#[allow(dead_code)]` or other suppressions to obtain a green build. Every suppression needs a documented technical reason.

---

## 4. Idiomatic Rust

Review for unnecessary `.clone()`, allocations, `collect()`, excessive `Arc<Mutex<...>>`, giant functions, swallowed errors, speculative abstractions, and blocking work on async executor threads.

**Data-plane code** must stay simple and predictable — token timing and forwarding must not depend on management-plane complexity.

---

## 5. Rust error handling

Audit `unwrap()`, `expect()`, `panic!()`, and ignored `Result`s in production paths.

- Runtime failures should return meaningful errors, not panic.
- Tests may use `unwrap()`/`expect()` for clarity.
- API errors must map to appropriate HTTP status codes without exposing secrets, paths, or stack traces.

---

## 6. Unsafe Rust

The workspace sets `unsafe_code = "forbid"`. Do not introduce `unsafe` without a compelling reason and an explicit workspace lint exception plus `// SAFETY:` documentation.

---

## 7. React and TypeScript dependency hygiene

- Single package manager: **npm** with authoritative `frontend/web/package-lock.json`.
- Remove unused npm packages, duplicate utility libraries, and abandoned polyfills.
- Do not mix npm/yarn/pnpm unless intentionally documented.

---

## 8. React dead code and duplicate code

Remove unused components, hooks, types, CSS, assets, routes, and duplicate API wrappers.

Avoid multiple implementations of the same component or API operation unless consolidation genuinely improves maintainability.

---

## 9. React best practices

Review for unnecessary state/effects, stale closures, missing loading/error/empty states, unstable list keys, and premature memoization.

Prefer normal React data flow. **Do not add a global state framework** unless complexity clearly warrants it.

---

## 10. TypeScript quality

Avoid unjustified `any`, unsafe assertions, non-null assertions, and duplicate interfaces for the same API shape.

Frontend metric and status types must stay aligned with `frontend/web/src/types.ts` and the OpenAPI/metrics schema. Counter names are **stable API contracts**.

---

## 11. React hooks

Verify Rules of Hooks, dependency arrays, cleanup of subscriptions/WebSocket listeners, and stale async request handling.

The metrics WebSocket hook must respect bounded poll intervals (250–5000 ms, default 1000 ms) and must not assume one message per BACnet frame.

---

## 12. API boundary (React ↔ Rust)

Treat the contract as architectural:

| Source of truth | Purpose |
|-----------------|---------|
| `openapi/openapi.json` | REST paths, request/response shapes |
| `frontend/web/src/types.ts` | Browser-side metrics and status types |
| `router-core` | Config validation, metrics snapshot structs |

Avoid handwritten models drifting independently. Prefer updating OpenAPI + types + Rust serde types together with tests. Do not add code generation unless it provides clear value here.

---

## 13. HTTP and REST hygiene

Verify sensible verbs and status codes on `/healthz`, `/api/*`, and `/metrics`.

Frontend must handle non-success responses intentionally. Do not return HTTP 200 for error conditions when a 4xx/5xx is appropriate.

---

## 14. Static React asset serving

Production flow:

1. `npm --prefix frontend/web ci && npm --prefix frontend/web run build`
2. Vite writes to `frontend/web/dist`
3. `routerd` serves from `management.web_root` (default `frontend/web/dist`)
4. Buildroot image embeds the built web root

Audit:

- hashed JS/CSS assets and MIME types
- SPA fallback via Axum `ServeDir` + `ServeFile` for `index.html`
- **API routes registered before static fallback** — unknown `/api/*` paths must **not** return `index.html`
- cache headers: long-lived for hashed assets; short-lived for `index.html` when configurable

---

## 15. Client-side routing

If client routes are added (e.g. `/mstp`, `/system`), direct navigation and refresh must work via SPA fallback.

Valid frontend paths → `index.html`. Unknown API paths → JSON/backend 404, not React HTML.

---

## 16–18. Caching, build quality, configuration

- Do not ship dev-only debug UI or secrets in frontend bundles.
- Frontend env vars are **public**; never embed passwords, signing keys, or backend tokens.
- Backend secrets stay in TOML/SSH-managed host config, not in React build-time variables.
- Validate required backend config at startup with useful diagnostics.

---

## 19–21. Security and input validation

Review CORS, XSS (`dangerouslySetInnerHTML`), auth boundaries (M6+), request size limits, and fail-closed defaults.

- Management bind address defaults to loopback.
- Browser config writes disabled until M6 auth/audit gates pass.
- All browser input is untrusted; TOML and future write APIs must validate on the Rust side.
- Never commit credentials; `validate-repository.sh` checks for hardcoded passwords and `ttyUSB` aliases.

---

## 22. Logging and observability

Prefer `tracing` over `println!` in production code. Do not log authorization headers, serial payloads, or credentials.

Prometheus and WebSocket metrics deliver **aggregate snapshots** only — never per-packet browser events.

---

## 23–25. Accessibility, CSS, frontend errors

Preserve existing DBR visual design unless fixing correctness or maintainability.

Ensure async operations surface loading, error, and empty states without exposing internal backend details.

---

## 26. Performance

**Data plane:** atomics or bounded nonblocking channels; no unbounded queues; management backpressure must not stall MS/TP.

**Frontend:** avoid duplicate API calls, unnecessary rerenders, and polling faster than the configured metrics interval.

---

## 27. Build integration

One clear production path:

```bash
npm --prefix frontend/web ci
npm --prefix frontend/web run build
cargo build --release -p routerd
# Appliance image:
bash scripts/build-image.sh x86_64   # or rpi4_64, etc.
```

Buildroot packages the frontend via `buildroot-external/package/diy-bacnet-router/`. Do not commit `frontend/web/dist` unless a documented exception requires it.

Avoid hidden manual copy steps — automate in scripts or Buildroot recipes.

---

## 28. Docker and deployment

There is **no Dockerfile** in this repository today. Appliance delivery is **Buildroot images** (raw/ISO/IMG + SHA256SUMS + manifest), not container-first.

If Docker is added later, use multi-stage builds (frontend → Rust → minimal runtime) and do not ship Node/Rust toolchains in the final image unless required.

---

## 29–30. Rust validation gates

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo check --workspace --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
```

Do not remove or weaken tests to pass. Add regression tests when fixing bugs.

---

## 31–33. React validation gates

```bash
npm --prefix frontend/web ci
npm --prefix frontend/web run check    # TypeScript
npm --prefix frontend/web run build    # production Vite build
```

Add `npm run lint` only if an ESLint config is introduced; until then, `check` + `build` are the TypeScript gate.

---

## 34–35. Full application and smoke testing

Validate Rust + React together:

- `routerd` starts with built `frontend/web/dist` present
- `/healthz` returns JSON with honest `ready_to_route`
- `/api/openapi.json` serves valid OpenAPI
- static JS/CSS load with correct content types
- SPA fallback works for frontend routes
- unknown `/api/...` routes do **not** return `index.html`
- WebSocket **`/api/ws/metrics`** delivers bounded aggregate snapshots

Existing HTTP-level tests in `crates/routerd/src/web.rs` are the baseline. Extend with integration/smoke tests where practical — prefer lightweight Axum/tower tests over heavy browser automation unless justified.

**Appliance-level smoke (M0+):**

```bash
bash scripts/qemu-smoke.sh <images-dir>   # uses -snapshot
curl -sf http://<guest>:8080/healthz
```

---

## 36. Buildroot and appliance audit

When touching images or CI:

| Item | Requirement |
|------|-------------|
| Targets | x86_64 (QEMU gate), rpi3_64, rpi4_64, rpi5_64 |
| Entry script | `scripts/build-image.sh` |
| CI workflow | `.github/workflows/build-os.yml` |
| Checksums | `SHA256SUMS` generated before QEMU; verify after `-snapshot` boot |
| Legal info | Produced per target; no false "PASS" without artifact |
| Rootfs services | `routerd`, `openssh`, serial by-id udev, embedded web root |
| Host Rust in BR | Independent from `rust-toolchain.toml`; do not duplicate `--locked` in package mk |
| Upstream lock | `config/upstream-lock.toml` matches committed `Cargo.lock` |
| Lab workflow | [LOCAL_BUILDROOT_VM.md](../operations/LOCAL_BUILDROOT_VM.md) — artifact acceptance before local Buildroot debugging |
| Contract tests | `scripts/test-image-evidence-contract.sh` via `validate-repository.sh` |

**QEMU rules:**

- Always use `-snapshot` in smoke tests so `rootfs.ext2` checksums remain valid.
- x86_64 is the mandatory boot gate in CI; Pi images prove build reproducibility.

**SSH / Linux networking:**

- Document that operators configure NICs and routes over SSH.
- Application TOML owns BACnet ports and router policy only.
- Buildroot defconfig must include SSH server and basic networking tools appropriate to the image.

**Serial / hardware jobs:**

- Hardware MS/TP workflows run only from trusted branches (`hardware-mstp.yml`).
- Never run hardware tests from untrusted PRs.
- Passive decode must pass before any transmit on a live trunk.

---

## 37. CI audit

Ensure CI enforces:

| Job | Validates |
|-----|-----------|
| `ci` / `rust` | fmt, clippy, test (`--locked`) |
| `ci` / `web` | npm ci, audit (high), check, build |
| `ci` / `repository-contract` | `validate-repository.sh`, `validate-workflows.sh` |
| `ci` / `security` | cargo-audit, cargo-deny |
| `build-os` | per-target images, checksums, x86 QEMU smoke |

Do not duplicate jobs without benefit. Do not weaken gates (`continue-on-error`, optional required checks) to obtain green builds.

---

## 38. Dependency security

CI runs `cargo audit`, `cargo deny`, and `npm audit --audit-level=high`.

Do not blindly upgrade — confirm reachability and breaking changes. Record upstream rusty-bacnet SHA changes in the lock docs.

---

## 39–41. No fake cleanup; architectural simplicity

Do not delete tests, hide warnings, or rewrite the app without necessity.

Prefer:

- clear module ownership (`router-core` vs `routerd` vs adapter)
- small focused components and Rust modules
- explicit state transitions and fail-closed defaults

Avoid speculative service layers, DI frameworks, or plugin systems.

---

## 42. Final validation gate (mandatory before claiming done)

Run and report results for **every** command:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo check --workspace --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
npm --prefix frontend/web ci
npm --prefix frontend/web run check
npm --prefix frontend/web run build
bash scripts/validate-repository.sh
```

When Buildroot or appliance work is in scope, also report:

- latest `build-os` Actions conclusion (or M0 artifact acceptance on lab VM)
- QEMU smoke result with run ID / artifact path

Inspect the git diff for accidental generated files, secrets, unrelated churn, or unintended lockfile changes.

**Do not claim the repository is production-ready or fully validated unless these gates were executed successfully.**

---

## 43. Completion report template

When finishing an audit or refactor, provide:

1. Problems discovered
2. Rust changes made
3. React/TypeScript changes made
4. Dependencies removed
5. Dependency versions consolidated
6. Duplicate Rust crates that remain (and why)
7. Duplicate frontend packages that remain (and why)
8. Dead code removed
9. Duplicate logic consolidated
10. Unsafe Rust findings (expect: none under current workspace lint)
11. API contract improvements
12. Static asset serving improvements
13. Security findings and fixes
14. Tests added or improved
15. Build/deployment/Buildroot improvements
16. CI improvements
17. Exact validation commands executed
18. Final result of every validation gate
19. Remaining technical debt that could not safely be addressed

---

## Related files

| File | Role |
|------|------|
| [SOFTWARE_SPEC.md](SOFTWARE_SPEC.md) | Product and UI specification |
| [AGENTS.md](../../AGENTS.md) | Non-negotiable engineering contract |
| [SPEC.md](SPEC.md) | Milestones M0–M6 |
| [TESTING.md](../TESTING.md) | Gate labels G0–G11 |
| [UPSTREAM_LOCK.md](../UPSTREAM_LOCK.md) | rusty-bacnet pin policy |
| [LOCAL_BUILDROOT_VM.md](../operations/LOCAL_BUILDROOT_VM.md) | VMware Ubuntu lab workflow |
