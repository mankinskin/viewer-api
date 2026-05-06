/**
 * DOM node CSS-transform positioning for Graph3DView.
 *
 * Projects 3D node positions to screen-space and updates DOM element styles.
 * Simplified version (no nesting, no decomposition, no viz-state dimming).
 */
import type { InternalLayout } from '../types';
import type { InteractionState } from '../hooks/useMouseInteraction';
import { worldToScreen, worldScaleAtDepth } from '../utils/math';
import { markOverlayScanDirty } from '../../WgpuOverlay/WgpuOverlay';

// ── Types ──

export interface PositionContext {
    layout: InternalLayout;
    nodeElMap: Map<number, HTMLDivElement>;
    viewProj: Float32Array;
    invSubVP: Float32Array | null;
    camPos: [number, number, number];
    vw: number;
    vh: number;
    containerRect: DOMRect;
    inter: InteractionState;
    connectedSet: Set<number>;
}

/**
 * Position all DOM node elements via CSS transforms.
 *
 * Nodes outside a margin around the viewport are culled (display: none)
 * to save layout/paint cost.
 */
export function positionDOMNodes(ctx: PositionContext): void {
    const { layout, nodeElMap, viewProj, camPos, vw, vh, inter, connectedSet } = ctx;

    const CULL_MARGIN = 200;
    let anyVisible = false;

    for (const n of layout.nodes) {
        const el = nodeElMap.get(n.index);
        if (!el) continue;

        const screen = worldToScreen([n.x, n.y, n.z], viewProj, vw, vh);
        const scale = worldScaleAtDepth(camPos, [n.x, n.y, n.z], vh);
        const pixelScale = Math.max(0.1, (scale * n.radius * 2.5) / 80);

        // Frustum culling
        if (!screen.visible || pixelScale < 0.02
            || screen.x < -CULL_MARGIN || screen.x > vw + CULL_MARGIN
            || screen.y < -CULL_MARGIN || screen.y > vh + CULL_MARGIN) {
            el.style.display = 'none';
            continue;
        }
        el.style.display = '';
        anyVisible = true;

        // Dim unconnected nodes when a node is selected
        const isSelected = n.index === inter.selectedIdx;
        const isConnected = connectedSet.has(n.index);
        const dimmed = inter.selectedIdx >= 0 && !isSelected && !isConnected;
        el.style.opacity = dimmed ? '0.15' : '1';

        el.classList.toggle('selected', isSelected);
        el.classList.toggle('span-highlighted', n.index === inter.hoverIdx);

        const zIdx = Math.round((1 - screen.z) * 1000);
        el.style.zIndex = isSelected ? '10000' : String(zIdx);
        el.style.transform = `translate(-50%, -50%) translate(${screen.x.toFixed(1)}px, ${screen.y.toFixed(1)}px) scale(${pixelScale.toFixed(3)})`;
        el.setAttribute('data-depth', screen.z.toFixed(4));
    }

    if (anyVisible) {
        markOverlayScanDirty();
    }
}
