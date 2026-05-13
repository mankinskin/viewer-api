# Interview: Hypergraph Refactoring, File Tree Sync, and Mobile Support

**Date:** 2026-03-03
**Scope:** log-viewer, doc-viewer, viewer-api — frontend refactoring
**Status:** ANSWERED — ready for plan creation
**Related:** `agents/plans/20260301_PLAN_VIEWER_TOOLS_FEATURES.md` (Phases 1–8)

---

## Answers Summary

| # | Question | Answer |
|---|----------|--------|
| Q1 | Refactoring depth | **Full + extraction** — Full refactor + move core rendering engine to viewer-api |
| Q2 | DecompositionOverlay | **Delete it** — Dead code, imperative approach is the live implementation |
| Q3 | Performance priorities | **All four** — P1 dirty-flag, P2 connected-set cache, P3 merge RAF, P4 frustum culling |
| Q4 | File tree grouping | **Hybrid** — Directory tree + virtual category folders at top level |
| Q5 | Non-log file display | **CodeViewer tab** — Reuse existing code viewer tab for non-.log files |
| Q6 | Sidebar sync level | **Shell + TreeView + Tooltip** — Shared structure, app-owned behavior |
| Q7 | Gesture support | **Both 3D views** — HypergraphView + Scene3D get touch gestures |
| Q8 | Mobile sidebar | **Full-screen overlay** — Sidebar replaces entire screen temporarily |
| Q9 | Breakpoints | **Two: 768px + 480px** — Tablet mini sidebar + phone hidden sidebar |
| Q10 | Implementation order | **Risk-first** — WS1 (refactor) → WS2 (file tree) → WS3 (mobile) |

---

## Initial Implementation Plan (Draft)

### Overview

Three workstreams, ordered by dependency:

1. **WS1: HypergraphView Refactoring** — Audit & deduplicate the rendering pipeline, improve modularization and performance
2. **WS2: File Tree & Sidebar Synchronization** — Unify file tree and sidebar across doc-viewer and log-viewer, support generic file types
3. **WS3: Mobile & Responsive Support** — Collapsible sidebar, gesture-based 3D camera, responsive layout

### WS1: HypergraphView Refactoring

**Current Architecture (rendering pipeline):**
```
HypergraphView.tsx (orchestrator)
  ├── layout.ts                       — Force-directed 3D layout + focused/search-path layouts
  ├── hooks/useCamera.ts              — Orbit camera, smooth focus, axis computation
  ├── hooks/useMouseInteraction.ts    — Drag, orbit, pan, hover, selection (mouse-only)
  ├── hooks/useVisualizationState.ts  — Derives viz state from events (search/insert/read)
  ├── hooks/useOverlayRenderer.ts     — WebGPU overlay: edges, grid, decomposition, particles (550+ lines)
  ├── components/NodeLayer.tsx        — DOM node rendering with CSS classes
  ├── components/NodeInfoPanel.tsx    — Selected node details
  ├── components/NodeTooltip.tsx      — Hover tooltip
  ├── components/SearchStatePanel.tsx — Step navigation (500+ lines)
  ├── components/InsertStatePanel.tsx — Insert operation details
  ├── components/PathChainPanel.tsx   — Search path breadcrumb
  ├── components/QueryPathPanel.tsx   — Query token strip
  ├── components/GraphInfoOverlay.tsx — Graph stats overlay
  ├── components/ControlsHUD.tsx      — Mouse/keyboard hints
  ├── components/DecompositionOverlay.tsx — (unused? needs audit)
  └── utils/
      ├── math.ts                     — Ray-sphere, projection, edge key helpers
      └── nodeStyles.ts               — Width-based CSS class mapping
```

**Identified Issues & Refactoring Targets:**

