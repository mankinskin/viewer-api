# Plan: Extract WgpuOverlay / GPU Pipeline to viewer-api

**Ticket:** 1789cdfa-cd7e-45c4-a683-815b80c39970
**Component:** viewer-api (target), log-viewer (source), ticket-viewer (consumer)
**Risk:** High

---

## Goal

Extract the entire GPU rendering pipeline (WgpuOverlay, shaders, effects, 3D math, element scanning) from `log-viewer/frontend` to `viewer-api/frontend` as shared infrastructure. Then import it back in log-viewer and use it in ticket-viewer to render the dependency graph with 3D GPU rendering replacing the current SVG `GraphView`.

## Current Architecture

The log-viewer frontend contains a layered GPU rendering system:

```
WgpuOverlay (canvas behind DOM)
├── gpu-init.ts          — WebGPU device + pipeline creation
├── gpu-buffers.ts       — Uniform/element/particle/palette buffers
├── gpu-render-loop.ts   — 3-pass render loop (compute, background, overlays, particles)
├── element-scanner.ts   — DOM element tracking via MutationObserver/IntersectionObserver
├── element-types.ts     — CSS selectors, element kind constants, buffer layouts
├── overlay-api.ts       — Signals: gpuOverlayEnabled, fxEnabled, overlayGpu; renderer callbacks
├── thumbnail-capture.ts — JPEG frame capture for theme previews
└── WgpuOverlay.tsx      — Preact component wiring everything together

effects/
├── palette.ts           — buildPaletteBuffer() → Float32Array for GPU
├── palette.wgsl         — ThemePalette struct (24 vec4 slots)
└── particle-shading.wgsl — Spark/ember/beam/glitter RGBA functions

Scene3D/
├── math3d.ts            — Vec3/Mat4 math library (column-major, WebGPU conventions)
├── Scene3D.tsx          — 3D demo cubes (uses overlay callback)
└── scene3d.wgsl         — Blinn-Phong cubes + grid shader

HypergraphView/ (log-viewer-specific, stays)
├── gpu/pipeline.ts      — Edge/grid pipeline using WgpuOverlay
├── gpu/edgeBuilder.ts   — Edge classification + instance buffers
├── gpu/constants.ts     — Colors, geometry constants
├── hooks/useOverlayRenderer.ts — Per-frame rendering orchestration
├── hooks/useCamera.ts   — Orbit camera + focus animation
├── hooks/useMouseInteraction.ts — Selection, dragging, hit detection
└── ... (nesting, decomposition, search paths — all log-viewer-specific)
```

## Extraction Plan

### Phase 1: Move WgpuOverlay infrastructure to viewer-api

**Files to move** (log-viewer → viewer-api):

| Source (log-viewer/frontend/src/) | Target (viewer-api/frontend/src/) |
|---|---|
| `components/WgpuOverlay/WgpuOverlay.tsx` | `components/WgpuOverlay/WgpuOverlay.tsx` |
| `components/WgpuOverlay/gpu-init.ts` | `components/WgpuOverlay/gpu-init.ts` |
| `components/WgpuOverlay/gpu-buffers.ts` | `components/WgpuOverlay/gpu-buffers.ts` |
| `components/WgpuOverlay/gpu-render-loop.ts` | `components/WgpuOverlay/gpu-render-loop.ts` |
| `components/WgpuOverlay/element-scanner.ts` | `components/WgpuOverlay/element-scanner.ts` |
| `components/WgpuOverlay/element-types.ts` | `components/WgpuOverlay/element-types.ts` |
| `components/WgpuOverlay/overlay-api.ts` | `components/WgpuOverlay/overlay-api.ts` |
| `components/WgpuOverlay/thumbnail-capture.ts` | `components/WgpuOverlay/thumbnail-capture.ts` |

**Shader files to move:**

| Source | Target |
|---|---|
| `effects/palette.ts` | `effects/palette.ts` |
| `effects/palette.wgsl` | `effects/palette.wgsl` |
| `effects/particle-shading.wgsl` | `effects/particle-shading.wgsl` |

All WGSL shader files that `gpu-init.ts` concatenates (background.wgsl, particles.wgsl, compute.wgsl, types.wgsl, noise.wgsl) also need to move.

**3D math to move:**

