Add reusable state containers to `viewer-api-dioxus` for cross-cutting viewer concerns.

## Deliverables

1. `store/tabs.rs`
   - `TabsStore<T>` with signals `tabs: Vec<Tab<T>>` and `active: Option<String>`
   - Methods: `open(id, payload)`, `close(id)`, `activate(id)`, `set_tabs(...)`, `active_tab()`
   - Optional `bind_url(UrlStateManager<String>)` helper to sync the active tab id with the URL hash
   - Risk: Dioxus `Signal` + generics; if blocked, fall back to `TabsStoreInner` (non-generic, payloads as `Rc<dyn Any>`) with thin generic facade

2. `store/url_path.rs`
   - `PathCodec` trait: `fn encode(&self, id: &str) -> String; fn decode(&self, path: &str) -> Option<String>;`
   - Default `ColonSegmented` impl covering the spec/crate `category:name:sub/path` pattern
   - `expand_path_to(set: &mut HashSet<String>, segments: &[&str])` helper for syncing tree-view expansion to URL state

3. `store/prefetch.rs`
   - `Prefetcher<K: Hash+Eq+Clone, V: Clone>` with `with_capacity(n)` LRU eviction
   - `async fn get_or_fetch(&self, key: K, fetcher: impl FnOnce(K) -> Future<Output=Result<V,E>>) -> Result<V,E>`
   - Single-flight semantics: concurrent calls for the same key share one in-flight future

## Wiring

- Re-exported from `store/mod.rs` and `lib.rs`
- Documented in module-level rustdoc with usage examples

## Acceptance criteria

- `cargo check -p viewer-api-dioxus --target wasm32-unknown-unknown` passes
- `cargo test -p viewer-api-dioxus` for the non-WASM-bound logic (PathCodec, Prefetcher LRU) passes
- TabsStore has at least one consumer-facing usage demonstrated in spec-viewer (defer adoption to Phase 5 if necessary, but the API must be exercised by at least a unit example)
