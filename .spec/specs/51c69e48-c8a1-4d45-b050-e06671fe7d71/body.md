# viewer-api: session

Canonical specification for `viewer-api::session` — the lightweight server
session helper (cookie-backed UUID + per-session key/value store) used by
viewers to persist non-sensitive user state across reloads.

## Public surface

- `session::SessionId(Uuid)` newtype + `Display`.
- `session::SessionStore` (`Arc<RwLock<HashMap<SessionId, SessionData>>>`).
- `session::session_layer(SessionStore)` — Axum middleware that resolves /
  creates a `SessionId` from the `viewer-session` cookie.
- `axum::Extension<SessionId>` extractor for handlers.
- `session::SessionData::get/set/remove/clear`.

## Demo behavior

The `pages/session.rs` page shows:

1. The current `SessionId` (read from `/api/demo/session`).
2. A small key/value editor that stores values into the session.
3. A "New session" button that clears the cookie and reloads.
4. A round-trip indicator: change a value, reload the page, verify the
   value persists; clear the cookie, reload, verify the session id changes.

## Acceptance behavior (validated by e2e)

- First request to `/api/demo/session` sets a `viewer-session` cookie.
- Subsequent requests with that cookie return the same `session_id`.
- `POST /api/demo/session/kv` round-trips a key/value pair.
- Clearing the cookie produces a new `session_id` on the next request.

## Code references

- `tools/viewer/viewer-api/src/session.rs`
- `tools/viewer/e2e/tests/demo-viewer/session.spec.ts`
