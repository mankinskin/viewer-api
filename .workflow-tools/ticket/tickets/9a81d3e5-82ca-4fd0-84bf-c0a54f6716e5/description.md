Reuse the shared toggle button contract for FileTree sort controls so explorer sort rows stop duplicating active/inactive button markup.

Scope:
- adopt the shared toggle button contract in FileTree sort controls,
- preserve current sort classes and labels,
- keep the demo tree-view showcase and spec-viewer tree sort surface on the shared implementation,
- avoid changing sort semantics or broadening to unrelated toolbar actions.

Validation target:
- cargo check for viewer-api-dioxus, spec-viewer-dioxus, and ticket-viewer-dioxus,
- focused browser validation against a consumer rendering the FileTree sort row.

Implementation update:
- Reused the shared `FilterToggleButton` contract in `FileTree` sort controls so explorer sort rows no longer keep a second inline active/inactive button implementation.
- Preserved the current sort labels, classes, and arrow metadata while routing sort interactions through the shared toggle surface.
- Kept the demo tree-view showcase and `spec-viewer` tree sort surface on the shared `FileTree` implementation.

Validation status:
- Passed: `cargo check --target wasm32-unknown-unknown -p viewer-api-dioxus`
- Passed: `cargo check --target wasm32-unknown-unknown -p spec-viewer-dioxus`
- Passed: `cargo check --target wasm32-unknown-unknown -p ticket-viewer-dioxus`
- Passed: focused Chromium probe against managed `spec-viewer` at `1440x900`, validating that the `FileTree` sort row renders with the shared toggle contract, active class, and sort title metadata.