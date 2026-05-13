# Feature page — tracing

Implement the demo page that showcases the WASM-tracing pipeline
(`viewer-api/tracing` and child `viewer-api/tracing/file-sink`).

## Scope

- Demonstrate that `tracing::info!` / `warn!` / `error!` from WASM reach
  the server-side log files via the `client_log` route (cross-feature
  with `viewer-api/client-log`).
- Render the current tracing config (env filter, batch size, file sink
  enabled?) read from a new `GET /api/demo/tracing/config` endpoint.

## Frontend

- Module: `frontend/dioxus/src/pages/tracing.rs`.
- Buttons for each log level emit one event with a structured field set
  (`{ "demo.kind": "manual" }`).
- Live tail of the last 50 events received by the server (reuse the
  client-log tail endpoint).
- Spec link to spec-viewer for `viewer-api/tracing`.

## Acceptance criteria

- Each level button results in a server-side log entry with the matching
  level and the expected field.
- The config view matches the actual server-side filter.

## E2E test

- File: `tools/viewer/e2e/tests/demo-viewer/tracing.spec.ts`.
- Register on the spec:
  `spec refs add b06c9df8-2866-433a-af73-ae9b1f4a0f0a --path tools/viewer/e2e/tests/demo-viewer/tracing.spec.ts --kind test`.

## Validation

- E2E green.
- Manual: open the page, click each button, verify entries in
  `target/test-logs/` (or wherever the file sink writes) using
  `mcp_log-viewer-mc_query_logs`.
