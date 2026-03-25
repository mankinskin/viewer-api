/// <reference types="@webgpu/types" />
/**
 * WebGPU overlay renderer hook for Graph3DView.
 *
 * Simplified version of the log-viewer's useOverlayRenderer:
 * - No VisualizationState (search/insert step highlighting)
 * - No DecompositionManager (no expanded parent nodes)
 * - No nesting view (no shell/duplicate nodes)
 * - Pure graph rendering: nodes as DOM, edges as GPU quads, 3D grid
 *
 * Uses `useOverlayContext()` to access the shared GPU overlay.
 */
import { useEffect, useRef, useMemo } from 'preact/hooks';
import { mat4Multiply, mat4Inverse } from '../../../utils/math3d';
import type { InternalLayout } from '../types';
import type { CameraController } from './useCamera';
import type { InteractionState } from './useMouseInteraction';
import { buildPaletteBuffer } from '../../../effects/palette';
import { DEFAULT_THEME } from '../../../store/theme';
import {
    registerOverlayRenderer,
    unregisterOverlayRenderer,
    setOverlayParticleVP,
    setOverlayParticleViewport,
    setOverlayRefDepth,
    setOverlayWorldScale,
    setOverlayCameraPos,
    type OverlayRenderCallback,
} from '../../WgpuOverlay/WgpuOverlay';
import { useOverlayContext } from './useOverlayContext';
import { createGpuResources, destroyGpuResources } from '../gpu/pipeline';
import { buildEdgeInstances } from '../gpu/edgeBuilder';
import { animateNodes } from '../animation/nodeAnimator';
import { positionDOMNodes } from '../animation/nodePositioner';

/**
 * Hook to set up and manage the WebGPU overlay renderer for Graph3DView.
 */
