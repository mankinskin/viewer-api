Build on the structured-tracing parent ticket: add a tracing `Layer` that batches log records and POSTs them to a server endpoint (e.g. `POST /api/client-log`) which appends them to a per-session JSONL file under `target/test-logs/` (or a configurable path). This makes browser logs queryable from log-viewer MCP tools alongside server logs.

## Scope

- Add a tracing `Layer` (or wrap an existing one) that buffers records and ships them to `/api/client-log`.
- Server side: `viewer-api` or each viewer's HTTP layer accepts the POST and appends to a structured (JSONL) log file.
- Client config: enabled/disabled via a viewer config flag or query string; redact sensitive fields if any.
- Backpressure: drop or coalesce records under high volume; do not block the rAF loop.
- log-viewer MCP integration: the new client log files appear in `list_logs` / `get_log` / `query_logs`.

## Acceptance Criteria

- Browser logs (`info`/`warn`/`error` from any frontend tracing call) appear in `target/test-logs/` within ~1 s.
- log-viewer MCP can read them with the same JQ filters used for server logs.
- Disabling the feature flag eliminates all network traffic to `/api/client-log`.
- No measurable frame-time regression in WgpuOverlay (>= 60 FPS on `/specs`).
- `tracing-wasm` console output continues to work in parallel.
