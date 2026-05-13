# viewer-api: keyboard interaction model

Draft cross-viewer interaction contract for keyboard ownership, shortcut precedence, and phased rollout across shared viewer surfaces.

## Goals

- make keyboard behavior predictable across viewers
- avoid conflicts between local controls, overlays, editors, and global shortcuts
- keep browser-default accessibility behavior intact unless an explicit viewer contract replaces it

## Focus ownership

Keyboard events are resolved in this order:

1. focused text input / textarea / editable field
2. active modal or overlay
3. focused local interaction surface (tree, result list, tab strip, graph canvas)
4. viewer-global shortcuts

A higher-priority owner suppresses lower-priority shortcuts.

## Baseline rules

- `Escape` closes the topmost dismissible overlay or modal before any lower-level action runs.
- Search-launch shortcuts such as `/` or `Ctrl+K` only fire when focus is not inside a text-editing field.
- Raw `Tab` / `Shift+Tab` remain browser focus traversal by default.
- Switching content tabs should prefer an explicit scoped shortcut such as `Ctrl+Tab` / `Ctrl+Shift+Tab` or a surface-local alternative, rather than stealing bare `Tab` globally.

## Surface contracts

### Trees and result lists

- `ArrowUp` / `ArrowDown` move visible focus.
- `Enter` activates the focused item.

### Tab strips

- Tab strips may expose dedicated keyboard activation, but they do not override the default browser tab-order contract without an explicit approved design.

### Graph canvases

- Keyboard camera controls require explicit graph focus or an armed graph interaction mode.
- `W`, `A`, `S`, `D` MUST NOT move the camera while a text-editing control or modal owns focus.
- The graph surface needs a visible focus or armed-state indicator before movement shortcuts become active.

## Rollout phases

1. Local ticket-list navigation: sidebar explorer + quick-search.
2. Detail-panel tab switching and related focused actions.
3. Graph/camera keyboard controls, including WASD gating and escape hatches.

## Related shared specs

- `viewer-api/components/tree-view`
- `viewer-api/components/tab-bar`
- `viewer-api/components/graph3d`
