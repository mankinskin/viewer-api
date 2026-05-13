# Plan: Viewer Refactoring, File Tree Sync & Mobile Support

**Date:** 2026-03-04
**Scope:** log-viewer, doc-viewer, viewer-api — frontend
**Interview:** `agents/interviews/20260303_VIEWER_REFACTORING_AND_MOBILE.md`
**Parent plan:** `agents/plans/20260301_PLAN_VIEWER_TOOLS_FEATURES.md`
**Status:** READY

---

## Objective

Refactor the HypergraphView rendering pipeline for modularity and performance, synchronize file tree and sidebar implementations across doc-viewer and log-viewer, and add mobile/responsive support with collapsible sidebar and touch gestures.

## Decisions (from interview)

- **Refactoring:** Full refactor + extraction to viewer-api
- **DecompositionOverlay.tsx:** Delete (dead code)
- **Performance:** All four optimizations (dirty-flag edge buffer, connected-set cache, merge RAF loops, frustum culling)
- **File tree grouping:** Hybrid (directory tree + virtual category folders)
- **Non-log files:** Reuse existing CodeViewer tab
- **Sidebar sync:** Shared Shell + TreeView + Tooltip; app-owned node types & behavior
- **Touch gestures:** Both HypergraphView and Scene3D
- **Mobile sidebar:** Full-screen overlay
- **Breakpoints:** 768px (tablet) + 480px (phone)
- **Order:** WS1 (refactor) → WS2 (file tree) → WS3 (mobile)

---

## WS1: HypergraphView Refactoring & Extraction

**Goal:** Break the 550+ line `useOverlayRenderer` into small focused modules, apply performance optimizations, delete dead code, and extract the core rendering engine to `viewer-api` for reuse.

### Phase 1.1: Delete Dead Code & Audit

| Step | Description | Files |
|------|-------------|-------|
| 1.1.1 | Delete `DecompositionOverlay.tsx` | `log-viewer/frontend/src/components/HypergraphView/components/DecompositionOverlay.tsx` |
| 1.1.2 | Remove export from components barrel | `log-viewer/frontend/src/components/HypergraphView/components/index.ts` |
| 1.1.3 | Audit `HypergraphView.tsx` for unused imports/refs | `log-viewer/frontend/src/components/HypergraphView/HypergraphView.tsx` |
| 1.1.4 | Verify no other references to DecompositionOverlay in the codebase | grep search across repo |

**Validation:** `cd tools/log-viewer/frontend && npm run build` succeeds, no import errors.

### Phase 1.2: Extract GPU Pipeline Setup

Split one-time GPU resource creation out of `useOverlayRenderer`.

| Step | Description | New file |
|------|-------------|----------|
| 1.2.1 | Create `gpu/pipeline.ts` — export `createEdgePipeline()`, `createGridPipeline()`, `createBuffers()` that encapsulate shader compilation, pipeline layout, vertex/bind group layout, uniform buffers | `HypergraphView/gpu/pipeline.ts` |
| 1.2.2 | Create `gpu/constants.ts` — move `EDGE_INSTANCE_FLOATS`, `GRID_LINE_FLOATS`, `QUAD_VERTS`, color constants (`PATTERN_COLORS`, `PATH_EDGE_COLOR`, `SP_*`, `CANDIDATE_EDGE_COLOR`, etc.) | `HypergraphView/gpu/constants.ts` |
| 1.2.3 | Create `gpu/index.ts` barrel | `HypergraphView/gpu/index.ts` |
| 1.2.4 | Update `useOverlayRenderer.ts` to import from `gpu/pipeline` and `gpu/constants` | `HypergraphView/hooks/useOverlayRenderer.ts` |

**Validation:** Build succeeds, hypergraph view renders identically.

### Phase 1.3: Extract Edge Rendering Logic

Move the 100+ line edge coloring cascade into a dedicated module.

