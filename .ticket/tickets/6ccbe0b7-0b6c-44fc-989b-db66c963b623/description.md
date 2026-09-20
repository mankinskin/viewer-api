W3. The left file-tree panel is badly formatted and expensive to render; widening the panel is sluggish and laggy.

Requirements:
- Fix formatting to match the viewer-api tree-view spec (a20a0395).
- Eliminate resize lag: decouple width changes from expensive relayout/measurement.
- Avoid per-frame DOM thrash during drag-resize.
- Bound render cost for long trees (virtualize or window the list).

Shared component lives in viewer-api and benefits spec-viewer/log-viewer too. Relates to prior FileTree work (d7971816 sortable header, c10cc92e sidebar parity).

## W3 implementation notes (current pass)
- TreeView visible-id collection now builds an O(1) expanded-id lookup (`HashSet<&str>`) to avoid repeated linear membership scans in recursive traversal.
- Tree row and label CSS now enforce `min-width: 0`, truncation-safe flex behavior, and fixed icon shrink behavior to prevent left-panel overflow churn.
- Tree list uses `content-visibility: auto` and intrinsic-size hints to reduce upfront rendering cost for long trees.
- FileTree container now uses `contain: layout paint` to localize layout/paint invalidation during panel resize.

## Validation evidence
- Validation spec id: `vt-ticket-viewer-w3-filetree-layout-performance`.
- Passed execution record: `.test/default/executions/exec-vt-ticket-viewer-w3-20260710-compile-libtest.json`.
- Blocked execution record (known unrelated doctest issue): `.test/default/executions/exec-vt-ticket-viewer-w3-20260710-doctest-blocker.json`.
- Commands captured in passed record:
  - `cargo check --manifest-path viewer-api/viewer-api/frontend/dioxus/Cargo.toml`
  - `cargo check --target wasm32-unknown-unknown --manifest-path viewer-api/viewer-api/frontend/dioxus/Cargo.toml`
  - `cargo test --lib --manifest-path viewer-api/viewer-api/frontend/dioxus/Cargo.toml`
- Full `cargo test` currently fails in an unrelated doctest macro import (`viewer-api/frontend/dioxus/src/lib.rs`, missing `classes` macro import in doctest scope).