| Source | Target |
|---|---|
| `components/Scene3D/math3d.ts` | `utils/math3d.ts` |
| `components/Scene3D/Scene3D.tsx` | `components/Scene3D/Scene3D.tsx` |
| `components/Scene3D/scene3d.wgsl` | `components/Scene3D/scene3d.wgsl` |

**App schema system (A2):**

Rather than a bare `elementSelectors` prop override, we use a typed **app schema** that packages the full descriptor of what a viewer cares about. This gives viewers a clean, first-class declaration of their GPU rendering model:

```typescript
// viewer-api/frontend/src/components/WgpuOverlay/schemas.ts

export interface ElementSelectorEntry {
  selector: string;
  kind: number;
  hue: number;
}

/** Full descriptor of a viewer's GPU rendering model. */
export interface AppSchema {
  /** CSS selectors to track + their GPU kind/hue metadata. */
  selectors: ElementSelectorEntry[];
  /** Named kind constants for use in selector entries and GPU pipelines. */
  kinds: Record<string, number>;
  /** Optional particle range overrides. */
  particleRanges?: Array<{ name: string; start: number; end: number }>;
}

/** Minimal schema for generic HTML structure — viewer-api default. */
export const MINIMAL_SCHEMA: AppSchema = {
  selectors: [
    { selector: 'header, [role="banner"]', kind: KIND_STRUCTURAL, hue: 0.58 },
    { selector: 'aside, [role="complementary"]', kind: KIND_STRUCTURAL, hue: 0.55 },
    { selector: 'main, [role="main"]', kind: KIND_STRUCTURAL, hue: 0.52 },
    { selector: 'nav, [role="navigation"]', kind: KIND_STRUCTURAL, hue: 0.50 },
    { selector: '[aria-selected="true"], .selected', kind: KIND_SELECTED, hue: 0.12 },
  ],
  kinds: { STRUCTURAL: 0, SELECTED: 6 },
};
```

```typescript
// tools/log-viewer/frontend/src/gpu-schema.ts — extends minimal
export const LOG_VIEWER_SCHEMA: AppSchema = {
  selectors: [
    ...MINIMAL_SCHEMA.selectors,
    { selector: '.log-entry.level-error', kind: KIND_ERROR, hue: 0.0 },
    { selector: '.log-entry.level-warn', kind: KIND_WARN, hue: 0.10 },
    { selector: '.log-entry.level-info', kind: KIND_INFO, hue: 0.53 },
    { selector: '.log-entry.level-debug', kind: KIND_DEBUG, hue: 0.60 },
    { selector: '.log-entry.level-trace', kind: KIND_DEBUG, hue: 0.65 },
    { selector: '.log-entry.span-highlighted', kind: KIND_SPAN_HL, hue: 0.30 },
    // ... effect preview selectors
  ],
  kinds: { ...MINIMAL_SCHEMA.kinds, ERROR: 1, WARN: 2, INFO: 3, DEBUG: 4, SPAN_HL: 5, PANIC: 7 },
};

// tools/ticket-viewer/frontend/src/gpu-schema.ts — extends minimal
export const TICKET_VIEWER_SCHEMA: AppSchema = {
  selectors: [
    ...MINIMAL_SCHEMA.selectors,
    { selector: '.ticket-tree .tree-item-row.selected', kind: KIND_SELECTED, hue: 0.12 },
    { selector: '[data-ticket-state="open"]', kind: KIND_INFO, hue: 0.60 },
    { selector: '[data-ticket-state="in-progress"]', kind: KIND_WARN, hue: 0.10 },
    { selector: '[data-ticket-state="done"]', kind: KIND_DEBUG, hue: 0.35 },
    { selector: '[data-ticket-state="blocked"]', kind: KIND_ERROR, hue: 0.0 },
  ],
  kinds: { ...MINIMAL_SCHEMA.kinds, ERROR: 1, WARN: 2, INFO: 3, DEBUG: 4 },
};
```

`WgpuOverlay` accepts `schema: AppSchema` as a required prop. Each app passes its schema at mount. The element scanner uses `schema.selectors` instead of the hardcoded array.

**2DGraph moved to viewer-api (A7):**

The existing SVG `GraphView.tsx` in ticket-viewer should be extracted to viewer-api as a generic `Graph2DView` component for simple, non-GPU dependency/flow graph rendering. This is appropriate for document contexts (doc-viewer, lightweight embeds) and as a graceful fallback when WebGPU is unavailable.

