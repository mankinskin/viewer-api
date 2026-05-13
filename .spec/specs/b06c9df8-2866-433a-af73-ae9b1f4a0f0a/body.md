# viewer-api: structured tracing for WASM frontend

Specification for replacing ad-hoc `web_sys::console::log_1!()` calls in the
Dioxus WASM frontend (`tools/viewer/viewer-api/frontend/dioxus`, shared by
`doc-viewer`, `log-viewer`, `ticket-viewer`, `spec-viewer`) with structured
`tracing` instrumentation.

The reference Rust/native side already uses `tracing` extensively (see
`crates/context-trace`, `tools/http/spec-http`, `tools/viewer/log-viewer`).
The WASM frontend MUST follow the same pattern so logs are uniform across
process boundaries.

Tracking ticket: `b480632a-8605-4b5b-a4e8-f2988b2565a0`.
Companion spec: [`viewer-api/tracing/file-sink`](../tracing/file-sink) — ships
the captured records to a server endpoint for persistence.

---

## 1. Motivation

The recent shared-GPU refactor (commit `d09bfe39`) added several diagnostic
`console.log` calls to debug a cross-device WebGPU issue. Those calls are useful
but unstructured:

- No levels (everything is `LOG`).
- No spans — operations like "overlay bootstrap" or "graph3d init" cannot be
  visually grouped or filtered.
- No structured fields — values like `device.label` are baked into the message
  string, defeating downstream filtering.
- No persistence — DevTools-only output is lost on tab close.

Migrating to `tracing` gives all four of those properties and aligns the
frontend with the rest of the codebase.

---

## 2. Scope

### 2.1 Dependencies

Add to `tools/viewer/viewer-api/frontend/dioxus/Cargo.toml` under the
`[target.'cfg(target_arch = "wasm32")'.dependencies]` table:

| Crate           | Version | Reason                                        |
|-----------------|---------|-----------------------------------------------|
| `tracing`       | `0.1`   | Core API (`info!`, `warn!`, `error!`, spans). |
| `tracing-subscriber` | `0.3` (with `registry` and `env-filter` features) | Pluggable layer composition. |
| `tracing-wasm`  | `0.2`   | Console layer that writes to browser DevTools. |

`tracing-web` is an acceptable alternative to `tracing-wasm`; the choice is
deferred to implementation. Whichever is chosen MUST be reflected in the
Dioxus port of `ticket-viewer`, `doc-viewer`, `log-viewer`, and `spec-viewer`
(since they re-use the shared `viewer-api-dioxus` crate).

### 2.2 Subscriber bootstrap

A new function `viewer_api_dioxus::tracing_setup::install()` MUST be called
at the start of `App::main` in every viewer's `main.rs`, BEFORE any other
viewer code logs anything. It MUST be idempotent (safe to call twice).

Default configuration:

- Console layer (`tracing-wasm` or equivalent) installed at level `INFO`.
- An `EnvFilter` that reads from a query-string parameter `?log=...` (e.g.
  `?log=viewer_api_dioxus=debug,wgpu_overlay=trace`) and from `localStorage`
  key `viewer-api-log-filter`. Query string overrides `localStorage`.
- Spans rendered with timing and entry/exit markers in the console.

The default level is `INFO`. Build profile MUST NOT change the default — the
goal is consistent behavior between dev and release; verbosity is opt-in via
the filter mechanism.

### 2.3 Migration map

Every existing `web_sys::console::log_1!()` / `console::error_1!()` / `console::warn_1!()`
call in `tools/viewer/viewer-api/frontend/dioxus/src/**` MUST be replaced.
The mapping table:

| Existing call                            | Tracing replacement                                |
|------------------------------------------|----------------------------------------------------|
| `console::log_1(&"...".into())`          | `tracing::info!("...")`                            |
| `console::warn_1(&"...".into())`         | `tracing::warn!("...")`                            |
| `console::error_1(&"...".into())`        | `tracing::error!("...")`                           |
| `console::debug_1(&"...".into())`        | `tracing::debug!("...")`                           |

Each migrated call site MUST adopt structured fields when the message contains
inline values. Examples taken from the current diagnostic logs in
`effects/wgpu_overlay/gpu_init.rs` and `graph3d/mod.rs`:

```rust
// before
web_sys::console::log_1(&format!("[WgpuOverlay/init] device created label={}", dev_label).into());

// after
tracing::info!(target: "wgpu_overlay::init", device.label = %dev_label, "device created");
```

```rust
// before
web_sys::console::log_1(&format!(
    "[WgpuOverlay/frame] #{} t={:.2}s dt={:.4}s smoke={:.2} dev={}",
    n, time_s, dt_s, settings.smoke_intensity, dev_label
).into());

// after (gated by debug level — frame logs are noisy)
tracing::debug!(
    target: "wgpu_overlay::frame",
    frame.n = n,
    frame.time_s = time_s,
    frame.dt_s = dt_s,
    smoke = settings.smoke_intensity,
    device.label = %dev_label,
    "frame"
);
```

### 2.4 Spans

The following operations MUST be wrapped in `tracing::info_span!()`:

| Operation           | Span name                       | Fields                       |
|---------------------|---------------------------------|------------------------------|
| Overlay bootstrap   | `wgpu_overlay::bootstrap`       | (none — entry+exit only)     |
| GPU init            | `wgpu_overlay::init`            | `canvas.width`, `canvas.height` |
| Graph3D init        | `graph3d::init`                 | `device.label`               |
| Per-frame render    | `wgpu_overlay::frame`           | `frame.n`                    |
| Frame callbacks     | `wgpu_overlay::frame_callbacks` | `n.callbacks`                |

