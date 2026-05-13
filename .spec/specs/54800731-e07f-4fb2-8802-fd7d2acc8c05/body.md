# viewer-api: SSE streaming

Canonical specification for `viewer-api::sse` — the server-sent-events
helper used by every viewer that pushes live data (log tail, ticket board
events, spec changes).

## Public surface

- `sse::sse_response<S>(stream: S) -> Sse<…>` where
  `S: Stream<Item = Result<sse::Event, ApiError>>`.
- `sse::keepalive()` — 15 s default keep-alive.
- `sse::Event::data(payload)` / `Event::named(event, payload)` /
  `Event::with_id(id, ..)` re-exports.

## Demo behavior

The `pages/sse.rs` page demonstrates:

1. A live counter that ticks once per second from `/api/demo/sse/stream`,
   showing the EventSource state (`CONNECTING|OPEN|CLOSED`).
2. A start/stop toggle that opens / closes the underlying `EventSource`.
3. A "burst" button that triggers the server to emit 10 events back-to-back.
4. A reconnection demo: kill the connection from the browser DevTools and
   confirm the client auto-reconnects (browser-native behavior).

## Acceptance behavior (validated by e2e)

- Opening `/api/demo/sse/stream` yields at least 3 `data:` events within 4 s.
- Each event carries a monotonically increasing `id:` field.
- A keep-alive comment (`:`) is observed within 16 s of the last event.
- Closing the EventSource client-side terminates the response on the server
  (verified via a follow-up `/api/demo/sse/stats` endpoint that reports
  `active_streams = 0`).

## Code references

- `tools/viewer/viewer-api/src/sse.rs`
- `tools/viewer/e2e/tests/demo-viewer/sse.spec.ts`
