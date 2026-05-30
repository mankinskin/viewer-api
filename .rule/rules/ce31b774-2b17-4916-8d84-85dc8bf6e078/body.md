# viewer-api

viewer-api is the shared runtime repository for viewer servers, viewer lifecycle control, and browser-side UI scaffolding.

## Tool Surface

| Package or surface | What it is used for | Typical entry points |
| --- | --- | --- |
| `viewer-api` | Shared HTTP and MCP runtime for viewer tools, including auth, middleware, pagination, query/session helpers, source adapters, SSE, static file serving, and tracing setup. | Rust library consumed by viewer servers |
| `viewer-ctl` | Config-driven lifecycle manager for viewer components declared in `viewer-ctl.toml`. | `viewer-ctl list`, `viewer-ctl start`, `viewer-ctl prepare` |
| `viewer-api-dioxus` | Dioxus viewer platform scaffold with the root app, WebGPU canvas, UI overlay shell, and shared frontend building blocks. | `trunk serve`, `cargo check --target wasm32-unknown-unknown -p viewer-api-dioxus` |