Per-frame spans are info-level; the `frame.n` field is set so a downstream
sink can filter to e.g. every 120th frame without losing the surrounding
records when investigating a glitch.

### 2.5 Uncaptured GPU errors

The existing `device.onuncapturederror` handler in
`effects/wgpu_overlay/gpu_init.rs` MUST emit at level `error` on the
`wgpu_overlay::uncaught` target with the validation message and the device
label as structured fields:

```rust
tracing::error!(
    target: "wgpu_overlay::uncaught",
    device.label = %dev_label,
    error = %msg,
    "uncaptured WebGPU validation error"
);
```

### 2.6 Configurability

Runtime level changes are supported via two mechanisms (resolved in this order):

1. URL query string `?log=<env-filter-spec>` — wins for the current page load only.
2. `localStorage["viewer-api-log-filter"]` — survives reloads.

Both accept the standard `EnvFilter` syntax (e.g. `info,wgpu_overlay=debug`).
A missing/empty value falls back to `info`.

The frontend MUST NOT expose a UI for changing the filter in this spec — that
is deferred to a possible follow-up. Power users edit `localStorage` or the URL.

### 2.7 Out of scope

- File sink (server-side persistence) — covered by the companion spec
  [`viewer-api/tracing/file-sink`](../tracing/file-sink).
- A UI control for editing the log filter.
- Tracing on the `viewer-api` Rust HTTP server — already uses `tracing`.
- Performance counters, perf marks, browser-perf-API integration.

---

## 3. Design

### 3.1 Module layout

```
tools/viewer/viewer-api/frontend/dioxus/src/
├── tracing_setup/
│   ├── mod.rs            # pub fn install(); pub fn current_filter() -> String
│   ├── filter.rs         # query-string + localStorage resolver
│   └── console_layer.rs  # thin wrapper that picks tracing-wasm or tracing-web
└── lib.rs                // re-exports tracing_setup::install
```

### 3.2 Bootstrap order

```rust
// main.rs (every viewer)
fn main() {
    viewer_api_dioxus::tracing_setup::install();
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "viewer starting");
    dioxus::launch(app);
}
```

`install` MUST tolerate being called from a non-WASM target (it becomes a
no-op) so that the same `main.rs` continues to compile under
`cargo check --target x86_64-pc-windows-msvc`.

### 3.3 Field naming

Field names MUST use dotted snake_case to match the rest of the codebase:
`device.label`, `frame.n`, `canvas.width`, `error.kind`, `error.message`.
This keeps later JSON output parseable by JQ filters without translation.

---

## 4. Acceptance Criteria

A change is considered complete when ALL of the following hold:

1. **Dependencies present.** `tracing`, `tracing-subscriber`, and one of
   {`tracing-wasm`, `tracing-web`} appear in
   `tools/viewer/viewer-api/frontend/dioxus/Cargo.toml`.
2. **Subscriber installed.** `viewer_api_dioxus::tracing_setup::install()` is
   called from every viewer's `main.rs` before any other code.
3. **No `console::log_1`/`warn_1`/`error_1` in production code.**
   `grep -r 'console::\(log\|warn\|error\|debug\)_1!' tools/viewer/viewer-api/frontend/dioxus/src` returns no matches outside `tracing_setup/`.
4. **Browser shows structured records.** Reload `http://localhost:4002/specs`
   and confirm DevTools shows lines tagged with target, level, and structured
   fields (e.g. `device.label="overlay-device-…"`).
5. **Filter works.** Append `?log=wgpu_overlay=debug` to the URL and confirm
   the per-frame `wgpu_overlay::frame` records appear; without it they do not.
6. **No regressions.** WgpuOverlay still bootstraps, Graph3D still renders,
   and the smoke effect remains visible after SPA navigation
   `/specs → /specs/graph → /specs` (verified by Playwright screenshot, same
   procedure used in commit `d09bfe39`).
7. **Cross-target compile.** `cargo check -p viewer-api-dioxus` (native target)
   AND `trunk build --release` (WASM) both succeed.

---

## 5. References

- Diagnostic logs added in commit `d09bfe39` (the call sites this spec replaces).
- `crates/context-trace/src/lib.rs` — example of structured tracing patterns
  used elsewhere in the codebase.
- `tools/viewer/log-viewer` — the eventual consumer of WASM logs once the
  file-sink spec ships.

---

## 6. End-to-end tests

The acceptance criteria for this specification are exercised by the Playwright
E2E suite under `tools/viewer/e2e`.

**Test file:** `tests/viewers/tracing.spec.ts`

| Test name | Acceptance criterion |
|-----------|----------------------|
| `startup: tracing subscriber emits first record to browser console` | AC #4 — structured records visible in DevTools |
| `default: no POST to /api/client-log without log_sink flag` | AC #4 — console-only by default; no leakage to network |
| `?log_sink=on: network layer posts batched records to /api/client-log` | companion spec AC #3 — opt-in network sink works |
| `localStorage opt-in: network layer activates when viewer-api-log-sink=on` | companion spec AC #3 — localStorage opt-in |
| `?log=off: filter blocks all events from reaching the network layer` | AC #5 — filter applied before any layer receives events |
| `localStorage filter: viewer-api-log-filter overrides default level` | AC #5 — localStorage filter resolution |

Registered for: `spec-viewer` (port 4002) and `ticket-viewer` (port 3002).

```sh
cd tools/viewer/e2e
npx playwright test tests/viewers/tracing.spec.ts
```
