# viewer-api: client log endpoint

Canonical specification for `viewer-api::client_log` — the server endpoint
that receives structured log events from WASM frontends and writes them to
the same tracing pipeline as backend logs.

## Public surface

- `client_log::ClientLogState { sink: Arc<dyn ClientLogSink>, max_batch: usize }`.
- `client_log::client_log_router(state) -> Router` mounting `POST /api/log/client`.
- `client_log::ClientLogEvent { timestamp, level, target, message, fields, span: Option<SpanInfo> }`.
- WASM-side helpers in `viewer-api/frontend/dioxus/src/tracing_setup/` that
  buffer events and POST batches.

## Demo behavior

The `pages/client_log.rs` page exposes:

1. Buttons that emit `tracing::trace!`, `debug!`, `info!`, `warn!`, `error!`
   from the WASM frontend.
2. A live tail of the last 50 events received by the server (rendered from
   `/api/demo/client-log/recent`).
3. A "burst 100" button that emits 100 events as fast as possible to verify
   batching + ordering.

## Acceptance behavior (validated by e2e)

- Clicking each level button results in an event of that level appearing in
  the tail within 1 s.
- A burst of 100 events is delivered with no loss (count matches; ids
  monotonic).
- Malformed payloads (`POST /api/log/client` with non-JSON) return `400`.

## Code references

- `tools/viewer/viewer-api/src/client_log.rs`
- `tools/viewer/viewer-api/frontend/dioxus/src/tracing_setup/`
- `tools/viewer/e2e/tests/demo-viewer/client-log.spec.ts`
