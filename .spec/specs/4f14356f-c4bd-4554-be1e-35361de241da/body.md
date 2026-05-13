# viewer-api: Graph3D component

Canonical specification for the shared 3D dependency-graph Dioxus component
(`memory-viewers/viewer-api/viewer-api/frontend/dioxus/src/graph3d/`) used by
ticket-viewer and spec-viewer.

## Public surface

- `Graph3D { layout, children, container_id?, container_style?, on_layout_change?, initial_camera?, on_camera_change?, selected_node_id?, hovered_node_id?, camera_command?, camera_command_seq?, projection?, layout_mode?, on_layout_mode_change?, on_projection_change? }`.
- `selected_node_id` is the committed focus anchor. Incident edges widen slightly,
  switch to the selected focus palette, and render directional packet motion;
  unrelated edges dim.
- `hovered_node_id` is the transient focus anchor used when there is no active
  selection. Incident edges brighten with a cooler focus tint while unrelated
  edges dim.
- `graph3d::can_use_webgpu() -> bool` — runtime probe.
- `interaction::install(container_id, state)` — mouse handlers (orbit /
  pan / zoom / right-button-drag with contextmenu suppression).
- Node DOM: caller-supplied cards tagged with `data-node-idx="<i>"` are
  positioned per-frame inside the Graph3D container. Callers may add
  `node-card-selected` to the active card to keep it in the foreground.

## Demo behavior

The `pages/graph3d.rs` page renders a hard-coded 12-node / 18-edge sample
graph fetched from `/api/demo/graph`:

1. Cards are HTML (Dioxus) overlays positioned by the per-frame callback.
2. Pointer interactions:
   - Left drag: orbit camera.
   - Shift+left drag or right drag: pan.
   - Wheel: zoom.
   - Right-button drag must NOT open the browser context menu.
3. Focus visuals:
   - Hovered nodes may pass `hovered_node_id` to produce transient edge focus.
   - Selected nodes may pass `selected_node_id` to produce committed edge focus,
     directional packet motion, and a foreground card state.
4. A "fit camera" button resets the view.
5. WebGPU-unavailable fallback renders an SVG version of the same graph.

## Acceptance behavior

### E2E-covered

- The graph mounts with WebGPU available; ≥1 card is positioned with
  `display: block`.
- Right-button drag fires `contextmenu` with `defaultPrevented === true`
  (regression coverage for the existing fix in
  `memory-viewers/viewer-api/viewer-api/frontend/dioxus/src/graph3d/interaction.rs`).
- Plain right-click leaves `defaultPrevented === false`.
- In spec-viewer `/specs/graph`, clicking a visible graph card opens the preview
  panel and marks that card with `node-card-selected`.
- Without WebGPU (no flags), the SVG fallback renders ≥1 `<line>`
  representing an edge.

### Browser spot-check

- Hovering a graph card applies transient edge emphasis: incident edges brighten
  with the hover palette and unrelated edges dim.
- Selecting a graph card promotes that emphasis to the selected palette, keeps
  unrelated edges dimmed, and adds animated directional packets along the
  focused edges.

## Code references

- `memory-viewers/viewer-api/viewer-api/frontend/dioxus/src/graph3d/`
- `tools/viewer/e2e/tests/dioxus/graph3d-right-drag.spec.ts`
- `tools/viewer/e2e/tests/spec-viewer/graph-selection.spec.ts`
