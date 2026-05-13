Extract a fully self-contained `Graph3DView` component into `viewer-api/frontend`. This is not a thin wrapper — it owns camera, layout, interaction, animation, and GPU rendering. Log-viewer only adds its thin hypergraph model layer on top. Any viewer can use Graph3DView independently.

## What Gets Extracted (all hooks — A5)

From `log-viewer/frontend/src/components/HypergraphView/`:
- **All hooks:** `useCamera`, `useMouseInteraction`, `useTouchInteraction`, `useOverlayRenderer` (updated to use `useOverlayContext()`)
- **Animation:** `animation/nodeAnimator.ts`, `animation/nodePositioner.ts`
- **UI:** `components/NodeLayer.tsx`
- **Utilities:** `utils/math.ts`, `utils/nodeStyles.ts`
- **New layout engines (not ports of existing layout.ts):**
  - `layout/forceDirected.ts` — new simple spring-electrical simulation on `Graph3DNode[]`
  - `layout/hierarchical.ts` — new BFS-depth layered layout, good for DAGs

The layout prop `'force-directed' | 'hierarchical'` selects the engine at render time.

## What Stays in log-viewer

All hypergraph-specific logic: `useVisualizationState`, `useNestingState`, `DecompositionManager`, `gpu/edgeBuilder.ts`, `gpu/constants.ts`, `nesting/`, `search-path/`, `layout.ts` (coupled to `HypergraphSnapshot`), all HUD panels.

## Graph3DView API Surface

```typescript
interface Graph3DNode {
  id: string;
  label: string;
  color?: string;
  size?: number;
  data?: unknown;
}

interface Graph3DEdge {
  source: string;
  target: string;
  color?: string;
  style?: 'solid' | 'dashed';
}

interface Graph3DViewProps {
  nodes: Graph3DNode[];
  edges: Graph3DEdge[];
  onNodeClick?: (node: Graph3DNode) => void;
  selectedNodeId?: string;
  layoutMode?: 'force' | 'hierarchical';
}
```

## Depends On

b3d250d5 — WgpuOverlay must already be in viewer-api before this can build on top of it.