Files to move/create:
- `tools/ticket-viewer/frontend/src/components/GraphView.tsx` → `tools/viewer-api/frontend/src/components/Graph2DView/Graph2DView.tsx`
- Generalize `STATE_COLORS` → accept a `nodeColorFn: (node) => string` prop
- Export `Graph2DNode`, `Graph2DEdge`, `Graph2DViewProps` from viewer-api index

### Phase 2: Update log-viewer imports + Isolated rendering contexts (A3)

**Isolated rendering contexts — problem and solution:**

**The problem:** `overlay-api.ts` currently exports module-level signal singletons (`gpuOverlayEnabled`, `fxEnabled`, `overlayGpu`) and a module-level `Set<OverlayRenderCallback>`. ES modules are evaluated once and cached — so all consumers in the same JS runtime share the same instances. Today this is fine because each viewer is a separate Vite bundle with its own module registry. But it creates hidden coupling:

- If two `<WgpuOverlay>` components ever mounted simultaneously (e.g., embedded panels, future multi-panel layout), they'd share one GPU context and one callback registry.
- `gpuOverlayEnabled` controls ALL overlays globally — there's no per-instance toggle.
- Unit tests can't inject a clean state without module-level teardown tricks.

**Direction A — Factory function (chosen approach):**

```typescript
// viewer-api/frontend/src/components/WgpuOverlay/overlay-context.ts

export interface OverlayContext {
  gpuOverlayEnabled: Signal<boolean>;
  fxEnabled: Signal<boolean>;
  overlayGpu: Signal<GpuHandle | null>;
  registerRenderer(cb: OverlayRenderCallback): void;
  unregisterRenderer(cb: OverlayRenderCallback): void;
  getCallbacks(): ReadonlySet<OverlayRenderCallback>;
  captureOverlayThumbnail(): Promise<string>;
  hasCaptureRequest(): boolean;
  consumeCaptureRequest(): ((url: string) => void) | null;
  markOverlayScanDirty(): void;
  resetOverlayParticles(): void;
}

export function createOverlayContext(): OverlayContext { /* ... */ }
```

Combined with a Preact Context Provider for zero prop-drilling:

```typescript
const OverlayCtx = createContext<OverlayContext | null>(null);

export function OverlayProvider({ children }: { children: ComponentChildren }) {
  const ctx = useMemo(() => createOverlayContext(), []);
  return <OverlayCtx.Provider value={ctx}>{children}</OverlayCtx.Provider>;
}

export function useOverlayContext(): OverlayContext {
  const ctx = useContext(OverlayCtx);
  if (!ctx) throw new Error('useOverlayContext must be used inside <OverlayProvider>');
  return ctx;
}
```

**Direction B — Keep globals (deferred option):** Add a comment noting "one GPU context per JS module registry" as an explicit constraint. Valid today but not forward-looking.

**Why Direction A:** Factory + Preact Context makes the contract testable and eliminates hidden shared state. Both log-viewer and ticket-viewer `App.tsx` simply wrap with `<OverlayProvider>`. `WgpuOverlay` and all hooks use `useOverlayContext()`. This replaces every `import { gpuOverlayEnabled } from '../overlay-api'` with a context read.

**Implementation impact on b3d250d5:**
- Replace module-level exports in `overlay-api.ts` with `createOverlayContext()` factory
- Add `OverlayProvider` + `useOverlayContext()` to viewer-api exports
- Update `WgpuOverlay.tsx` to read from `useOverlayContext()` instead of module globals
- Update log-viewer `App.tsx` and ticket-viewer `App.tsx` to wrap with `<OverlayProvider>`

**Import updates** (after context change):

```typescript
// Before:
import { WgpuOverlay } from '../WgpuOverlay/WgpuOverlay';
import { registerOverlayRenderer } from '../WgpuOverlay/overlay-api';
import { buildPaletteBuffer } from '../../effects/palette';

// After:
import { WgpuOverlay, registerOverlayRenderer, buildPaletteBuffer } from '@context-engine/viewer-api-frontend';
```

The HypergraphView GPU pipeline (`gpu/pipeline.ts`, `gpu/edgeBuilder.ts`, `gpu/constants.ts`) stays in log-viewer — it contains hypergraph-specific edge rendering. It imports shared GPU utilities from viewer-api.

