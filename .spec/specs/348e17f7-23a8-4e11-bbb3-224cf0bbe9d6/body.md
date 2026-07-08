<!-- aligned-structure:v1 -->

# Summary

Canonical specification for the shared `TabBar` Dioxus component (`viewer-api/frontend/dioxus/src/components/tab_bar.rs`).

## Behavior Story

Canonical specification for the shared `TabBar` Dioxus component (`viewer-api/frontend/dioxus/src/components/tab_bar.rs`).

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# viewer-api: TabBar component

Canonical specification for the shared `TabBar` Dioxus component
(`viewer-api/frontend/dioxus/src/components/tab_bar.rs`).

## Public surface

- `TabItem { id, label, icon: Option<NodeIcon>, closable: bool, dirty: bool }`.
- `TabBar { items, active_id, on_select, on_close, on_reorder }`.

## Demo behavior

The `pages/tab_bar.rs` page demonstrates:

1. A `TabBar` with 5 initial tabs of varying length.
2. Buttons to add a new tab, mark the active tab as dirty (•), and reset.
3. Closable vs pinned tabs (no close ✕ on pinned).
4. Drag-to-reorder (mouse + touch) with visible drop indicator.
5. Overflow behavior: when tabs exceed the bar width, a scroll affordance
   appears on each side.

## Acceptance behavior (validated by e2e)

- Clicking a tab makes it active (`aria-selected="true"` on that tab).
- Closing a non-active tab does not change the active id.
- Closing the active tab activates the neighbour to the right (or left if
  it was the last).
- Reordering moves the dragged tab to the drop position; the order is
  reflected in `on_reorder`.
- Dirty indicator (•) appears next to the label.

## Code references

- `tools/viewer/viewer-api/frontend/dioxus/src/components/tab_bar.rs`
- `tools/viewer/e2e/tests/demo-viewer/tab-bar.spec.ts`
