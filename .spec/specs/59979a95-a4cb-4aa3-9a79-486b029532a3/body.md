<!-- aligned-structure:v1 -->

# Summary

Canonical specification for the **HTTP server bootstrap** primitives exported by `viewer-api`: `ServerConfig`, `ServerArgs`, `run_server`, `default_cors`, `with_static_files`, `init_tracing`, and the `--http`/`--mcp`/`--port` CLI contract.

## Behavior Story

Canonical specification for the **HTTP server bootstrap** primitives exported by `viewer-api`: `ServerConfig`, `ServerArgs`, `run_server`, `default_cors`, `with_static_files`, `init_tracing`, and the `--http`/`--mcp`/`--port` CLI contract.

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# viewer-api: server-infra

Canonical specification for the **HTTP server bootstrap** primitives exported
by `viewer-api`: `ServerConfig`, `ServerArgs`, `run_server`, `default_cors`,
`with_static_files`, `init_tracing`, and the `--http`/`--mcp`/`--port` CLI
contract.

The demo-viewer (`viewer-api/demo-viewer`) is the reference consumer. Every
viewer in the workspace bootstraps its HTTP listener in the same way; this
spec captures the canonical sequence so deviations are caught by the
demo-viewer e2e suite.

## Public surface

- `ServerConfig::new(name, port).with_host(..).with_static_dir(..).with_workspace_root(..)`
- `ServerArgs::parse() -> { http: bool, mcp: bool, port: Option<u16> }`
  with defaults: `--http` when no flag is given, `--port` overrides
  `ServerConfig::port`.
- `run_server(config, state, create_router, mcp_factory).await`
- `default_cors() -> CorsLayer` — permissive CORS suitable for local dev.
- `with_static_files(router, Some(dir)) -> Router` — serves SPA + falls
  back to `index.html` for client-side routes.
- `init_tracing(level)` — sets up env-filtered fmt subscriber.
- `to_unix_path(&Path) -> String` — forward-slash normalised path.

## Demo behavior

The demo-viewer's `pages/server_infra.rs` page renders:

1. The current server config (name, host, port, static dir) read from
   `/api/demo/health`.
2. A live test of `--port`: clicking "Restart on :3199" via viewer-ctl
   restarts the demo-viewer on the new port and the page re-resolves.
3. A static-files probe: shows that an asset under `public/` is served and
   that an unknown SPA route falls back to `index.html`.
4. CORS verification: a button issues a cross-origin XHR from a `data:`
   iframe and prints the resolved `Access-Control-Allow-Origin`.

## Acceptance behavior (validated by e2e)

- `GET /api/demo/health` returns `{ "name": "demo-viewer", "port": <u16> }`.
- `GET /unknown/spa/route` returns the SPA `index.html` (status 200).
- `OPTIONS /api/demo/health` returns CORS headers permitting `*`.
- `--port 3199` is honoured (override of the `ServerConfig` default).

## Anti-patterns (do not copy)

- Constructing an `axum::Router` and binding `tokio::net::TcpListener`
  directly — viewers must go through `run_server` so static files, CORS,
  and tracing are set up identically.

## Code references

- `tools/viewer/viewer-api/src/lib.rs` (`run_server`, `ServerConfig`)
- `tools/viewer/viewer-api/examples/demo-viewer/src/main.rs`
- `tools/viewer/e2e/tests/demo-viewer/server-infra.spec.ts`