| Step | Description | New file |
|------|-------------|----------|
| 1.3.1 | Create `gpu/edgeBuilder.ts` — export `buildEdgeInstances(layout, vizState, interactionState, expandedNodes)` → returns `{ edgeData: Float32Array, edgeCount: number }` | `HypergraphView/gpu/edgeBuilder.ts` |
| 1.3.2 | Move edge type classification logic (search path keys, candidate detection, insert detection, parent/child highlighting) into `classifyEdge()` helper | same file |
| 1.3.3 | Move color selection logic into `getEdgeColor(classification)` helper | same file |
| 1.3.4 | Update `useOverlayRenderer.ts` render callback to call `buildEdgeInstances()` | `HypergraphView/hooks/useOverlayRenderer.ts` |

**Validation:** Edge highlighting still works for all edge types (search path, insert, candidate, selection).

### Phase 1.4: Extract Grid Rendering

| Step | Description | New file |
|------|-------------|----------|
| 1.4.1 | Create `gpu/gridBuilder.ts` — export `buildGridData()` → returns `{ gridData: Float32Array, gridCount: number }` | `HypergraphView/gpu/gridBuilder.ts` |
| 1.4.2 | Move grid line generation (GRID_EXTENT, GRID_STEP, axis lines) from useOverlayRenderer | same file |
| 1.4.3 | Update `useOverlayRenderer.ts` to import grid builder | `HypergraphView/hooks/useOverlayRenderer.ts` |

**Validation:** Grid renders correctly.

### Phase 1.5: Extract Node Animation & Positioning

| Step | Description | New file |
|------|-------------|----------|
| 1.5.1 | Create `animation/nodeAnimator.ts` — export `animateNodes(nodes, dt, lerpSpeed)` for node lerp updates | `HypergraphView/animation/nodeAnimator.ts` |
| 1.5.2 | Create `animation/nodePositioner.ts` — export `positionDOMNodes(nodes, nodeElMap, viewProj, camPos, vw, vh, interState, vizState, expandedNodes, reparentedSet)` for the CSS transform positioning loop | `HypergraphView/animation/nodePositioner.ts` |
| 1.5.3 | Create `animation/index.ts` barrel | `HypergraphView/animation/index.ts` |
| 1.5.4 | Update `useOverlayRenderer.ts` render callback to use extracted functions | `HypergraphView/hooks/useOverlayRenderer.ts` |

**Validation:** Node animation smooth, positioning correct.

### Phase 1.6: Extract Decomposition Manager

Move the imperative DOM reparenting logic out of the render callback.

| Step | Description | New file |
|------|-------------|----------|
| 1.6.1 | Create `decomposition/manager.ts` — export `DecompositionManager` class with `expand(idx)`, `collapse(idx)`, `collapseAll()`, `update(desiredExpanded)`, `getReparentedInfo()` | `HypergraphView/decomposition/manager.ts` |
| 1.6.2 | Move `expandNode()`, `collapseNode()`, `collapseAll()`, `reorderNodeLayer()`, `updateNodeElMap()`, `ROW_COLORS`, and `ExpandedNodeState` interface into manager | same file |
| 1.6.3 | Create `decomposition/index.ts` barrel | `HypergraphView/decomposition/index.ts` |
| 1.6.4 | Update `useOverlayRenderer.ts` to instantiate and use `DecompositionManager` | `HypergraphView/hooks/useOverlayRenderer.ts` |

**Validation:** Decomposition expand/collapse works for selected and search-path root nodes.

### Phase 1.7: Merge RAF Loops

Eliminate the double `requestAnimationFrame` pattern.

| Step | Description | Files |
|------|-------------|-------|
| 1.7.1 | Move focused-layout projection logic from `HypergraphView.tsx` `useEffect` into the overlay render callback (where camera axes are already available) | `HypergraphView.tsx`, `useOverlayRenderer.ts` |
| 1.7.2 | Pass `focusedOffsetsRef` and `originalPositionsRef` to the overlay renderer via props or a shared ref | same |
| 1.7.3 | Remove the standalone `requestAnimationFrame` loop from `HypergraphView.tsx` | `HypergraphView.tsx` |
| 1.7.4 | Ensure focused layout still projects correctly each frame using camera axes from the render callback | verify manually |

