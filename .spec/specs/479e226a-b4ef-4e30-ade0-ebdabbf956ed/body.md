# viewer-api: WASM tracing file sink

Specification for shipping `tracing` records emitted by the Dioxus WASM
frontend (see [`viewer-api/tracing`](../tracing)) to a server-side endpoint
that writes them to a file on disk.

Tracking ticket: `8f349d96-a307-400b-a90e-3aceb2250166`.
Parent ticket: `b480632a-8605-4b5b-a4e8-f2988b2565a0`.
Parent spec: [`viewer-api/tracing`](../tracing) — installs the in-browser
subscriber that this spec extends with a network layer.

This spec is **a strict extension** of the parent: it MUST NOT be implemented
until the parent spec is shipped and the structured records exist in-browser.

---

## 1. Motivation

In-browser console logging is ephemeral — it is lost on tab close, hard to
share, and impossible to grep across multiple sessions. A persisted log file
on the server side enables:

- Post-mortem debugging of WebGPU and rendering glitches without asking users
  to re-paste console output.
- Cross-correlation with server-side `tracing` records (already persisted by
  `crates/context-trace` / `tools/viewer/log-viewer`).
- Long-running smoke tests where the browser tab may close before the operator
  inspects the logs.

---

## 2. Scope

### 2.1 New endpoint

`tools/viewer/viewer-api/src/` MUST expose:

```
POST /api/log
Content-Type: application/json
Body: { "records": [ { ...record... }, ... ] }
```

- The handler appends one JSON-Lines record per element in `records` to a
  configurable file path.
- Default file path: `<viewer-api-data-dir>/logs/frontend.jsonl`.
- The file is opened in append mode and rotated by an external tool (no
  in-process rotation is required for this spec).
- The endpoint MUST be limited to `127.0.0.1` connections by default.
  Cross-origin requests from a non-loopback origin MUST be rejected with `403`.

### 2.2 Record schema

Each record MUST contain at least these fields, with these names exactly:

| Field         | Type           | Notes                                          |
|---------------|----------------|------------------------------------------------|
| `ts`          | RFC3339 string | Browser-side timestamp (`Date.now()` formatted).|
| `level`       | string         | One of `trace`, `debug`, `info`, `warn`, `error`. |
| `target`      | string         | Tracing target (e.g. `wgpu_overlay::frame`).   |
| `message`     | string         | Formatted message body.                        |
| `fields`      | object         | All structured fields (`device.label`, `frame.n`, ...). |
| `span`        | object?        | Current span name + fields, if any.            |
| `viewer`      | string         | Viewer identifier (`spec-viewer`, `doc-viewer`, ...). |
| `session_id`  | string         | Random per-tab UUID (kept in `sessionStorage`).|

Server-side, the handler MAY enrich records with `received_ts` and the client
IP, but MUST NOT mutate any of the above fields.

### 2.3 Browser-side layer

A new `viewer_api_dioxus::tracing_setup::network_layer` MUST add a
`tracing_subscriber::Layer` that:

1. Buffers records in memory (`Vec<serde_json::Value>`).
2. Flushes the buffer to `POST /api/log` every **2 seconds** OR when the
   buffer reaches **64 records**, whichever happens first.
3. Schedules a final synchronous-best-effort flush on
   `window.beforeunload` (using `navigator.sendBeacon`).
4. Drops records silently if the network layer cannot reach the server (no
   user-visible errors). A periodic warning (`tracing::warn!`) MAY be emitted
   on the console layer once per minute when records are being dropped.

The network layer is **opt-in**: it is only registered when

```
?log_sink=on
```

is present in the URL, OR `localStorage["viewer-api-log-sink"] === "on"`.
Default behaviour MUST remain console-only to avoid surprise PII leaks.

### 2.4 Configuration

`viewer-ctl.toml` MUST gain a per-server option:

```toml
[[server.frontend_logging]]
enabled = false                     # default OFF
file_path = "logs/frontend.jsonl"   # relative to the viewer's data dir
max_body_bytes = 1_048_576          # 1 MiB POST limit
```

The HTTP handler MUST refuse requests whose body exceeds `max_body_bytes`.

### 2.5 Out of scope

- Log rotation, retention, compression — handled by external ops tooling.
- Sampling, head/tail-based.
- Cross-tab deduplication.
- Authentication beyond the loopback restriction.
- Real-time streaming to `log-viewer` UI (a future spec may bridge the
  written `frontend.jsonl` into the existing log-viewer pipeline).

---

## 3. Design

### 3.1 Server-side handler

New file: `tools/viewer/viewer-api/src/routes/log_sink.rs`.

Behavior:

