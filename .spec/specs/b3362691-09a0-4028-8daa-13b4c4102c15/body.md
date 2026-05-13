# viewer-api: layout components

Canonical specification for the shared **page-shell** Dioxus components
exported from `viewer-api/frontend/dioxus/src/components/layout.rs` and
`resize_handle.rs`: `Layout`, `Header`, `Sidebar`, `Panel`, `GlassPanel`,
`PanelPlacement`, `ResizeHandle`, `ResizeDirection`, `ResizeEdge`.

## Public surface

- `Layout { children, header, sidebar, footer }` — flex page shell.
- `Header { title, subtitle, actions }`.
- `Sidebar { children, placement: PanelPlacement, resizable: bool, default_width: u32 }`.
- `Panel`, `GlassPanel { children, blur, opacity }` — themed containers
  driven by the theme-settings store.
- `ResizeHandle { direction, edge, on_resize }` — drag handle for splitter
  layouts; emits delta in pixels.

## Demo behavior

The `pages/layout.rs` page is itself an instance of every primitive:

1. A `Layout` with header + left `Sidebar` + main content + right
   `Sidebar` + footer.
2. The right sidebar contains a `GlassPanel` whose blur + opacity sliders
   are live-bound to `ThemeSettings`.
3. The middle `Panel` has a vertical `ResizeHandle` that resizes the left
   sidebar; a horizontal `ResizeHandle` that resizes the footer height.
4. A "reset layout" button restores defaults.

## Acceptance behavior (validated by e2e)

- The default layout renders header (≥40 px), left sidebar (≥200 px),
  main content (flex 1), right sidebar (≥200 px), footer (≥32 px).
- Dragging the vertical `ResizeHandle` 100 px right increases the left
  sidebar width by ~100 px (±2 px tolerance for snapping).
- Toggling `gpuOverlayEnabled` off makes `GlassPanel` fall back to a
  solid background (no `backdrop-filter`).
- Layout state (sidebar widths) persists across reload via `localStorage`.

## Code references

- `tools/viewer/viewer-api/frontend/dioxus/src/components/layout.rs`
- `tools/viewer/viewer-api/frontend/dioxus/src/components/resize_handle.rs`
- `tools/viewer/e2e/tests/demo-viewer/layout.spec.ts`
