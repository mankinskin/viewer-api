# Impl: viewer-api extraction for shared tree/file/graph server primitives

**Wave 1 / Track E** | Component: `viewer-api`

## Design inputs
- API contract: `21a1b9ca/assets/design/api-contract-v0.1.md`
- Hook emission contract: `24aa7e5e/assets/design/hook-contract-v0.1.md`
- Subgraph API semantics: `e79fdc1f/assets/design/subgraph-api-v0.1.md`

## Objective
`viewer-api` already provides HTTP/MCP server bootstrap primitives shared by
`doc-viewer` and `log-viewer`. This ticket extends it to also provide the
reusable server-side primitives that `ticket-viewer` and future viewers will
need — specifically: generic pagination types, subgraph traversal traits/types,
SSE event envelope helpers, and request-id middleware — so these are not
duplicated per-tool.

## What is already in viewer-api
- `ServerConfig`, `run_server`, `McpServerFactory`
- `dev_proxy`, `default_cors`, `with_static_files`
- `init_tracing`, arg parsing (`--http`, `--mcp`)
- Re-exports: `axum`, `tower_http`, `tokio`, `tracing`, `rmcp`

## What needs to be added / extracted

### 1. `request_id` middleware
Tower middleware layer that generates `uuid::Uuid::new_v4()` per request and
inserts it into request extensions + response header `X-Request-Id`.

```rust
// viewer_api::middleware::request_id
pub fn request_id_layer() -> RequestIdLayer { ... }
```

### 2. Pagination cursor type
Generic opaque cursor for paginated list endpoints:
```rust
// viewer_api::pagination
pub struct PageCursor(String);  // base64-encoded checkpoint
pub struct PageParams { pub limit: usize, pub cursor: Option<PageCursor> }
pub struct PageResult<T> { pub items: Vec<T>, pub next_cursor: Option<PageCursor> }
```

### 3. Standard error envelope
```rust
// viewer_api::error
#[derive(Serialize)]
pub struct ApiError {
    pub code: String,       // "auth.invalid_token"
    pub message: String,
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}
impl IntoResponse for ApiError { ... }
```

### 4. SSE helpers
Re-export and helpers around `axum::response::sse`:
```rust
// viewer_api::sse
pub fn sse_event<T: Serialize>(event_name: &str, id: u64, payload: &T)
    -> Result<axum::response::sse::Event, serde_json::Error>
```

### 5. Bearer auth middleware
Generic bearer-token Tower layer (extracted from `43dedd9b` serve auth):
```rust
// viewer_api::auth
pub struct BearerAuthLayer { ... }
pub struct TokenSet { tokens: HashSet<String> }
impl TokenSet { pub fn contains(&self, token: &str) -> bool }
```
This ensures `ticket serve` and future viewers share the same token checking logic.

## Implementation plan

### Step 1 — Review current viewer-api for conflicts
Read `tools/viewer-api/src/` to check for any existing request_id/pagination/error
implementations before creating new modules.

### Step 2 — Add modules to `tools/viewer-api/src/`
- `middleware/mod.rs` + `middleware/request_id.rs`
- `pagination.rs`
- `error.rs`
- `sse.rs`
- `auth.rs`
- Update `lib.rs` to re-export all new modules

### Step 3 — Update `tools/viewer-api/Cargo.toml`
Add deps if missing: `uuid` (with v4 feature), `base64`, `serde_json`, `tower`

### Step 4 — Update `ticket serve` implementation (`43dedd9b`)
Switch from local implementations to `viewer_api::auth::BearerAuthLayer`,
`viewer_api::error::ApiError`, `viewer_api::pagination::*`, `viewer_api::sse::*`.

### Step 5 — Check doc-viewer / log-viewer for duplication
Scan `tools/doc-viewer/src/http_server.rs` and `tools/log-viewer/src/` for any
equivalent request_id / error envelope logic. If found, migrate to viewer-api
types (no behavioral change, only re-exports).

### Step 6 — Tests
- `tools/viewer-api/src/tests/pagination.rs` — round-trip cursor encode/decode
- `tools/viewer-api/src/tests/error.rs` — `ApiError` serializes to correct shape
- `tools/viewer-api/src/tests/auth.rs` — `TokenSet::contains` correctly validates

## Acceptance criteria
- [ ] `viewer_api::auth::BearerAuthLayer` exists and used by `ticket serve`
- [ ] `viewer_api::error::ApiError` serializes to `api-contract-v0.1.md` error envelope
- [ ] `viewer_api::pagination::{PageCursor, PageParams, PageResult}` exist
- [ ] `viewer_api::sse::sse_event()` helper exists
- [ ] `viewer_api::middleware::request_id_layer()` exists
- [ ] doc-viewer/log-viewer compile cleanly; no regressions

## Dependencies / Handoff
- Used by: `43dedd9b` (ticket serve mode), `02dea1fa` (ticket-viewer shell)
- Parallel to: `43dedd9b` — coordinate on auth type shape to avoid collision