export function useOverlayRenderer(
    containerRef: { current: HTMLDivElement | null },
    nodeLayerRef: { current: HTMLDivElement | null },
    layoutRef: { current: InternalLayout | null },
    camera: CameraController,
    interRef: { current: InteractionState },
    onNodeClick?: (index: number) => void,
): void {
    const gpu = useOverlayContext();
    const curLayout = layoutRef.current;

    // ── Connected-set caching ──
    const selectedIdx = interRef.current.selectedIdx;
    const connectedCache = useMemo(() => {
        const connectedSet = new Set<number>();
        if (selectedIdx >= 0 && curLayout) {
            connectedSet.add(selectedIdx);
            const sel = curLayout.nodeMap.get(selectedIdx);
            if (sel) {
                for (const ci of sel.sourceIndices) connectedSet.add(ci);
                for (const pi of sel.targetIndices) connectedSet.add(pi);
            }
        }
        return connectedSet;
    }, [selectedIdx, curLayout]);

    const connectedCacheRef = useRef(connectedCache);
    connectedCacheRef.current = connectedCache;

    useEffect(() => {
        const container = containerRef.current;
        const nodeLayer = nodeLayerRef.current;
        if (!gpu || !curLayout || !container || !nodeLayer || curLayout.nodes.length === 0) return;

        const { device, format } = gpu;

        // ── One-time GPU resource creation ──
        const res = createGpuResources(device, format, Math.max(curLayout.edges.length, 1));

        // ── Pre-allocated per-frame scratch buffers ──
        const postMatrix = new Float32Array(16);
        const invPostMatrix = new Float32Array(16);
        const camBuf = new Float32Array(32);
        let cachedPaletteColors: unknown = null;
        let cachedPaletteBuf: Float32Array | null = null;

        // ── Dirty-flag state ──
        let prevSelectedIdx = -2;
        let prevHoverIdx = -2;
        let prevConnectedRef: Set<number> | null = null;
        let nodesMoving = true;

        // ── Node element map (built from nodeLayer DOM) ──
        const nodeElMap = new Map<number, HTMLDivElement>();

        // ── Overlay render callback ──
        const renderCallback: OverlayRenderCallback = (pass, dev, time, dt, canvasW, canvasH) => {
            const rect = container.getBoundingClientRect();
            const vx = Math.max(0, Math.round(rect.left));
            const vy = Math.max(0, Math.round(rect.top));
            const vw = Math.min(Math.round(rect.width), canvasW - vx);
            const vh = Math.min(Math.round(rect.height), canvasH - vy);
            if (vw <= 0 || vh <= 0) return;

            pass.setViewport(vx, vy, vw, vh, 0, 1);
            pass.setScissorRect(vx, vy, vw, vh);

            const { viewProj, camPos } = camera.getViewProj(vw, vh, dt);
            const inter = interRef.current;

            // ── Particle system VP setup ──
            const W = canvasW, H = canvasH;
            const sx = vw / W, sy = vh / H;
            const tx = (2 * vx + vw) / W - 1;
            const ty = 1 - (2 * vy + vh) / H;
            postMatrix.fill(0);
            postMatrix[0] = sx; postMatrix[5] = sy; postMatrix[10] = 1; postMatrix[15] = 1;
            postMatrix[12] = tx; postMatrix[13] = ty;
            const fullVP = mat4Multiply(postMatrix, viewProj);
            const invSubVP = mat4Inverse(viewProj);
            if (invSubVP) {
                invPostMatrix.fill(0);
                invPostMatrix[0] = 1 / sx; invPostMatrix[5] = 1 / sy;
                invPostMatrix[10] = 1; invPostMatrix[15] = 1;
                invPostMatrix[12] = -tx / sx; invPostMatrix[13] = -ty / sy;
                const fullInvVP = mat4Multiply(invSubVP, invPostMatrix);
                setOverlayParticleVP(fullVP, fullInvVP);
                setOverlayCameraPos(camPos[0], camPos[1], camPos[2]);
            }
            setOverlayParticleViewport(vx, vy, vw, vh);

            // Reference depth + world scale
            const camState = camera.stateRef.current;
            const [ttx, tty, ttz] = camState.target;
            const vp = viewProj;
            const tw = vp[3]! * ttx + vp[7]! * tty + vp[11]! * ttz + vp[15]!;
            const refZ = tw > 0.001 ? (vp[2]! * ttx + vp[6]! * tty + vp[10]! * ttz + vp[14]!) / tw : 0;
            setOverlayRefDepth(refZ);
            const dist = Math.sqrt((camPos[0] - ttx) ** 2 + (camPos[1] - tty) ** 2 + (camPos[2] - ttz) ** 2);
            setOverlayWorldScale((2 * dist * Math.tan(Math.PI / 8)) / vh);

            // ── Sync node element map from DOM ──
            const nodeEls = nodeLayer.querySelectorAll<HTMLDivElement>('[data-node-idx]');
            nodeElMap.clear();
            for (const el of nodeEls) {
                const idx = Number(el.getAttribute('data-node-idx'));
                nodeElMap.set(idx, el);
            }

            // ── Handle click callbacks ──
            if (onNodeClick && inter.selectedIdx !== prevSelectedIdx && inter.selectedIdx >= 0) {
                onNodeClick(inter.selectedIdx);
            }

            // ── Animate nodes ──
            nodesMoving = animateNodes(curLayout.nodes, dt);

            // ── Position DOM nodes ──
            const cached = connectedCacheRef.current;
            positionDOMNodes({
                layout: curLayout,
                nodeElMap,
                viewProj,
                invSubVP,
                camPos,
                vw,
                vh,
                containerRect: rect,
                inter,
                connectedSet: cached,
            });

            // ── Build and upload edge instances (dirty-flag optimization) ──
            const edgeDirty = nodesMoving
                || inter.dragIdx >= 0
                || inter.selectedIdx !== prevSelectedIdx
                || inter.hoverIdx !== prevHoverIdx
                || cached !== prevConnectedRef;

            if (edgeDirty) {
                buildEdgeInstances(res.edgeDataBuf, curLayout, inter.selectedIdx, cached);
                dev.queue.writeBuffer(
                    res.edgeIB,
                    0,
                    res.edgeDataBuf.buffer,
                    res.edgeDataBuf.byteOffset,
                    res.edgeDataBuf.byteLength,
                );
                prevSelectedIdx = inter.selectedIdx;
                prevHoverIdx = inter.hoverIdx;
                prevConnectedRef = cached;
            }

            // ── Camera + palette uniforms ──
            camBuf.set(viewProj, 0);
            camBuf.set([camPos[0], camPos[1], camPos[2], 0], 16);
            camBuf.set([time, 0, 0, 0], 20);
            dev.queue.writeBuffer(res.camUB, 0, camBuf);

            const currentColors = DEFAULT_THEME;
            if (currentColors !== cachedPaletteColors) {
                cachedPaletteColors = currentColors;
                cachedPaletteBuf = buildPaletteBuffer(currentColors);
            }
            dev.queue.writeBuffer(
                res.paletteUB,
                0,
                cachedPaletteBuf!.buffer,
                cachedPaletteBuf!.byteOffset,
                cachedPaletteBuf!.byteLength,
            );

            // ── Draw grid then edges ──
            pass.setPipeline(res.gridPipeline);
            pass.setVertexBuffer(0, res.quadVB);
            pass.setVertexBuffer(1, res.gridIB);
            pass.setBindGroup(0, res.camBG);
            pass.draw(6, res.gridCount);

            if (curLayout.edges.length > 0) {
                pass.setPipeline(res.edgePipeline);
                pass.setVertexBuffer(0, res.quadVB);
                pass.setVertexBuffer(1, res.edgeIB);
                pass.setBindGroup(0, res.camBG);
                pass.draw(6, curLayout.edges.length);
            }

            pass.setViewport(0, 0, canvasW, canvasH, 0, 1);
            pass.setScissorRect(0, 0, canvasW, canvasH);
        };

        registerOverlayRenderer(renderCallback);

        return () => {
            unregisterOverlayRenderer(renderCallback);
            destroyGpuResources(res);
        };
    }, [gpu, curLayout, camera]);
}
