# Scaffold

Create the demo-viewer crate skeleton inside the `viewer-api` workspace
(per the umbrella spec `viewer-api/demo-viewer`).

## Files to create

- `tools/viewer/viewer-api/examples/demo-viewer/Cargo.toml`
  - `[package]` name = `demo-viewer`, edition `2021`.
  - `[[bin]]` name = `demo-viewer`, path = `src/main.rs`.
  - dependencies: `viewer-api`, `axum`, `tokio` (full), `serde`, `serde_json`,
    `tracing`, `uuid`, `tower-http`.
- `tools/viewer/viewer-api/examples/demo-viewer/src/main.rs`
  - `ServerConfig::new("demo-viewer", 3099).with_static_dir(…)`.
  - `run_server(config, AppState, create_router, None)`.
  - `--demo-token` CLI flag (default `demo-token`).
- `tools/viewer/viewer-api/examples/demo-viewer/src/state.rs` — `AppState`.
- `tools/viewer/viewer-api/examples/demo-viewer/src/routes.rs` — empty
  `create_router(state, static_dir) -> Router` returning a router with the
  shared layers (CORS, tracing, auth) and `with_static_files`.
- `tools/viewer/viewer-api/examples/demo-viewer/frontend/dioxus/`
  - mirror of `ticket-viewer/frontend/dioxus` minimal layout: `Cargo.toml`,
    `Trunk.toml`, `index.html`, `public/` (copy `viewer-api.css`), `src/`.
  - `src/main.rs`, `src/lib.rs`, `src/routes.rs` (Dioxus Router with one
    route per feature page), `src/pages/` with stub `mod.rs` re-exports.
- `tools/viewer/viewer-api/examples/demo-viewer/README.md` — how to run.
- Wire into root `Cargo.toml` workspace members.
- Add a `viewer-ctl.toml` entry for `demo-viewer` (port 3099, prepare +
  start commands mirroring `ticket-viewer`).

## Acceptance criteria

- `cargo build -p demo-viewer` succeeds.
- `viewer-ctl prepare demo-viewer` builds and installs the SPA to
  `~/.context-engine/static/demo-viewer`.
- `viewer-ctl start demo-viewer` listens on `:3099` and serves an empty
  Dioxus shell with the left navigation listing every feature page slug
  from the umbrella spec.
- `GET /api/demo/health` returns `{"name":"demo-viewer","port":3099}`.

## Validation

- Manual: open `http://localhost:3099/`, see the navigation.
- E2E (added by ticket `e2e-infra`): smoke test that hits `/api/demo/health`
  and asserts every nav entry is rendered.
