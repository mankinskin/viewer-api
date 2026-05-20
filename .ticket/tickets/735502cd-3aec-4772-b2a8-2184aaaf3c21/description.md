Extract a shared clickable chip button in viewer-api so explorer filter/state toggles stop duplicating button markup and state wiring across FileTree and ticket-viewer.

Scope:
- add a reusable interactive chip primitive to viewer-api-dioxus,
- adopt it in FileTree filter buttons,
- adopt it in ticket-viewer state filter chips while preserving current test ids and keyboard behavior,
- keep the existing demo tree-view surface as a showcase through FileTree.

Validation target:
- cargo check for viewer-api-dioxus, spec-viewer-dioxus, and ticket-viewer-dioxus,
- focused ticket-viewer Playwright sidebar tests for state filters and keyboard navigation.