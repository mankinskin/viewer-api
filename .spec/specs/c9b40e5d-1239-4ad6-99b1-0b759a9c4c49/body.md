# viewer-api: pagination + query helpers

Canonical specification for `viewer-api::pagination` and `viewer-api::query`
— the cursor / page parsing helpers reused by every viewer's list endpoints.

## Public surface

- `pagination::Page { cursor: Option<String>, limit: u32 }`
  with `Page::from_query(&Query<HashMap<String,String>>) -> Result<Page, ApiError>`.
- `pagination::Page::default_limit()` = 50, `max_limit()` = 500.
- `pagination::Paginated<T> { items: Vec<T>, next_cursor: Option<String>, total: Option<u64> }`.
- `query::TypedQuery<T>` — Axum extractor that deserializes the query string
  via `serde_urlencoded` and maps decode errors to
  `ApiError::BadRequest(field, msg)`.

## Demo behavior

The `pages/pagination_query.rs` page exposes:

1. A list of 50 deterministic seed items rendered in pages of 10.
2. Buttons: **Prev**, **Next**, **First**, jump-to-page selector.
3. URL `?cursor=…&limit=…` round-trips into the page state (deep-linkable).
4. A "TypedQuery" sub-section: a free-form input that hits `/api/demo/query?q=…`
   and shows the typed echo + any validation error.
5. Visible decode of the cursor (base64-decoded "offset:N").

## Acceptance behavior (validated by e2e)

- `GET /api/demo/items?limit=10` returns the first 10 of 50 items and a
  non-null `next_cursor`.
- `GET /api/demo/items?cursor=<…>&limit=10` returns the next 10 items.
- Following `next_cursor` 4 more times yields a `next_cursor: null` on page 5.
- `GET /api/demo/items?limit=9999` is clamped to `max_limit` (500) and the
  response carries `X-Pagination-Limit-Clamped: true`.
- `GET /api/demo/query?limit=abc` returns `400` with `error.code = "bad_request"`
  and `error.field = "limit"`.

## Code references

- `tools/viewer/viewer-api/src/pagination.rs`
- `tools/viewer/viewer-api/src/query.rs`
- `tools/viewer/e2e/tests/demo-viewer/pagination-query.spec.ts`
