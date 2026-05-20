Replace the Dioxus doc-viewer's ad-hoc tab-state signals with the existing shared `viewer_api_dioxus::TabsStore<OpenArtifactTab>` so the frontend actually consumes the tab-state primitive that already landed in `viewer-api-dioxus`.

Scope:
- construct `TabsStore<OpenArtifactTab>` at the app root in `tools/viewer/doc-viewer/frontend/dioxus/src/app.rs`,
- migrate open, close, activate, and full-reset flows from `Signal<Vec<OpenArtifactTab>>` plus `Signal<Option<String>>` to shared store methods,
- preserve existing first-artifact auto-open, active-tab fallback, and JSON fetch behavior,
- keep the current `TabBar` UI contract and artifact payload shape stable,
- keep any demo-viewer linkage limited to showcasing the shared tab/store primitives rather than widening into new layout work.

Validation target:
- `cargo check --target wasm32-unknown-unknown -p viewer-api-dioxus`,
- `cargo check --manifest-path tools/viewer/doc-viewer/frontend/dioxus/Cargo.toml --target wasm32-unknown-unknown`,
- focused external Chromium validation of doc-viewer tab open, close, and reselect behavior.

Implementation update:
- Replaced the Dioxus doc-viewer `open_tabs` and `active_tab_id` signal pair with a shared `TabsStore<OpenArtifactTab>` mounted via `use_hook` at the app root.
- Migrated tab open, activate, close, JSON-fetch, active-view, and full-reset flows to operate on the shared store while keeping the existing `TabBar` UI and payload shape unchanged.
- Preserved the current active-tab fallback behavior on close by committing the remaining tabs back through `TabsStore::set_tabs`.

Validation status:
- Passed: `cargo check --target wasm32-unknown-unknown -p viewer-api-dioxus`
- Passed: `cargo check --manifest-path tools/viewer/doc-viewer/frontend/dioxus/Cargo.toml --target wasm32-unknown-unknown`
- Passed: focused Chromium probe against managed `doc-viewer` at `1440x900`, validating tree-driven tab open, tab reselection, and closing the active tab after the `TabsStore` migration.
- Note: the current workspace data did not present an initially auto-opened renderable artifact, so browser validation exercised the migrated tab state by opening artifacts from the tree explicitly.
