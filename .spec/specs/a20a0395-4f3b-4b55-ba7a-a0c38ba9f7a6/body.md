# viewer-api: TreeView component

Canonical specification for the shared `TreeView` Dioxus component
(`viewer-api/frontend/dioxus/src/components/tree_view.rs`) — the
expandable tree used in doc-viewer (crate browser), log-viewer (file
list), spec-viewer (spec hierarchy), and ticket-viewer (ticket
hierarchy).

## Public surface

- `TreeNode { id, label, badge, tooltip, tooltip_render, icon: NodeIcon,
  children }`.
- `NodeIcon` enum (Folder / FolderOpen / File / Doc / Code / Module / etc.).
- `TreeView { nodes, selected, on_select, filters, sort_key }`.
- `FileTree` convenience constructor that builds a `TreeView` from a list of
  `(path, kind)` tuples.
- `FilterDef { id, label, predicate }`, `SortKey { Name, Kind, Custom(fn) }`.
- `ExplorerShell { search, controls, status, body, class }` for composing the
  outer explorer column around a tree or custom row body.
- `SidebarSearch { value, on_input, placeholder, hint, input_testid,
  hint_testid, on_focus, on_keydown }` for reusing sidebar search semantics
  without forcing a single tree implementation.
- `FilterToggleButton { onclick, active, class, active_class,
  inactive_class, test_id, aria_label, title, children }` for shared
  explorer filter/state toggle buttons with consumer-specific styling.

## Shared tooltip behavior

- `TreeNode.tooltip_render` provides an optional rich tooltip slot for rows
  that need structured metadata instead of a plain `title` string.
- Consumers can keep a short string `tooltip` for the native browser fallback
  while rendering a richer hover card with labels, counts, ids, or file paths.
- Shared tree consumers should prefer `tooltip_render` when the hover content
  needs multiple lines or distinct visual emphasis.

## Shared explorer shell behavior

- Consumers can share one vertical explorer layout while keeping their own
  body renderer.
- Search affordances can preserve consumer-specific placeholders, hint copy,
  test ids, and keyboard handlers through a shared sidebar search contract.
- Loading, empty, and error states render in a consistent slot between the
  shared controls and the body content.

## Shared explorer control behavior

- Explorer filter and state toggles should share one interactive button
  contract instead of duplicating button markup across `FileTree` and
  viewer-specific explorer toolbars.
- The shared toggle must preserve consumer-specific classes, test ids,
  optional icon content, and optional count badges so adopters can reuse one
  primitive without flattening their current visual design.
- Explorer sort controls should reuse the same interactive button contract as
  explorer filters so `FileTree` does not keep a second inline active/inactive
  button implementation just for sort rows.

## Demo behavior

The `pages/tree_view.rs` page renders a tree of ~80 nodes representing a
mock project layout (folders, modules, source files, doc pages):

1. Click to expand/collapse; chevron rotates.
2. Selection is highlighted; selected node id is shown in a side panel.
3. A search box filters nodes by label substring (case-insensitive).
4. Pre-defined filter chips: `code`, `docs`, `tests`, `all`.
5. Sort selector: `Name (A→Z)`, `Name (Z→A)`, `Kind`.
6. Keyboard support: ↑ / ↓ navigate, → expand, ← collapse, Enter select.

## Acceptance behavior (validated by e2e)

- The tree renders >50 nodes initially.
- Clicking a folder toggles its expansion (children visible / hidden).
- Typing "auth" in the search box reduces visible nodes to those matching.
- Selecting `tests` filter chip hides non-test nodes.
- Pressing ↓ then Enter selects the first child node and emits `on_select`.

## Implementation Status

- Implemented in `viewer-api-dioxus` as `ExplorerShell` and `SidebarSearch`.
- Adopted by `spec-viewer` `SpecTree` to share the search/status/body shell
  around `FileTree` while preserving state-filter chips and title sorting.
- Adopted by `ticket-viewer` `TicketTree` to share the search/controls/status
  shell while preserving custom rows, state chips, selection controls, and
  keyboard navigation.
- Implemented in `viewer-api-dioxus` as `FilterToggleButton` for shared
  explorer filter and state toggle buttons.
- Adopted by `FileTree` filter rows, which automatically updates the demo
  tree-view showcase plus current `spec-viewer` and `log-viewer` file-tree
  consumers.