| ID | Issue | Location | Severity |
|----|-------|----------|----------|
| R1 | `useOverlayRenderer` is 550+ lines — mixes GPU pipeline setup, edge coloring logic, decomposition DOM manipulation, node positioning, and animation in one monolithic closure | `hooks/useOverlayRenderer.ts` | High |
| R2 | Edge coloring logic has a 100+ line cascade of `if/else if` branches for each edge type (search path, insert, candidate, parent/child, pattern) | `useOverlayRenderer.ts` renderCallback | Medium |
| R3 | Decomposition DOM reparenting is done imperatively inside the overlay render callback — mixing rendering concerns with DOM manipulation | `useOverlayRenderer.ts` expandNode/collapseNode | High |
| R4 | Node 3D positioning (lerp + CSS transform) happens inside the overlay render callback instead of a separate animation loop | `useOverlayRenderer.ts` renderCallback | Medium |
| R5 | `math.ts` (HypergraphView) duplicates functionality from `Scene3D/math3d.ts` (both have ray-sphere, vec3 ops) | `utils/math.ts` vs `Scene3D/math3d.ts` | Low |
| R6 | `DecompositionOverlay.tsx` appears to be unused or superseded by the imperative decomposition reparenting in `useOverlayRenderer` | `components/DecompositionOverlay.tsx` | Low |
| R7 | `focusedLayout` effect in `HypergraphView.tsx` runs a nested `requestAnimationFrame` loop for camera-relative projection — should be merged with the main render loop | `HypergraphView.tsx` useEffect | Medium |
| R8 | Connected-set computation (parent/child adjacency) is recomputed every frame inside renderCallback | `useOverlayRenderer.ts` renderCallback | Medium-perf |
| R9 | `edgeDataBuf` write + GPU buffer upload happens every frame even when nothing changed | `useOverlayRenderer.ts` renderCallback | Low-perf |
| R10 | `updateNodeElMap()` linear scan of all DOM children is called at init + on every decomposition change | `useOverlayRenderer.ts` | Low-perf |

**Proposed Modularization:**

```
hooks/useOverlayRenderer.ts → split into:
  ├── gpu/pipeline.ts           — GPU pipeline/buffer setup (one-time init)
  ├── gpu/edgeRenderer.ts       — Edge instance buffer filling + coloring logic
  ├── gpu/gridRenderer.ts       — Grid line generation
  ├── animation/nodeAnimator.ts — Node lerp + CSS transform positioning
  ├── decomposition/manager.ts  — Decomposition DOM reparenting (extracted from render callback)
  └── hooks/useOverlayRenderer.ts — thin orchestrator that wires the above pieces together
```

**Performance Optimizations:**
- P1: Dirty-flag edge buffer — only rebuild edge instances when vizState, selection, or layout changes (not every frame)
- P2: Move connected-set computation out of render loop — compute in a `useMemo` keyed on `selectedIdx`
- P3: Merge focusedLayout RAF loop with main overlay render callback (eliminate double RAF)
- P4: Use `IntersectionObserver` or frustum culling to skip off-screen node transforms

### WS2: File Tree & Sidebar Synchronization

**Current State:**

| Component | doc-viewer | log-viewer | viewer-api |
|-----------|-----------|-----------|-----------|
| Sidebar shell | Custom (inline `<aside>`) | Custom `Sidebar.tsx` (flat file list) | Shared `Sidebar.tsx` (shell only) |
| Tree view | Custom `TreeItem` in `Sidebar.tsx` (with expansions, multi-type nodes) | None (flat `file-list`) | Shared `TreeView.tsx` (basic) |
| File actions | Opens docs, crate modules, source files | Opens `.log` files only | N/A (generic) |
| Code viewer | `FileViewer.tsx` (Markdown + code) | `CodeViewer.tsx` (basic) | Shared `CodeViewer.tsx` (syntax highlighting) |

**Proposed Architecture:**

```
viewer-api (shared)
  ├── components/Sidebar.tsx       — Enhanced: collapsible, resizable, responsive
  ├── components/TreeView.tsx      — Enhanced: file tree with tooltips, expandable dirs
  ├── components/FileContentViewer.tsx — NEW: generic file type visualizer
  │     ├── Renders .log files → LogViewer (log-viewer specific)
  │     ├── Renders .md files → MarkdownViewer
  │     ├── Renders .rs/.ts/.json/etc → CodeViewer (syntax highlighting)
  │     └── Renders images/unknown → Fallback viewer
  └── components/CodeViewer.tsx    — Existing (syntax highlighting)

log-viewer
  ├── Sidebar wraps shared TreeView with log-specific filters & badges
  ├── FileContentViewer dispatches to LogViewer for .log files
  └── Other file types use shared CodeViewer/MarkdownViewer

doc-viewer
  ├── Sidebar wraps shared TreeView with doc-specific node types
  ├── FileContentViewer dispatches to DocViewer for agent docs
  └── Source files use shared CodeViewer
```

