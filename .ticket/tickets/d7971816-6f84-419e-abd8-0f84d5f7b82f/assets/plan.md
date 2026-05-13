# Plan: Sortable FileTree with Generic Sorting Header

**Ticket:** d7971816-6f84-419e-abd8-0f84d5f7b82f
**Component:** viewer-api (shared), ticket-viewer (consumer)
**Risk:** Low

---

## Goal

Add a generic sorting header to the shared `FileTree` component in `viewer-api/frontend` so any viewer can offer column-based sorting. Integrate it in `ticket-viewer` with "last modified", "created", and "title" sort keys.

## Current State

- `TreeView` (`viewer-api/frontend/src/components/TreeView.tsx`): Generic recursive tree with `TreeNode<T>` — no sorting awareness.
- `FileTree` (`viewer-api/frontend/src/components/FileTree.tsx`): Thin wrapper adding loading/empty states around `TreeView`. No sorting props.
- `TicketTree` (`ticket-viewer/frontend/src/components/TicketTree.tsx`): Groups tickets by state into folder nodes. Has search + state filter toolbar. No sorting.
- `TicketSummary` has `updated_at` (string ISO date). `TicketDetail` has `created_at`. Title is on `TicketSummary`.

## Design

### Sorting Types (viewer-api)

```typescript
// In FileTree.tsx or a new sort-header.tsx

export interface SortOption<K extends string = string> {
  key: K;
  label: string;
  /** Default direction when first selected. Defaults to 'asc'. */
  defaultDirection?: 'asc' | 'desc';
}

export type SortDirection = 'asc' | 'desc';

export interface SortState<K extends string = string> {
  key: K;
  direction: SortDirection;
}
```

### SortHeader Component (viewer-api)

A small row rendered above the tree:

```
  [ Title ▲ ] [ Modified ▽ ] [ Created ▽ ]
```

- Each sort key is a clickable button.
- Active key shows direction arrow (▲/▼).
- Clicking active key toggles direction.
- Clicking inactive key activates it with its `defaultDirection`.

### FileTree Additions

Add optional sorting props to `FileTreeProps`:

```typescript
export interface FileTreeProps<T = unknown> {
  // ... existing props ...
  /** Available sort columns. If provided, renders a sort header. */
  sortOptions?: SortOption[];
  /** Current sort state (controlled). */
  sortState?: SortState;
  /** Callback when user changes sort. */
  onSortChange?: (state: SortState) => void;
}
```

FileTree renders `<SortHeader>` above `<TreeView>` when `sortOptions` is provided. Sorting logic (actual reordering of nodes) stays in the consumer — FileTree only renders the UI control and fires callbacks.

### TicketTree Integration

```typescript
type TicketSortKey = 'title' | 'updated_at' | 'created_at';

const SORT_OPTIONS: SortOption<TicketSortKey>[] = [
  { key: 'title', label: 'Title', defaultDirection: 'asc' },
  { key: 'updated_at', label: 'Modified', defaultDirection: 'desc' },
  { key: 'created_at', label: 'Created', defaultDirection: 'desc' },
];
```

- Add `treeSortState` signal to store (default: `{ key: 'updated_at', direction: 'desc' }`).
- Persist sort state in `persistWorkspaceState` / `restoreWorkspaceState`.
- Before grouping tickets by state, sort them using the active key.
- For `created_at`: **RESOLVED** — `created_at` will be added to the `TicketSummary` HTTP response (see sub-ticket `d3a8b66a`). The `IndexedTicket` storage model already has `created_at`, so this is a 1-line addition to the Rust struct + handler mapping.

### CSS

Add to `viewer-api/frontend/src/styles/file-tree.css`:

```css
.file-tree__sort-header {
  display: flex;
  gap: 2px;
  padding: 4px 8px;
  border-bottom: 1px solid var(--border-subtle);
  font-size: 0.75rem;
}

.file-tree__sort-btn {
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 3px;
}

.file-tree__sort-btn:hover {
  background: var(--bg-hover);
}

.file-tree__sort-btn--active {
  color: var(--text-primary);
  font-weight: 600;
}
```

## Implementation Steps

1. **Add `SortOption`, `SortDirection`, `SortState` types** to viewer-api FileTree exports.
2. **Create `SortHeader` component** in viewer-api (small, <40 lines).
3. **Extend `FileTreeProps`** with optional `sortOptions`, `sortState`, `onSortChange`.
4. **Render `SortHeader` in FileTree** when `sortOptions` provided.
5. **Add sort header CSS** to `file-tree.css`.
6. **Add `treeSortState` signal** to ticket-viewer store.
7. **Sort tickets in TicketTree** before building the grouped tree nodes.
8. **Persist/restore sort state** in workspace state.
9. **Verify**: typecheck viewer-api + ticket-viewer, manual test in browser.

## Files Changed

| File | Change |
|------|--------|
| `tools/viewer-api/frontend/src/components/FileTree.tsx` | Add SortHeader, extend props |
| `tools/viewer-api/frontend/src/components/TreeView.tsx` | Export sort types (or co-locate in FileTree) |
| `tools/viewer-api/frontend/src/styles/file-tree.css` | Sort header styles |
| `tools/viewer-api/frontend/src/index.ts` | Re-export sort types |
| `tools/ticket-viewer/frontend/src/components/TicketTree.tsx` | Use sort props, apply sorting |
| `tools/ticket-viewer/frontend/src/store.ts` | Add treeSortState signal + persistence |

## Resolved Questions

1. **RESOLVED:** `created_at` will be added to `TicketSummary` HTTP API response (sub-ticket `d3a8b66a`). `IndexedTicket` already has it.
2. **RESOLVED:** Sorting persists per-workspace, consistent with existing filter persistence.
3. **RESOLVED (A1):** Sorting is applied **within each state group** — tickets within a group are sorted by the active key, but the state groups themselves maintain their fixed order. This preserves the structural grouping while adding per-group ordering. Fixed group order: open → in-progress → review → validating → validated → done → blocked → cancelled → unknown.

## Dependency

This ticket depends on `d3a8b66a` (Add created_at to TicketSummary HTTP response).
