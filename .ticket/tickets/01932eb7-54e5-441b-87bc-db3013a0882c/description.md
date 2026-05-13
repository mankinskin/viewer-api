## Problem

The current `viewer-api-dioxus::components::layout::{Sidebar, Panel}` primitives are flat: each viewer hardcodes a single left `Sidebar` + one optional right `Panel`. There is no support for:

- Multiple side panels stacked or split.
- User-driven splits (drag a divider between two arbitrary regions to create a new tile).
- Tab groups within a tile (cycle between, e.g., a ticket-detail view and a history view).
- Persisting the tile tree per workspace.

The user's stated goal: **side panels should always be draggable to expand; the panel system needs to be implemented by a tiling view system, with support for tabs.** Think VS Code's editor groups + sidebar/panel containers, or i3/Sway tiling.

## Acceptance criteria

1. New module `viewer-widgets::tiling` providing:
   - `TileTree` data structure: `enum Tile { Leaf(TabGroup), Split { dir: SplitDir, ratio: f64, a: Box<Tile>, b: Box<Tile> } }`.
   - `TabGroup { tabs: Vec<TabSpec>, active: usize }`.
   - `TileView` Dioxus component that renders any `TileTree` recursively with draggable splitters.
2. Drag a splitter between two leaves → live ratio update; double-click splitter → 50/50 reset.
3. Drag a tab header out of one `TabGroup` → new `Split` is created (mouse position decides direction); drag onto another tab strip → tab moves to that group.
4. Empty `TabGroup` is auto-collapsed.
5. Tile tree state is serialisable (`serde`) so callers can persist it (e.g. ticket-viewer per-workspace localStorage).
6. Backwards-compatible adapter: `Sidebar` and `Panel` keep working but are implemented on top of `TileView` internally (or are deprecated with a clear migration note).
7. New demo page in `demo-viewer` (or under `viewer-widgets/examples/`) showing the tiling system in action.
8. Playwright e2e tests for drag-to-split, drag-to-merge, tab reorder, persistence reload.

## Implementation notes

- WebGPU pointer-event interactions and the existing `ResizeHandle` component cover the splitter drag mechanics — reuse rather than rewrite.
- For tab drag-and-drop on the web, look at the HTML5 drag-and-drop API surfaced by `dioxus::prelude::Event<DragData>`.
- Consider how this interacts with the mobile breakpoint: at ≤768 px the tile tree should collapse to a single visible tab + drawer behaviour.
- Investigate whether existing crates fit (e.g. `egui_dock` is for egui only; no Dioxus equivalent known at time of writing). The user opted for "build from scratch" during refinement.

## Out of scope

- Floating/detached tile windows (browser tabs only).
- Per-tile theme overrides.

## Parent

Epic `4e2b2b0b-9f56-4786-991c-8f10e653f4c3`. Should ideally land after the crate extraction (`92964ada-4ab5-4fe1-ab29-5bfd55583ad2`) so the new code goes into `viewer-widgets` directly.

## Status

Marked `new` and explicitly **deferred** per user direction. Pick up after the smaller theme/button quick-wins land.
