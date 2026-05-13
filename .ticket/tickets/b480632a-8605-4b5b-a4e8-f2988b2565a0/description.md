Replace ad-hoc `web_sys::console::log_1!()` calls in the Dioxus WASM frontend (viewer-api, spec-viewer, ticket-viewer) with structured tracing.

## Motivation

Recent WgpuOverlay/Graph3D shared-GPU refactor (commit `d09bfe39`) added several diagnostic `console.log` calls to debug a cross-device WebGPU issue. They are useful but unstructured: no levels, no spans, no filtering, no persistence.

The codebase already uses the `tracing` crate extensively on the Rust/native side. The WASM frontend should follow the same pattern.

## Scope

- Add `tracing` + `tracing-wasm` (or `tracing-web`) dependencies to `tools/viewer/viewer-api/frontend/dioxus`.
- Wire a tracing subscriber at app bootstrap (`lib.rs` / `main.rs`) that writes to the browser console.
- Migrate existing `console::log_1!()` / `console::error_1!()` calls (especially in `effects/wgpu_overlay/*` and `graph3d/*`) to tracing macros (`info!`, `warn!`, `error!`, `debug!`) with structured fields (e.g. `device.label`, `frame.n`).
- Use spans for grouped operations (overlay bootstrap, graph3d init, per-frame render).
- Make level filtering configurable at runtime (e.g. via a query-string param or `localStorage` key).

## Out of Scope (separate ticket)

- File sink: shipping logs to a server endpoint for persistence.

## Acceptance Criteria

- All `console::log_1!()` / `console::error_1!()` in `viewer-api/frontend/dioxus` replaced with tracing macros.
- Browser DevTools shows structured log lines with level, target, and fields.
- Default log level is `INFO`; can be raised to `DEBUG` via a documented mechanism.
- No regressions: WgpuOverlay still bootstraps, Graph3D still renders, smoke effect still visible after SPA navigation.
