# Plan: Nesting View Mode for Hypergraph Visualization

**Date:** 2026-03-07  
**Status:** Ready for Implementation  
**Interview:** [20260307_INTERVIEW_NESTING_VIEW.md](../interviews/20260307_INTERVIEW_NESTING_VIEW.md)  
**Estimated Complexity:** High (>100 lines, >10 files)

## Objective

Implement a hierarchical nesting view for the hypergraph visualization that:
1. Shows selected node expanded with children inside (row layout)
2. Shows parent nodes as increasingly larger overlapping containers **around and behind** the selected node — the selected node sits physically inside its parents, which sit inside their parents (Russian-doll nesting)
3. Supports duplicate mode (default) where child nodes appear both in their original position AND inside the expanded parent

## Interview Summary

| Feature | Decision |
|---------|----------|
| Expansion style | Row layout (horizontal row below label) |
| Parent positioning | Nesting shells (selected node sits **inside** parents; parents sit inside grandparents) |
| Parent visual | Dimmed + larger container backgrounds (semi-transparent, drawn behind) |
| Duplicate appearance | Identical with badge + special edge endpoint indicator |
| Click duplicate | Navigate to original node |
| Original when duplicated | Stays visible but dimmed |
| Edges | Only to originals; internal edges hidden → node highlights |
| Depth | User-configurable for both parents and children |
| Navigation | Click parent label → smooth transition |
| Default | Remember last (fallback: nesting + duplicate mode) |
| Toggle location | ControlsHUD |
| Performance | Smart culling (hide off-screen duplicates) |

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    HypergraphViewCore                        │
│  ┌─────────────────┐  ┌──────────────────┐                  │
│  │ useNestingState │  │ useNestingLayout │                  │
│  │ (settings +     │  │ (shell positions,│                  │
│  │  localStorage)  │  │  child positions)│                  │
│  └────────┬────────┘  └────────┬─────────┘                  │
│           │                    │                             │
│           ▼                    ▼                             │
│  ┌────────────────────────────────────────┐                 │
│  │           NestingManager               │                 │
│  │  - Duplicate node creation/tracking    │                 │
│  │  - Shell layout computation            │                 │
│  │  - Edge-to-highlight conversion        │                 │
│  └────────────────────────────────────────┘                 │
│           │                                                  │
│           ▼                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │  NodeLayer   │  │ edgeBuilder  │  │nodePositioner│       │
│  │ (duplicates) │  │ (originals)  │  │ (shells)     │       │
│  └──────────────┘  └──────────────┘  └──────────────┘       │
└─────────────────────────────────────────────────────────────┘
```

## Detailed Design

### 1. Nesting State & Settings

Add the following shapes to `types.ts`:

- **`NestingSettings`** — user-configurable options: master enabled toggle, duplicate mode toggle, parent depth (1–5), child depth (1–3).
- **`NestingState`** — runtime state: the current selected node index, the computed list of `ShellNode`s (one per parent in the hierarchy), and the list of active `DuplicateNode`s.
- **`ShellNode`** — describes one parent node rendered as a nesting shell: its graph index, its shell level (1 = direct parent), its rendered container size (width × height), and its center offset from the selected node's position (to handle multi-parent horizontal spread).
- **`DuplicateNode`** — describes one child rendered inside an expanded parent: the original node index, a stable unique ID for DOM keying, which parent contains it, and its slot index in the row layout.

`hooks/useNestingState.ts` manages the settings half of this state with `localStorage` persistence (key `hg-nesting-settings`). It exposes the current settings and a setter. The rest of `NestingState` (shells, duplicates) is derived/computed, not persisted.

### 2. Shell Layout Algorithm

The selected node is nested **inside** increasingly larger parent containers — like Russian nesting dolls. The selected node is the innermost element. Its direct parent(s) form shell level 1, rendered as a larger background rect behind the selected node. Grandparents form shell level 2, rendered even larger behind level 1. Each shell visually contains everything inside it.

**Size computation** (`nesting/shellLayout.ts`):

Start with the selected node's bounding box as the baseline content size. For each level outward, add padding on all sides:

```
padding(level) = BASE_PADDING + level × LEVEL_PADDING

shellSize(level) = contentSize(level - 1) + padding(level) × 2
```

When a level has multiple parents, they are arranged side-by-side. The combined width at that level becomes the content width for the next level outward:

```
combinedWidth(level) = shellWidth(level) × parentCount(level)
```

**Position computation:**

All shells share the same center as the selected node. They are drawn behind it (lower z-index), expanding outward symmetrically. When multiple parents exist at the same level, they are offset horizontally from center so each visually contains the shared child region:

```
for each parent[i] of count N at level L:
    centerX = (i - (N-1)/2) × shellWidth(L)
    centerY = 0   // same vertical center as selected node
```

**Traversal algorithm:**

```
visited = {selectedIdx}
contentSize = selectedNodeSize
currentLevel = [selectedIdx]

for level = 1 to parentDepth:
    nextLevel = []
    for each idx in currentLevel:
        for each parentIdx of node(idx):
            if parentIdx not in visited:
                add to nextLevel, mark visited

    if nextLevel is empty: break

    compute shellSize from contentSize + padding(level)
    compute centerX offsets for each parent in nextLevel
    emit ShellNode for each parent

    contentSize = combinedSize of all siblings at this level
    currentLevel = nextLevel