**Key changes for log-viewer sidebar:**
- Convert flat file list → file tree (group by directory)
- Move file metadata (size, modified, badges) into hover tooltips
- Support opening non-.log files (JSON, TOML, source code) in code viewer tab
- Keep `.log` file behavior identical (opens in LogViewer with all existing features)

### WS3: Mobile & Responsive Support

**Sidebar:**
- Add collapse/expand toggle button (hamburger icon)
- Auto-collapse on narrow viewports (`@media (max-width: 768px)`)
- Slide-in overlay on mobile (backdrop + swipe-to-dismiss)
- Resize handle support (already exists in doc-viewer, need to add to log-viewer and share via viewer-api)

**3D Camera Gestures (HypergraphView):**
- Single-finger drag → orbit (replacing mouse right-drag)
- Two-finger pinch → zoom (replacing scroll wheel)
- Two-finger pan → pan (replacing shift+drag / middle mouse)
- Tap node → select (replacing left click)
- Double-tap → focus on node (replacing left click in highlight mode)
- Long press → context menu / info panel

---

## Interview Questions

### Batch 1: HypergraphView Refactoring Scope

**Q1: Refactoring depth — how far should we go?**

The `useOverlayRenderer` hook is the biggest target at 550+ lines. Options:

a) **Light refactor** — Extract edge coloring logic and decomposition management into separate functions (still in same file), add dirty flags for perf  
b) **Medium refactor** — Split into separate modules (gpu/, animation/, decomposition/) as proposed above, keep same hook API  
c) **Full refactor** — Medium + also extract the focused layout RAF loop from `HypergraphView.tsx` into the unified render loop, and unify `math.ts` with `Scene3D/math3d.ts`  
d) **Full + extraction** — All of the above + move the core rendering engine into `viewer-api` for reuse in doc-viewer

> **Recommendation:** Option (c) — full refactor without extraction. The HypergraphView is still tightly coupled to log-viewer signals (as noted in the existing plan). Extraction can happen later once the internal modules are clean.

**Q2: `DecompositionOverlay.tsx` — what should happen to it?**

It exists in the components directory but I don't see it referenced in `HypergraphView.tsx` or the components index. The decomposition logic is currently handled imperatively in `useOverlayRenderer`. Options:

a) **Delete it** — It's dead code, the imperative approach in useOverlayRenderer is the live implementation  
b) **Revive it** — Move decomposition logic back to a proper React component (declarative)  
c) **Keep both** — The imperative approach stays for the 3D view, DecompositionOverlay is for a future 2D mode

**Q3: Performance priority — which optimizations matter most?**

Given the current rendering pipeline, which of these should we prioritize?

a) P1: Dirty-flag edge buffer (avoid per-frame GPU writes)  
b) P2: Connected-set caching (avoid per-frame adjacency computation)  
c) P3: Merge RAF loops (eliminate double requestAnimationFrame)  
d) P4: Frustum culling for off-screen DOM nodes  

> **Recommendation:** P1 and P3 have the most impact. P2 is easy. P4 is unlikely to matter unless you have 1000+ nodes.

### Batch 2: File Tree & Sidebar Design

**Q4: Log-viewer file tree — grouping strategy**

The log-viewer currently shows a flat list of `.log` files. To convert to a tree, how should files be grouped?

a) **By directory** — Same directory structure as on disk (`target/test-logs/` → tree with folders)  
b) **By test name** — Parse test names from filenames and group by test module  
c) **By category** — Group by badges (graph, search, insert, paths) as top-level folders  
d) **Hybrid** — Directory tree with virtual category folders at the top level

> **Note:** The doc-viewer already has a rich tree with `root > category > crate > module > file` hierarchy. The viewer-api `TreeView` is a basic generic tree. We need to decide how the log-viewer's tree should work.

**Q5: Non-log file handling — what should the log-viewer content area display?**

When a user selects a non-.log file in the tree (e.g., `.rs`, `.toml`, `.json`, `.md`):

a) **CodeViewer tab** — Switch to the existing code viewer tab, show syntax-highlighted content  
b) **Inline in logs tab** — Show file content inline where log entries normally appear  
c) **New "File" tab** — Add a dedicated file viewer tab (like the code tab but for any file type)  
d) **Split view** — Show file content alongside log entries

