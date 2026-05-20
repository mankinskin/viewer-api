Extract a shared clickable chip button in viewer-api so explorer filter/state toggles stop duplicating button markup and state wiring across FileTree and ticket-viewer.

Scope:
- add a reusable interactive chip primitive to viewer-api-dioxus,
- adopt it in FileTree filter buttons,
- adopt it in ticket-viewer state filter chips while preserving current test ids and keyboard behavior,
- keep the existing demo tree-view surface as a showcase through FileTree.

Validation target:
- cargo check for viewer-api-dioxus, spec-viewer-dioxus, and ticket-viewer-dioxus,
- focused ticket-viewer Playwright sidebar tests for state filters and keyboard navigation.

Implementation update:
- Added shared `FilterToggleButton` to `viewer-api-dioxus` so explorer filter and state chips reuse one interactive button contract with shared active/inactive styling and `aria-pressed` behavior.
- Adopted the shared chip button in `FileTree` filter rows so the demo tree and current FileTree consumers inherit the same filter-button implementation.
- Adopted the shared chip button in `ticket-viewer` state filter chips while preserving the existing `ticket-tree-state-chip-*` test ids and sidebar keyboard flow.

Validation status:
- Passed: `cargo check --target wasm32-unknown-unknown -p viewer-api-dioxus`
- Passed: `cargo check --target wasm32-unknown-unknown -p spec-viewer-dioxus`
- Passed: `cargo check --target wasm32-unknown-unknown -p ticket-viewer-dioxus`
- Passed: `npm run test:e2e:release -- e2e-release/sidebar-query-state-filter.spec.ts e2e-release/keyboard-navigation.spec.ts -g "sidebar explorer keeps active state filter when filter text is non-empty|sidebar filter keeps focus while arrows move the active ticket and Enter selects it"` in `memory-viewers/ticket-viewer/frontend/dioxus`.
- Passed: focused Chromium probe against managed `spec-viewer` at `1440x900`, validating that a `FileTree` filter toggle becomes active after click with the shared `FilterToggleButton`.