```

### 3. Duplicate Node Rendering

`NodeLayer.tsx` currently renders one `NodeElement` per graph node. Extend it to also accept a list of `DuplicateNode`s and a map of their computed positions.

For each duplicate, render an additional `NodeElement` using the original node's data but with:
- A unique DOM key (the `duplicateId`)
- An `isDuplicate` flag that triggers badge rendering
- The duplicate's computed position (inside the expanded parent's row layout)

`NodeElement.tsx` gains an `isDuplicate` prop. When set, a small overlay badge is shown (e.g., a "return to original" icon, top-right corner) using an `::after` pseudo-element in `hypergraph.css`. The badge makes it visually clear the node is a copy; clicking it navigates to the original.

Original nodes that have an active duplicate are rendered with reduced opacity (dimmed) to signal they are "also shown elsewhere".

### 4. Edge → Node Highlight Conversion

`nesting/edgeHighlights.ts` encapsulates the rule: **edges between a node and its children are not drawn when those children are rendered inside the parent as nested content.** Instead, containment is visually implied by the nesting shell itself.

The conversion produces an `EdgeHighlight` list — one entry per involved node — which `edgeBuilder.ts` consumes. `edgeBuilder.ts` is modified to:
1. Skip any edge whose both endpoints are in the current nesting view (parent + its rendered children).
2. Pass the highlight list to the GPU buffer so nodes receive a subtle glow indicating the relationship.

Pseudo code for highlight generation:

```
highlights = []
add highlight for expandedNode  (role: parent)
for each child rendered inside expandedNode:
    add highlight for child  (role: child)
return highlights
```

### 5. Smart Culling for Performance

`nesting/duplicateManager.ts` maintains the active duplicate set. Before each render, it filters out duplicates whose computed position falls outside the current viewport (plus a small margin). Off-screen duplicates are excluded from the render list entirely — not just hidden via CSS — so they do not consume DOM nodes or layout budget.

```
visibleDuplicates = []
for each duplicate:
    pos = computedPosition(duplicate)
    if pos is within (viewport + MARGIN):
        add to visibleDuplicates
return visibleDuplicates
```

The margin (≈100px) prevents pop-in during panning. This keeps DOM node count proportional to what is actually visible rather than total graph size.



## File Changes

### New Files
| File | Purpose |
|------|---------|
| `hooks/useNestingState.ts` | Settings state + localStorage persistence |
| `nesting/shellLayout.ts` | Shell position computation |
| `nesting/duplicateManager.ts` | Duplicate node creation/tracking |
| `nesting/edgeHighlights.ts` | Edge → node highlight conversion |

### Modified Files
| File | Changes |
|------|---------|
| `types.ts` | Add `NestingSettings`, `ShellNode`, `DuplicateNode` types |
| `hooks/index.ts` | Export `useNestingState` |
| `components/NodeLayer.tsx` | Render duplicates, duplicate badge styling |
| `components/ControlsHUD.tsx` | Add nesting toggles + depth sliders |
| `components/NodeElement.tsx` | Handle `isDuplicate` prop, badge rendering |
| `gpu/edgeBuilder.ts` | Skip edges to duplicates, generate highlight data |
| `animation/nodePositioner.ts` | Position shell nodes + duplicates |
| `hooks/useOverlayRenderer.ts` | Integrate nesting manager, pass settings |
| `HypergraphViewCore.tsx` | Wire up nesting state and layout |
| `hypergraph.css` | Shell node styles, duplicate badge, highlight glow |
| `decomposition/manager.ts` | Deprecate/remove (replaced by nesting system) |

## Execution Steps

### Phase 1: Foundation (Settings & Types)
- [ ] 1.1 Add types to `types.ts`
- [ ] 1.2 Create `hooks/useNestingState.ts` with localStorage
- [ ] 1.3 Export from `hooks/index.ts`
- [ ] 1.4 Add toggles to `ControlsHUD.tsx`

### Phase 2: Shell Layout
- [ ] 2.1 Create `nesting/shellLayout.ts`
- [ ] 2.2 Integrate shell computation in `useOverlayRenderer`
- [ ] 2.3 Update `nodePositioner.ts` to position shell nodes
- [ ] 2.4 Add shell node CSS (dimmed, scaled)

### Phase 3: Duplicate System
- [ ] 3.1 Create `nesting/duplicateManager.ts`
- [ ] 3.2 Add duplicate rendering to `NodeLayer.tsx`
- [ ] 3.3 Add duplicate badge styling
- [ ] 3.4 Original node dimming when duplicated

### Phase 4: Edge Handling
- [ ] 4.1 Create `nesting/edgeHighlights.ts`
- [ ] 4.2 Modify `edgeBuilder.ts` to skip internal edges
- [ ] 4.3 Apply node highlight glow styling

### Phase 5: Navigation & Transitions
- [ ] 5.1 Click handler for shell parent navigation
- [ ] 5.2 Click handler for duplicate → original navigation
- [ ] 5.3 Smooth animation transitions between selections

### Phase 6: Polish
- [ ] 6.1 Smart culling implementation
- [ ] 6.2 Depth slider controls
- [ ] 6.3 Clean up old decomposition code
- [ ] 6.4 Testing with various graph structures

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Performance with many duplicates | Smart culling, limit max duplicates |
| Complex state management | Clear ownership: NestingManager owns duplicates |
| Animation jank during transitions | Use requestAnimationFrame, batch DOM updates |
| Confusion about duplicate vs original | Clear badge, click always goes to original |

## Validation Criteria

- [ ] Toggle enables/disables nesting view
- [ ] Parent shells render at correct positions and scales
- [ ] Duplicates appear inside expanded parent with badge
- [ ] Original nodes dim when duplicated
- [ ] Internal edges hidden, node highlights visible
- [ ] Click duplicate navigates to original
- [ ] Click shell parent transitions view smoothly
- [ ] Depth sliders work correctly
- [ ] Settings persist across page reloads
- [ ] Performance acceptable with 100+ nodes