> **Recommendation:** Option (a) — reuse the existing code tab. The doc-viewer already does this with its `FileViewer.tsx` that handles both markdown and code. We should share that component.

**Q6: Sidebar synchronization — level of unification**

How much should the doc-viewer and log-viewer sidebars share?

a) **Shell only** — Both use the shared `Sidebar` component from viewer-api as a wrapper, but internal content is completely separate  
b) **Shell + TreeView** — Both use shared `Sidebar` + shared `TreeView`, but each provides its own `TreeNode` data and click handlers  
c) **Full unification** — Shared sidebar with a superset of features (collapse/expand, resize, tree view, filters, badges, tooltips)  
d) **Same as (b) but with shared tooltip component** — TreeView gets a tooltip prop, each app provides its own tooltip content

> **Recommendation:** (d) — Share the structural components (Sidebar shell, TreeView, tooltip container) but let each app own its node types, click behavior, and tooltip content. This matches the existing viewer-api pattern.

### Batch 3: Mobile Support Scope

**Q7: Mobile support — which viewers need gesture support?**

a) **HypergraphView only** — 3D view gets touch gestures, other views use default mobile scrolling  
b) **HypergraphView + Scene3D** — Both 3D views get touch gestures  
c) **All views** — Touch gestures for navigation everywhere (including log scrolling, code viewer, etc.)  

> **Recommendation:** (b) — Both 3D views need custom touch handling. Other views work fine with native mobile scrolling.

**Q8: Sidebar mobile behavior — slide-in direction**

a) **Left slide-in** — Standard mobile pattern, sidebar slides in from the left with backdrop  
b) **Bottom sheet** — Sidebar slides up from bottom (common on mobile for secondary content)  
c) **Full-screen overlay** — Sidebar replaces the entire screen temporarily  
d) **Persistent mini sidebar** — Icons-only sidebar that expands on tap

> **Recommendation:** (a) — Standard left slide-in. Most familiar on mobile, matches VS Code's mobile behavior.

**Q9: Breakpoint strategy — at what widths should the layout change?**

a) **Single breakpoint** — `768px`: below = collapsed sidebar, above = expanded  
b) **Two breakpoints** — `768px` (tablet, mini sidebar) + `480px` (phone, hidden sidebar)  
c) **Three breakpoints** — `1024px` (collapse sidebar) + `768px` (stack layout) + `480px` (minimal UI)  
d) **Fluid** — No breakpoints, sidebar resizes proportionally with `clamp()` and `container queries`

> **Recommendation:** (b) — Two breakpoints covers the typical phone/tablet/desktop split without overcomplicating the CSS.

### Batch 4: Implementation Order

**Q10: Implementation order across the three workstreams**

These workstreams have some dependencies:

- **WS1 (HypergraphView refactoring)** is independent — can start immediately  
- **WS2 (File tree sync)** depends on deciding the shared TreeView API  
- **WS3 (Mobile support)** depends on WS2 for sidebar collapse behavior  
- **WS3 touch gestures** are independent (can be done alongside WS1)

Proposed order:

1. **WS1** — Refactor HypergraphView (2-3 sessions)
2. **WS2** — File tree & sidebar sync (2-3 sessions)
3. **WS3** — Mobile & responsive (1-2 sessions)

Or should we interleave?

a) **Sequential** — WS1 → WS2 → WS3 as above  
b) **Parallel start** — WS1 + WS3 gestures in parallel, then WS2, then WS3 responsive  
c) **Feature-first** — WS2 + WS3 first (user-visible features), then WS1 (internal quality)  
d) **Risk-first** — WS1 first (reduces tech debt, makes WS2/WS3 easier), then WS2 → WS3

> **Recommendation:** (d) — Cleaning up the HypergraphView first makes subsequent work cleaner and less likely to introduce regressions.

---

## Notes for Plan Creation

Once answers are received, the final plan will be created at:
`agents/plans/20260303_PLAN_VIEWER_REFACTORING_AND_MOBILE.md`

The plan will integrate with the existing `20260301_PLAN_VIEWER_TOOLS_FEATURES.md` since some work overlaps (Phase 2: doc-viewer integration, Phase 1B: frontend extraction).