### Phase 3: Create generic Graph3DView for viewer-api

Graph3DView is a **fully self-contained 3D graph viewport** in viewer-api. It owns camera, layout, interaction, animation, and GPU rendering. Log-viewer only adds its thin hypergraph model layer on top. The component is a first-class standalone viewer capability.

```typescript
// viewer-api/frontend/src/components/Graph3DView/Graph3DView.tsx

export interface Graph3DNode {
  id: string;
  label: string;
  color?: [number, number, number];
  radius?: number;
  group?: string;
}

export interface Graph3DEdge {
  from: string;
  to: string;
  color?: [number, number, number];
  label?: string;
  style?: 'solid' | 'dashed';
}

export interface Graph3DViewProps {
  nodes: Graph3DNode[];
  edges: Graph3DEdge[];
  selectedId?: string;
  onSelect?: (nodeId: string) => void;
  /** Layout algorithm (A4). Default: 'force-directed'. */
  layout?: 'force-directed' | 'hierarchical';
  /** Additional render children (HUD panels, etc.) */
  children?: ComponentChildren;
}
```

**Layout algorithms (A4):** Graph3DView ships with two layout engines, both working on the generic `Graph3DNode[]` type:

1. **`force-directed`** — New simple spring-electrical simulation on `Graph3DNode[]`. Does NOT port the existing `HypergraphView/layout.ts` (which is coupled to `HypergraphSnapshot` / `VizPathGraph`). New implementation: repulsion between all node pairs, attraction along edges, gravity toward origin, damping. Default for ticket dependency graphs.

2. **`hierarchical`** — Layered hierarchical placement derived from the depth-based layout logic in `GraphView.tsx` but in 3D. Nodes are bucketed by BFS depth, spread across XZ plane per layer, Y proportional to depth. Suitable for strict DAG display. Works well for dependency trees.

Layout is chosen via the `layout` prop. Both engines produce `Graph3DNode` position updates (x/y/z). The GPU pipeline and hooks are layout-agnostic.

**All hooks move to viewer-api (A5):** The entire hooks layer of HypergraphView moves. viewer-api Graph3DView is fully capable of standalone 3D rendering. Log-viewer only needs the thin hypergraph model layer.

**Moves to viewer-api** (all hooks + supporting modules):
- `hooks/useCamera.ts` — Orbit camera, focus animation (fully generic)
- `hooks/useMouseInteraction.ts` — Selection, dragging, hit detection (generic; hypergraph-specific data structs removed)
- `hooks/useTouchInteraction.ts` — Touch equivalent (fully generic)
- `hooks/useOverlayRenderer.ts` — GPU render callback orchestration (generic via `useOverlayContext()`)
- `animation/nodeAnimator.ts` — Position lerp (generic)
- `animation/nodePositioner.ts` — 3D→screen projection (generic)
- `components/NodeLayer.tsx` — HTML label overlay (generic)
- `utils/math.ts` — Math utilities needed by above
- `utils/nodeStyles.ts` — Generic node styling utilities
- New: `layout/forceDirected.ts` — New generic force layout
- New: `layout/hierarchical.ts` — New generic hierarchical layout