```rust
#[tracing::instrument(skip(state, body))]
pub async fn ingest(
    State(state): State<AppState>,
    body: Bytes,
) -> Result<StatusCode, (StatusCode, String)> {
    if !state.frontend_logging.enabled { return Err((StatusCode::NOT_FOUND, ...)); }
    if body.len() > state.frontend_logging.max_body_bytes { return Err((StatusCode::PAYLOAD_TOO_LARGE, ...)); }
    let payload: IngestPayload = serde_json::from_slice(&body)?;
    state.frontend_log_writer.write_records(&payload.records).await?;
    Ok(StatusCode::NO_CONTENT)
}
```

The `frontend_log_writer` is a small actor wrapping `BufWriter<File>` with
an `mpsc::Sender<Vec<Record>>` channel; the writer task does the actual disk
I/O so the handler returns quickly.

### 3.2 Browser-side layer

New file: `tools/viewer/viewer-api/frontend/dioxus/src/tracing_setup/network_layer.rs`.

The layer implements `tracing_subscriber::Layer<S>` with:

- `on_event` — serialise the event into the schema in §2.2 and push into the
  shared `Mutex<Vec<Value>>`.
- A `wasm_bindgen_futures::spawn_local` task that wakes every 2s (via
  `gloo_timers::future::sleep`) to drain the buffer.
- A `beforeunload` handler installed via `web_sys::window().add_event_listener_…`
  that calls `navigator.sendBeacon("/api/log", payload)` for the final flush.

### 3.3 Wire format example

```json
{
  "records": [
    {
      "ts": "2026-04-22T22:55:13.412Z",
      "level": "info",
      "target": "wgpu_overlay::init",
      "message": "device created",
      "fields": { "device.label": "overlay-device-1745370913401" },
      "span": { "name": "wgpu_overlay::bootstrap", "fields": {} },
      "viewer": "spec-viewer",
      "session_id": "f6b6c1c8-…"
    }
  ]
}
```

---

## 4. Acceptance Criteria

A change is considered complete when ALL of the following hold:

1. **Endpoint exists.** `POST /api/log` returns `204 No Content` for a valid
   single-record payload from `127.0.0.1`, and `403` from a non-loopback origin.
2. **File created.** After a single successful POST, the configured file path
   exists and contains exactly one JSON-Lines record matching the schema.
3. **Browser layer opt-in.** Without `?log_sink=on`, no `POST /api/log`
   requests appear in the browser network tab. With `?log_sink=on`, batched
   requests appear at most every 2s.
4. **Beforeunload flush.** Closing the tab while pending records exist results
   in those records being persisted (verified by counting records before close
   and after re-opening the file).
5. **Backpressure-safe.** Killing the viewer-api server, then refreshing the
   tab, then restarting the server, MUST NOT crash the WASM frontend; dropped
   records are reported on the console layer at most once per minute.
6. **Body limit enforced.** A POST exceeding `max_body_bytes` is rejected with
   `413 Payload Too Large`.
7. **Cross-target compile.** `cargo check -p viewer-api` (native) AND
   `trunk build --release` (WASM) both succeed.

---

## 5. References

- Parent spec [`viewer-api/tracing`](../tracing).
- Server-side tracing infrastructure in `crates/context-trace`.
- `tools/viewer/log-viewer` for the format conventions JSON-Lines records
  should follow so they remain queryable by the existing JQ-based tooling.

---

## 6. End-to-end tests

The acceptance criteria for this specification are exercised by the Playwright
E2E suite under `tools/viewer/e2e`.

**Test file:** `tests/viewers/tracing.spec.ts`

### Browser-side (suite: `tracing-console-suite`)

Registered for `spec-viewer` and `ticket-viewer`.

| Test name | Acceptance criterion |
|-----------|----------------------|
| `default: no POST to /api/client-log without log_sink flag` | AC #3 — no traffic when disabled |
| `?log_sink=on: network layer posts batched records to /api/client-log` | AC #3 — traffic present when enabled; payload is `{"records":[...]}` |
| `localStorage opt-in: network layer activates when viewer-api-log-sink=on` | AC #3 — localStorage trigger |
| `?log=off: filter blocks all events from reaching the network layer` | AC #3 — filter interaction with sink |

### Server endpoint (suite: `client-log-api-suite`)

Registered for `spec-viewer` and `ticket-viewer`.

| Test name | Acceptance criterion |
|-----------|----------------------|
| `valid payload returns 204 No Content` | AC #1 — endpoint exists and accepts records |
| `empty records array returns 204 No Content` | AC #1 — degenerate case handled |
| `batch of multiple records returns 204 No Content` | AC #1 — batch ingest works |
| `malformed JSON body returns 422 Unprocessable Entity` | AC #1 — invalid payload rejected |
| `JSON missing "records" field returns 422 Unprocessable Entity` | AC #1 — schema validated |
| `body exceeding 1 MiB returns 413 Payload Too Large` | AC #6 — body size limit enforced |

```sh
cd tools/viewer/e2e
npx playwright test tests/viewers/tracing.spec.ts
```
