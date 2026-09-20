Adopt the existing shared `TreeNode::tooltip_render` capability in current Dioxus tree consumers so the shared tree surface actually exposes richer doc-viewer-style metadata.

Scope:
- populate rich tooltip renderers in `spec-viewer` tree nodes,
- populate rich tooltip renderers in Dioxus `doc-viewer` tree nodes,
- add a minimal demo tree tooltip example when it helps showcase the shared tree capability,
- avoid widening into unrelated sidebar, tabs, or routing refactors.

Validation target:
- cargo check for viewer-api-dioxus, spec-viewer-dioxus, and the Dioxus doc-viewer frontend,
- focused external Chromium checks for tooltip visibility on the changed trees.

Implementation update:
- Added rich tooltip renderers to `spec-viewer` spec leaves so hover cards expose slug, component, state, and spec id.
- Added rich tooltip renderers to Dioxus `doc-viewer` package and artifact rows so hover cards expose target counts, artifact kinds, and HTML / rustdoc JSON availability.
- Added minimal rich tooltip examples to the shared `viewer-api-dioxus` demo tree and updated the shared tree spec traceability.
- Added the standard `#main` mount root to the shared `viewer-api-dioxus` demo HTML so the tooltip showcase mounts reliably under Trunk.

Validation status:
- Passed: `cargo check --target wasm32-unknown-unknown -p viewer-api-dioxus`
- Passed: `cargo check --target wasm32-unknown-unknown -p spec-viewer-dioxus`
- Passed: `cargo check --manifest-path tools/viewer/doc-viewer/frontend/dioxus/Cargo.toml --target wasm32-unknown-unknown`
- Passed: focused Chromium probe against managed `spec-viewer` at `1440x900`, validating the rich leaf tooltip after expanding the first component folder.
- Passed: focused Chromium probe against managed `doc-viewer` at `1440x900`, validating rich package and artifact tooltips.
- Passed: focused Chromium probe against the Trunk-served `viewer-api-dioxus` demo at `1440x900`, validating both directory and leaf rich tooltips after restoring the standard `#main` mount root.