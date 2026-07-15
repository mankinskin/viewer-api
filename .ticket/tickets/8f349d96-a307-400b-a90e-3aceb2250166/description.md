# Goal

Ship structured tracing records from the Dioxus WASM frontend to a bounded server-side JSONL sink so browser records are queryable alongside backend logs.

# Current implementation evidence

The repository already contains:

- a WASM `NetworkLayer` that serializes tracing events, buffers them, and posts batches to `/api/client-log`;
- a viewer-api `client_log_router` that validates payload size and appends JSONL under `target/test-logs/frontend-client.jsonl`;
- ticket-viewer and spec-viewer server integration;
- shared Playwright coverage for opt-in/opt-out behavior and endpoint payload validation.

The ticket remains open because the implementation does not yet prove the complete operational contract below. In particular, the raw network-layer request must be reconciled with the shared session header, buffering must be bounded, and final flush/query evidence must be deterministic.

# Scope

- Preserve console tracing while adding opt-in file persistence.
- Guarantee stable session correlation on every sink request.
- Bound buffered records and expose dropped-record diagnostics.
- Flush on interval, size threshold, and page/test shutdown without blocking requestAnimationFrame.
- Define redaction and payload limits.
- Make client files discoverable/queryable through log-viewer/log-MCP tooling.
- Coordinate end-to-end Playwright artifact behavior through `9202bc21` rather than duplicating its fixture work.

# Acceptance criteria

- [ ] Browser `info`/`warn`/`error` tracing records reach the configured JSONL sink within the documented flush interval.
- [ ] Persisted records carry the same `viewer-api-session-id` used by the browser test and backend requests.
- [ ] Log query tooling can select only the records for one session ID using the same structured filters as server logs.
- [ ] Disabled sink mode sends no `/api/client-log` traffic.
- [ ] Buffer size is capped; overflow policy and dropped-record count are test-covered.
- [ ] Interval, threshold, and final shutdown flush paths are test-covered.
- [ ] Sink failure is observable without recursively logging into the same failed sink.
- [ ] Sensitive fields are redacted according to a documented rule.
- [ ] A render-loop workload shows no regression beyond the budget established by `09bef250`.

# Implementation steps

1. Add failing network-layer tests for session headers and bounded buffering.
2. Reuse the shared session utility in sink requests.
3. Implement size-threshold/final flush and drop counters.
4. Validate endpoint persistence and log-query discovery.
5. Run the shared tracing suite and record test-api evidence.