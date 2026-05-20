Reuse the shared toggle button contract for FileTree sort controls so explorer sort rows stop duplicating active/inactive button markup.

Scope:
- adopt the shared toggle button contract in FileTree sort controls,
- preserve current sort classes and labels,
- keep the demo tree-view showcase and spec-viewer tree sort surface on the shared implementation,
- avoid changing sort semantics or broadening to unrelated toolbar actions.

Validation target:
- cargo check for viewer-api-dioxus, spec-viewer-dioxus, and ticket-viewer-dioxus,
- focused browser validation against a consumer rendering the FileTree sort row.