**Stays in log-viewer** (hypergraph-specific):
- `hooks/useVisualizationState.ts` — search/insert event parsing (consumes `HypergraphSnapshot`)
- `hooks/useNestingState.ts` — shell containers, duplicate nodes
- `decomposition/manager.ts` — expand/collapse DOM reparenting
- `gpu/edgeBuilder.ts` — 8+ edge type classification (search path, candidate, insert)
- `gpu/constants.ts` — hypergraph-specific colors
- `nesting/` — shell layout, duplicate manager, edge highlights
- `search-path/` — search path reconstruction + edge highlighting
- `layout.ts` — Stays (uses `HypergraphSnapshot`; log-viewer's thin layout layer)
- All log-viewer HUD panels (SearchStatePanel, InsertStatePanel, ControlsHUD, PathChainPanel, QueryPathPanel)

### Phase 4: GPU Dependency Graph in ticket-viewer

Replace the SVG `GraphView` with `Graph3DView`:

```typescript
// ticket-viewer: DependencyGraph.tsx
import { Graph3DView, Graph3DNode, Graph3DEdge } from '@context-engine/viewer-api-frontend';

function DependencyGraph({ rootId, workspace, authToken }) {
  const { nodes, edges, loading, error } = useTicketSubgraph(rootId, workspace, authToken);

  const graphNodes: Graph3DNode[] = nodes.map(n => ({
    id: n.id,
    label: n.title || n.id.slice(0, 8),
    color: STATE_COLORS[n.state] || DEFAULT_COLOR,
    group: n.state,
  }));

  const graphEdges: Graph3DEdge[] = edges.map(e => ({
    from: e.from,
    to: e.to,
    color: EDGE_KIND_COLORS[e.kind],
    label: e.kind,
  }));

  return (
    <Graph3DView
      nodes={graphNodes}
      edges={graphEdges}
      selectedId={rootId}
      onSelect={handleSelectTicket}
      layout="hierarchical"
    />
  );
}
```

The `GraphView.tsx` (current SVG implementation) can be kept as fallback for non-WebGPU browsers, or removed.

### Package dependency changes

```jsonc
// viewer-api/frontend/package.json — add:
{
  "devDependencies": {
    "@webgpu/types": "^0.1.x"
  }
}
```

## Sub-ticket Breakdown

| Sub-ticket | Scope | Depends on |
|---|---|---|
| **B1: Move WgpuOverlay + shaders to viewer-api** | Phase 1 + 2: file moves, app schema system, isolated OverlayContext, OverlayProvider, import updates, Graph2DView to viewer-api | — |
| **B2: Extract generic Graph3DView to viewer-api** | Phase 3: all hooks move, both layout engines (force-directed + hierarchical), Graph3DView component | B1 |
| **B3: GPU dependency graph in ticket-viewer** | Phase 4: replace SVG GraphView with Graph3DView, TICKET_VIEWER_SCHEMA, DependencyGraph component | B2 |

**B3 is blocked on B2 (A6).** No stub — implement Graph3DView first.

## Risks

1. **WGSL shader imports**: Shaders are concatenated as string literals. Moving them requires updating the concatenation paths in `gpu-init.ts`. Vite handles WGSL via `?raw` imports — this works across packages if configured.
2. **App schema coupling**: The schema system must be consistent — log-viewer and ticket-viewer schemas must not reuse kind numbers that conflict. Kind constants live in viewer-api's `MINIMAL_SCHEMA` and are extended by viewers.
3. **OverlayContext migration**: All existing imports of module-level globals (`gpuOverlayEnabled`, `fxEnabled`, etc.) in log-viewer must be replaced with `useOverlayContext()`. This touches ~10 files in log-viewer.
4. **Circular dependencies**: viewer-api must not import from log-viewer. All shared code flows one direction.
5. **WebGPU availability**: ticket-viewer needs the same WebGPU detection + graceful fallback. Graph2DView (moved to viewer-api from ticket-viewer's SVG GraphView) serves as the non-GPU fallback.
6. **Bundle size**: Moving GPU code + all hooks to viewer-api increases the shared package. Tree-shaking should handle this if imports are clean, but verify.
7. **Layout engine coupling**: The existing `HypergraphView/layout.ts` depends on `HypergraphSnapshot` and stays in log-viewer. The new generic force-directed + hierarchical engines in viewer-api are written from scratch but can share the spring simulation math.

## Files Changed (estimated)

~40 files moved/modified across viewer-api, log-viewer, and ticket-viewer.

### B1 scope additions (beyond original estimate):
- `tools/viewer-api/frontend/src/components/WgpuOverlay/schemas.ts` — new (AppSchema, MINIMAL_SCHEMA)
- `tools/viewer-api/frontend/src/components/WgpuOverlay/overlay-context.ts` — new (createOverlayContext, OverlayProvider, useOverlayContext)
- `tools/viewer-api/frontend/src/components/Graph2DView/Graph2DView.tsx` — moved from ticket-viewer GraphView.tsx + generalized
- `tools/viewer-api/frontend/src/index.ts` — re-export Graph2DView, AppSchema, OverlayProvider, useOverlayContext

### B2 scope additions:
- `tools/viewer-api/frontend/src/components/Graph3DView/layout/forceDirected.ts` — new
- `tools/viewer-api/frontend/src/components/Graph3DView/layout/hierarchical.ts` — new
- All hooks + animation + NodeLayer from HypergraphView (7+ files moved, adapted)
- `utils/math.ts`, `utils/nodeStyles.ts` moved from HypergraphView/utils/
