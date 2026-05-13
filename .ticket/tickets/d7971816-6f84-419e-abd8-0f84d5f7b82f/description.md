Add a generic sorting header to the shared `FileTree` component in `viewer-api/frontend` and integrate it in ticket-viewer.

## Design

**New types in viewer-api:**
- `SortOption<K>` — key, label, default direction
- `SortState<K>` — active key + direction (asc/desc)
- `SortHeader` component — clickable column buttons with direction arrows (▲/▼)

**FileTree additions:**
- Optional `sortOptions`, `sortState`, `onSortChange` props on `FileTreeProps`
- When `sortOptions` is provided, renders `<SortHeader>` above `<TreeView>`
- Sorting logic (actual node reordering) stays in the consumer — FileTree only renders the UI control

**TicketTree integration (A1):** Sorting is applied **within each state group** — tickets within a group are sorted by the active key, but state groups maintain their fixed order: open → in-progress → review → validating → validated → done → blocked → cancelled → unknown. Sorting never breaks the grouping structure.
- Sort keys: `title` (asc default), `updated_at` (desc default), `created_at` (desc default)
- Sorts tickets within each state group
- Sort state persisted per-workspace in localStorage

## Dependencies

Depends on d3a8b66a (Add `created_at` to TicketSummary HTTP response) for the "created" sort key.

## Files Touched

- `viewer-api/frontend/src/components/FileTree.tsx` — add sort props + SortHeader rendering
- `viewer-api/frontend/src/components/SortHeader.tsx` — new component
- `viewer-api/frontend/src/components/FileTree.css` — sort header styles
- `ticket-viewer/frontend/src/components/TicketTree.tsx` — sorting integration
- `ticket-viewer/frontend/src/store.ts` — persisted sort state signal