**Validation:** Node layouts animate correctly when selecting nodes with highlight mode on/off.

### Phase 1.8: Performance Optimizations

| Step | Description | Files |
|------|-------------|-------|
| 1.8.1 | **P1 — Dirty-flag edge buffer:** Add a dirty flag to `edgeBuilder`. Set dirty when `vizState`, `selectedIdx`, `layout`, or `expandedNodes` change. Skip `buildEdgeInstances()` + GPU upload when clean. | `gpu/edgeBuilder.ts`, `useOverlayRenderer.ts` |
| 1.8.2 | **P2 — Connected-set caching:** Move `connectedSet` and `connectedEdgeKeys` computation into a `useMemo` keyed on `[selectedIdx, layout]`. Pass the cached sets into the render callback via ref. | `useOverlayRenderer.ts` or new `hooks/useConnectedSet.ts` |
| 1.8.3 | **P3 — Merge RAF loops:** (completed in Phase 1.7) | — |
| 1.8.4 | **P4 — Frustum culling:** In `nodePositioner.ts`, use the viewProj matrix to check if a node's screen position is within `[-margin, vw+margin] x [-margin, vh+margin]` before applying CSS transform. Nodes outside get `display: none`. | `animation/nodePositioner.ts` |

**Validation:** Profile with 100+ node graph — fewer GPU writes per frame, reduced JS time in render callback.

### Phase 1.9: Unify Math Utilities

