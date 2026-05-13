# viewer-api: auth + middleware + error mapping

Canonical specification for the `viewer-api::auth`, `viewer-api::middleware`,
and `viewer-api::error` modules — the security and error-translation stack
used by every viewer.

## Public surface

- `auth::require_bearer(header_value: &str) -> Result<Claims, AuthError>`
- `middleware::tracing_layer()` — request id + latency span layer.
- `middleware::auth_layer(token: SharedString)` — extracts `Authorization:
  Bearer <token>` and returns `401 Unauthorized` on mismatch.
- `error::ApiError` enum + `IntoResponse` impl mapping each variant to a
  stable status code and JSON body `{ "error": "<code>", "message": "…" }`.

## Demo behavior

The `pages/auth_middleware.rs` page demonstrates:

1. **Token entry** — text input for the bearer token (default
   `demo-token`).
2. **Probe `/api/demo/secured`** — shows status code, `x-request-id`
   header (proving the tracing layer ran), and response body.
3. **Error gallery** — buttons that hit `/api/demo/error/<kind>` for each
   `ApiError` variant and render the resolved status + JSON.
4. **Latency badge** — pulled from the `x-response-time-ms` header set by
   the tracing layer.

## Acceptance behavior (validated by e2e)

- Without a bearer header, `/api/demo/secured` returns `401`.
- With `Authorization: Bearer demo-token`, returns `200` and the `x-request-id`
  header is a non-empty UUID.
- `/api/demo/error/not_found` returns `404` + `{ "error": "not_found", … }`.
- `/api/demo/error/internal` returns `500` + `{ "error": "internal", … }`.
- All responses carry a `x-request-id` header.

## Code references

- `tools/viewer/viewer-api/src/auth.rs`
- `tools/viewer/viewer-api/src/middleware.rs`
- `tools/viewer/viewer-api/src/error.rs`
- `tools/viewer/e2e/tests/demo-viewer/auth-middleware.spec.ts`
