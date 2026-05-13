# viewer-api: TreeView component

Canonical specification for the shared `TreeView` Dioxus component
(`viewer-api/frontend/dioxus/src/components/tree_view.rs`) — the
expandable tree used in doc-viewer (crate browser), log-viewer (file
list), spec-viewer (spec hierarchy), and ticket-viewer (ticket
hierarchy).

## Public surface

- `TreeNode { id, label, icon: NodeIcon, children, meta }`.
- `NodeIcon` enum (Folder / FolderOpen / File / Doc / Code / Module / etc.).
- `TreeView { nodes, selected, on_select, filters, sort_key }`.
- `FileTree` convenience constructor that builds a `TreeView` from a list of
  `(path, kind)` tuples.
- `FilterDef { id, label, predicate }`, `SortKey { Name, Kind, Custom(fn) }`.

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

## Code references

- `tools/viewer/viewer-api/frontend/dioxus/src/components/tree_view.rs`
- `tools/viewer/e2e/tests/demo-viewer/tree-view.spec.ts`