| Step | Description | Files |
|------|-------------|-------|
| 1.9.1 | Audit `HypergraphView/utils/math.ts` vs `Scene3D/math3d.ts` for duplicated functions | both files |
| 1.9.2 | Move shared functions (`worldToScreen`, `worldScaleAtDepth`, `raySphere`) into a common `utils/math3d.ts` in the HypergraphView or into `Scene3D/math3d.ts` (since it's already the more complete one) | depends on audit |
| 1.9.3 | Keep `edgePairKey`, `edgeTripleKey` in `HypergraphView/utils/math.ts` (hypergraph-specific) | `utils/math.ts` |
| 1.9.4 | Update all imports | affected consumers |

**Validation:** Build succeeds, no duplicate function definitions.

### Phase 1.10: Extract Core to viewer-api

Move the rendering engine (minus log-viewer-specific signals) to viewer-api.

| Step | Description | Target |
|------|-------------|--------|
| 1.10.1 | Identify direct signal dependencies: `hypergraphSnapshot`, `activeSearchStep`, `activeSearchState`, `activeSearchPath`, `activePathEvent`, `activePathStep`, `selectHighlightMode` | research |
| 1.10.2 | Define a `HypergraphViewProps` interface that accepts data via props instead of reading signals directly: `snapshot`, `vizEvent`, `searchPath`, `highlightMode`, etc. | `viewer-api/frontend/src/components/HypergraphView/types.ts` |
| 1.10.3 | Copy the refactored modules (`gpu/`, `animation/`, `decomposition/`, `hooks/`, `utils/`, `components/`, `layout.ts`) to `viewer-api/frontend/src/components/HypergraphView/` | new directory |
| 1.10.4 | Update `HypergraphView.tsx` to accept props instead of reading signals | extracted component |
| 1.10.5 | Create a thin wrapper in log-viewer that reads signals and passes them as props to the shared component | `log-viewer/frontend/src/components/HypergraphView/HypergraphView.tsx` (becomes wrapper) |
| 1.10.6 | Update `viewer-api/frontend/src/index.ts` barrel exports | `viewer-api/frontend/src/index.ts` |
| 1.10.7 | Update WGSL shader imports and CSS imports for the new location | extracted component |

**Validation:**
- `cd tools/log-viewer/frontend && npm run build` succeeds
- `cd tools/viewer-api/frontend && npm run build` succeeds (if applicable)
- Log-viewer renders hypergraph identically to before extraction

---

## WS2: File Tree & Sidebar Synchronization

**Goal:** Unify sidebar and file tree across doc-viewer and log-viewer. Support opening any file type. Maintain identical `.log` file behavior.

### Phase 2.1: Enhance Shared TreeView in viewer-api

| Step | Description | Files |
|------|-------------|-------|
| 2.1.1 | Extend `TreeNode` interface with optional fields: `tooltip?: ComponentChildren`, `icon?: string \| ComponentChildren`, `badge?: string \| number`, `data?: T` (generic) | `viewer-api/frontend/src/components/TreeView.tsx` |
| 2.1.2 | Add tooltip rendering: on hover, show tooltip content in a positioned popup near the tree row | same file, new `TreeTooltip` internal component |
| 2.1.3 | Add `onContextMenu` callback prop to `TreeView` for right-click actions | same file |
| 2.1.4 | Support external expanded-state control: accept optional `expanded` signal/set + `onToggle` callback (controlled mode) alongside existing internal state (uncontrolled mode) | same file |
| 2.1.5 | Update `tree.css` in viewer-api with tooltip styles | `viewer-api/frontend/src/styles/tree.css` |
| 2.1.6 | Export new types from `viewer-api/frontend/src/index.ts` | `viewer-api/frontend/src/index.ts` |

**Validation:** `cd tools/viewer-api/frontend && npm run build` (if applicable), types exported correctly.

### Phase 2.2: Enhance Shared Sidebar in viewer-api

| Step | Description | Files |
|------|-------------|-------|
| 2.2.1 | Add `collapsible?: boolean` prop to Sidebar — when true, shows a toggle button | `viewer-api/frontend/src/components/Sidebar.tsx` |
| 2.2.2 | Add `collapsed` signal + `onToggle` callback props for controlled collapse | same file |
| 2.2.3 | Add `resizable?: boolean` prop — when true, renders a resize handle on the right edge | same file |
| 2.2.4 | Extract `ResizeHandle` from doc-viewer into `viewer-api/frontend/src/components/ResizeHandle.tsx` | new file (move from `doc-viewer/frontend/src/components/ResizeHandle.tsx`) |
| 2.2.5 | Update viewer-api `layout.css` with collapse/expand animation styles | `viewer-api/frontend/src/styles/layout.css` |
| 2.2.6 | Export `ResizeHandle` from barrel | `viewer-api/frontend/src/index.ts` |

**Validation:** Sidebar collapses/expands, resize handle works.

### Phase 2.3: Create Shared FileContentViewer

A generic component that selects the right renderer based on file extension.

| Step | Description | Files |
|------|-------------|-------|
| 2.3.1 | Create `FileContentViewer.tsx` in viewer-api: accepts `file: Signal<string \| null>`, `content: Signal<string>`, `onRenderCustom?: (filename: string, content: string) => ComponentChildren \| null` | `viewer-api/frontend/src/components/FileContentViewer.tsx` |
| 2.3.2 | Built-in renderers: `.md` → MarkdownViewer (extract from doc-viewer `FileViewer.tsx`), `.rs/.ts/.js/.json/.toml/.yaml` → CodeViewer, other → plaintext CodeViewer | same file |
| 2.3.3 | `onRenderCustom` hook allows apps to override rendering for specific types (e.g., log-viewer provides custom `.log` renderer) | same file |
| 2.3.4 | Export from barrel | `viewer-api/frontend/src/index.ts` |

**Validation:** FileContentViewer renders code, markdown, and falls back to plain text.

### Phase 2.4: Convert Log-Viewer Sidebar to File Tree

| Step | Description | Files |
|------|-------------|-------|
| 2.4.1 | Add API endpoint or modify existing `GET /api/files` to return directory structure (nested) instead of flat list. Alternatively, build the tree client-side from flat paths. | `log-viewer/src/main.rs` or `log-viewer/frontend/src/store/index.ts` |
| 2.4.2 | Create `buildFileTree(files: LogFile[])` utility: groups files by directory, creates virtual category folders ("Graph", "Search", "Insert", "Paths") at top level containing symlink-like references to files matching each badge | `log-viewer/frontend/src/store/fileTree.ts` (new) |
| 2.4.3 | Replace the log-viewer `Sidebar.tsx` content: swap the flat `file-list` with the shared `TreeView` component, passing `TreeNode[]` from `buildFileTree()` | `log-viewer/frontend/src/components/Sidebar/Sidebar.tsx` |
| 2.4.4 | Move file metadata (size, modified date, badges) into TreeView tooltip content | same file |
| 2.4.5 | Maintain filter buttons at the top of the sidebar (they now filter the tree, not a flat list) — when a filter is active, expand matching category folder and collapse others | same file |
| 2.4.6 | Wire tree node click: `.log` files → `loadLogFile()` (existing), other files → `openSourceFile()` → CodeViewer tab | same file + `store/index.ts` |

**Validation:**
- Log files appear in a directory tree
- Virtual category folders appear at top level
- Clicking a `.log` file opens it in LogViewer (unchanged behavior)
- Clicking a `.rs`/`.toml` file opens in CodeViewer tab
- Filter buttons still work
- Tooltips show file size, modified date, and badges on hover

### Phase 2.5: Update Doc-Viewer to Use Enhanced Shared Components

| Step | Description | Files |
|------|-------------|-------|
| 2.5.1 | Update doc-viewer `Sidebar.tsx` to wrap content in the shared `Sidebar` component (with collapsible + resizable) | `doc-viewer/frontend/src/components/Sidebar.tsx` |
| 2.5.2 | Migrate doc-viewer inline `TreeItem` to use the shared `TreeView` from viewer-api, passing custom node types via `TreeNode.data` and custom icons via `TreeNode.icon` | same file |
| 2.5.3 | Move doc-specific icons (CrateIcon, ModuleIcon, SourceFileIcon, etc.) into their own file or keep inline — just ensure click handlers still work | same file or new icons file |
| 2.5.4 | Replace doc-viewer's custom `ResizeHandle` with the shared one from viewer-api | `doc-viewer/frontend/src/App.tsx` |
| 2.5.5 | Add tooltip content for doc tree nodes (e.g., show doc date, tags, status on hover) | `doc-viewer/frontend/src/components/Sidebar.tsx` |

**Validation:**
- Doc-viewer sidebar looks and behaves identically
- Collapse/expand and resize work with shared components
- Tooltips show doc metadata on hover
- `cd tools/doc-viewer/frontend && npm run build` succeeds

---

## WS3: Mobile & Responsive Support

**Goal:** Collapsible sidebar, full-screen overlay on mobile, touch gestures for 3D views, two responsive breakpoints.

### Phase 3.1: Responsive Sidebar CSS

| Step | Description | Files |
|------|-------------|-------|
| 3.1.1 | Add CSS variables: `--sidebar-collapsed-width: 0px`, `--mobile-breakpoint: 768px`, `--phone-breakpoint: 480px` | `viewer-api/frontend/src/styles/variables.css` |
| 3.1.2 | Add `@media (max-width: 768px)` rules: sidebar auto-collapses, shows hamburger toggle in header | `viewer-api/frontend/src/styles/layout.css` |
| 3.1.3 | Add `@media (max-width: 480px)` rules: sidebar hidden by default, full-screen overlay when toggled | same file |
| 3.1.4 | Add `.sidebar-overlay` class: `position: fixed; inset: 0; z-index: 1000; background: var(--bg-secondary)` with close button | same file |
| 3.1.5 | Add slide-in/out animation with `transform` + `transition` | same file |
| 3.1.6 | Mirror these styles in log-viewer's `layout.css` and doc-viewer's `layout.css` (or import shared) | `log-viewer/frontend/src/styles/layout.css`, `doc-viewer/frontend/src/styles/layout.css` |

**Validation:** Resize browser window — sidebar collapses at 768px, becomes full-screen overlay at 480px.

### Phase 3.2: Sidebar Toggle Logic

| Step | Description | Files |
|------|-------------|-------|
| 3.2.1 | Add `sidebarCollapsed` signal to viewer-api store (or as a prop-based pattern) | `viewer-api/frontend/src/store/` or component-level |
| 3.2.2 | Add hamburger toggle button to the shared `Header` component (visible only below breakpoint) | `viewer-api/frontend/src/components/Header.tsx` |
| 3.2.3 | Wire toggle in log-viewer `App.tsx`: hamburger toggles sidebar visibility | `log-viewer/frontend/src/App.tsx` |
| 3.2.4 | Wire toggle in doc-viewer `App.tsx`: same behavior | `doc-viewer/frontend/src/App.tsx` |
| 3.2.5 | Auto-collapse sidebar when a file is selected on mobile (< 480px) | sidebar click handler |
| 3.2.6 | Add backdrop overlay + click-to-dismiss when sidebar is open on mobile | CSS + JS event handler |

**Validation:** Toggle button appears on narrow viewport, opens full-screen overlay, tapping backdrop closes it.

### Phase 3.3: Touch Gestures for HypergraphView

| Step | Description | Files |
|------|-------------|-------|
| 3.3.1 | Create `hooks/useTouchInteraction.ts` — touch event handler registrations on the container | `HypergraphView/hooks/useTouchInteraction.ts` (new) |
| 3.3.2 | **Single-finger drag → orbit:** Track `touchstart` (1 finger) → `touchmove` → `touchend`. Map delta to `yaw`/`pitch` like mouse orbit. | same file |
| 3.3.3 | **Two-finger pinch → zoom:** Track distance between two touch points. Map distance delta to `camera.dist` like mouse wheel. | same file |
| 3.3.4 | **Two-finger pan → pan:** Track midpoint of two touches. Map midpoint delta to camera target like mouse pan. | same file |
| 3.3.5 | **Tap → select node:** If single touch starts and ends within 200ms and 10px, treat as tap. Ray-cast for node selection. | same file |
| 3.3.6 | **Double-tap → focus:** Detect double-tap (two taps within 300ms). Focus camera on tapped node. | same file |
| 3.3.7 | **Long press → info panel:** If touch held >500ms without moving >10px, show node info (equivalent to hover tooltip). | same file |
| 3.3.8 | Integrate `useTouchInteraction` into `HypergraphView.tsx` alongside `useMouseInteraction` | `HypergraphView/HypergraphView.tsx` |
| 3.3.9 | Add `touch-action: none` CSS to hypergraph container to prevent browser scroll/zoom interference | `HypergraphView/hypergraph.css` |

**Validation:** On mobile/tablet (or touch simulator): orbit, zoom, pan, tap-select, double-tap-focus all work.

### Phase 3.4: Touch Gestures for Scene3D

| Step | Description | Files |
|------|-------------|-------|
| 3.4.1 | Apply same touch gesture pattern to `Scene3D.tsx` — either share the hook or duplicate with Scene3D's camera API | `Scene3D/Scene3D.tsx` or new `Scene3D/useTouchInteraction.ts` |
| 3.4.2 | If camera APIs are compatible (both have yaw/pitch/dist/target), parameterize `useTouchInteraction` to accept a generic camera interface | `HypergraphView/hooks/useTouchInteraction.ts` |
| 3.4.3 | Add `touch-action: none` to Scene3D container CSS | `Scene3D/scene3d.css` |

**Validation:** Scene3D touch gestures work identically to HypergraphView gestures.

---

## File Summary

### New files

| File | Purpose |
|------|---------|
| `HypergraphView/gpu/pipeline.ts` | GPU pipeline/buffer setup |
| `HypergraphView/gpu/constants.ts` | Rendering constants & colors |
| `HypergraphView/gpu/edgeBuilder.ts` | Edge instance buffer building & coloring |
| `HypergraphView/gpu/gridBuilder.ts` | Grid line generation |
| `HypergraphView/gpu/index.ts` | Barrel export |
| `HypergraphView/animation/nodeAnimator.ts` | Node lerp animation |
| `HypergraphView/animation/nodePositioner.ts` | DOM node CSS transform positioning |
| `HypergraphView/animation/index.ts` | Barrel export |
| `HypergraphView/decomposition/manager.ts` | Decomposition DOM reparenting |
| `HypergraphView/decomposition/index.ts` | Barrel export |
| `HypergraphView/hooks/useTouchInteraction.ts` | Touch gesture handler |
| `viewer-api/frontend/src/components/ResizeHandle.tsx` | Shared resize handle (moved from doc-viewer) |
| `viewer-api/frontend/src/components/FileContentViewer.tsx` | Generic file type renderer |
| `log-viewer/frontend/src/store/fileTree.ts` | File tree builder from flat file list |

### Deleted files

| File | Reason |
|------|--------|
| `HypergraphView/components/DecompositionOverlay.tsx` | Dead code |

### Major modifications

| File | Changes |
|------|---------|
| `HypergraphView/hooks/useOverlayRenderer.ts` | ~550 lines → ~100 lines (thin orchestrator) |
| `HypergraphView/HypergraphView.tsx` | Remove focused-layout RAF loop, pass refs to overlay renderer |
| `log-viewer/frontend/src/components/Sidebar/Sidebar.tsx` | Flat list → TreeView with hybrid grouping |
| `doc-viewer/frontend/src/components/Sidebar.tsx` | Use shared TreeView + Sidebar from viewer-api |
| `doc-viewer/frontend/src/App.tsx` | Use shared ResizeHandle |
| `viewer-api/frontend/src/components/TreeView.tsx` | Add tooltips, controlled mode, generic data |
| `viewer-api/frontend/src/components/Sidebar.tsx` | Add collapsible + resizable props |
| `viewer-api/frontend/src/styles/layout.css` | Add responsive breakpoint rules |

---

## Dependencies Between Phases

```
WS1 Phase 1.1-1.6  (internal refactor)
  → Phase 1.7       (merge RAF)
  → Phase 1.8       (perf optimizations)
  → Phase 1.9       (unify math)
  → Phase 1.10      (extract to viewer-api)

WS2 Phase 2.1-2.2  (enhance shared components)
  → Phase 2.3       (FileContentViewer)
  → Phase 2.4       (log-viewer file tree)  ← depends on 2.1
  → Phase 2.5       (doc-viewer update)     ← depends on 2.1, 2.2

WS3 Phase 3.1-3.2  (responsive sidebar)    ← depends on WS2 Phase 2.2
  → Phase 3.3       (HypergraphView touch)  ← can start alongside WS1
  → Phase 3.4       (Scene3D touch)          ← depends on 3.3
```

WS1 Phases 1.1–1.9 can proceed independently of WS2/WS3.
WS1 Phase 1.10 (extraction) should complete before WS2 Phase 2.5 (doc-viewer consuming shared HypergraphView).
WS3 Phase 3.3 (touch gestures) can start in parallel with late WS1 phases.

---

## Risk Assessment

| Risk | Mitigation |
|------|-----------|
| Refactoring `useOverlayRenderer` breaks subtle rendering behavior | Incremental extraction with visual verification after each phase |
| Extraction to viewer-api introduces import/bundling issues | Test both consumer apps after each extraction step |
| File tree performance with many files | Virtualize the tree (only render visible nodes) if needed |
| Touch gestures conflict with browser defaults | `touch-action: none` + `preventDefault()` on touch events |
| Full-screen sidebar overlay blocks content on tablets | Add close button + backdrop dismiss + auto-close on selection |
| Connected-set caching becomes stale | Key cache on `[selectedIdx, layout]` — layout changes on snapshot change |

---

## Estimated Effort

| Phase | Sessions | Notes |
|-------|----------|-------|
| WS1 Phase 1.1–1.6 | 2–3 | Core refactoring |
| WS1 Phase 1.7–1.9 | 1 | Perf + math unification |
| WS1 Phase 1.10 | 1–2 | Extraction (may uncover coupling issues) |
| WS2 Phase 2.1–2.3 | 1–2 | Shared component enhancement |
| WS2 Phase 2.4–2.5 | 2 | App-specific sidebar conversion |
| WS3 Phase 3.1–3.2 | 1 | Responsive CSS + toggle |
| WS3 Phase 3.3–3.4 | 1–2 | Touch gesture implementation |
| **Total** | **~9–13 sessions** | |