- Adopted by `FileTree` sort rows, which now reuse the same shared toggle
  contract instead of keeping a second inline active/inactive button
  implementation.
- Adopted by `ticket-viewer` state filter chips while preserving the existing
  `ticket-tree-state-chip-*` test ids and sidebar keyboard flow.
- Adopted by `spec-viewer` spec leaves for richer hover metadata including
  slug, component, state, and spec id.
- Adopted by `doc-viewer` package and artifact rows for richer hover metadata
  including artifact counts, target kinds, and HTML / rustdoc JSON presence.
- Showcased by the shared `viewer-api-dioxus` demo tree with tooltip examples
  on both directories and leaf nodes, with the demo mounting through the same
  `#main` root container pattern as the working viewer frontends.

## Traceability

- Related ticket: `memory-viewers/viewer-api/.ticket/tickets/763f8c13-a4bd-47af-8894-3e95a63fde8d`
- Related ticket: `memory-viewers/viewer-api/.ticket/tickets/735502cd-3aec-4772-b2a8-2184aaaf3c21`
- Related ticket: `memory-viewers/viewer-api/.ticket/tickets/9a81d3e5-82ca-4fd0-84bf-c0a54f6716e5`
- Related ticket: `memory-viewers/viewer-api/.ticket/tickets/21a2e8f4-4bd8-4436-be52-c2c4a07bb692`
- Updated doc: `memory-viewers/viewer-api/.spec/specs/a20a0395-4f3b-4b55-ba7a-a0c38ba9f7a6/body.md`

## Validation

- `cargo check --target wasm32-unknown-unknown -p viewer-api-dioxus`
- `cargo check --target wasm32-unknown-unknown -p spec-viewer-dioxus`
- `cargo check --target wasm32-unknown-unknown -p ticket-viewer-dioxus`
- `cargo check --manifest-path tools/viewer/doc-viewer/frontend/dioxus/Cargo.toml --target wasm32-unknown-unknown`
- `npm run test:e2e:release -- e2e-release/sidebar-query-state-filter.spec.ts e2e-release/keyboard-navigation.spec.ts -g "sidebar explorer keeps active state filter when filter text is non-empty|sidebar filter keeps focus while arrows move the active ticket and Enter selects it"` in `memory-viewers/ticket-viewer/frontend/dioxus`
- Focused Chromium Playwright probe against managed `spec-viewer` at `1440x900`, validating the browse route and shared sidebar search shell after `viewer-ctl prepare spec-viewer`
- Focused Chromium Playwright probe against managed `spec-viewer` at `1440x900`, validating that a `FileTree` filter toggle becomes active after click with the shared `FilterToggleButton`
- Focused Chromium Playwright probe against managed `spec-viewer` at `1440x900`, validating that the `FileTree` sort row renders with the shared toggle contract, active class, and sort title metadata
- Focused Chromium Playwright probe against managed `spec-viewer` at `1440x900`, validating a rich spec leaf tooltip after expanding the first component folder
- Focused Chromium Playwright probe against managed `doc-viewer` at `1440x900`, validating rich package and artifact tooltips after `viewer-ctl prepare doc-viewer`
- `trunk serve --release --port 8092` for `viewer-api-dioxus` plus a focused Chromium Playwright probe at `1440x900`, validating both a directory tooltip and a leaf tooltip in the shared demo tree

## Code references

- `memory-viewers/viewer-api/viewer-api/frontend/dioxus/src/components/tree_view.rs`
- `memory-viewers/viewer-api/viewer-api/frontend/dioxus/src/main/demo.rs`
- `memory-viewers/viewer-api/viewer-api/frontend/dioxus/index.html`
- `memory-viewers/viewer-api/viewer-api/frontend/dioxus/src/components/tree_view/explorer_shell.rs`
- `memory-viewers/viewer-api/viewer-api/frontend/dioxus/src/components/tree_view/filter_toggle_button.rs`
- `memory-viewers/spec-viewer/frontend/dioxus/src/components/spec_tree.rs`
- `tools/viewer/doc-viewer/frontend/dioxus/src/app.rs`
- `memory-viewers/ticket-viewer/frontend/dioxus/src/components/ticket_tree/header.rs`
- `memory-viewers/ticket-viewer/frontend/dioxus/src/components/ticket_tree/page.rs`
