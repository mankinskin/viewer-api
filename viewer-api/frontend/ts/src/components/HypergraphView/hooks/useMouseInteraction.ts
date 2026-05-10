/**
 * Mouse interaction hook for 3D hypergraph view.
 * Handles node dragging, camera orbiting/panning, hover detection, and selection.
 */
import { useRef, useEffect, useState, useCallback } from 'preact/hooks';
import type { Vec3 } from '../../Scene3D/math3d';
import { mat4Inverse, screenToRay, rayPlaneIntersectGeneral, vec3Sub, vec3Normalize } from '../../Scene3D/math3d';
import { raySphere } from '../utils/math';
import type { GraphLayout, LayoutNode } from '../layout';
import type { CameraController } from './useCamera';

const INTERACTION_BLOCK_SELECTOR = [
    '[data-graph-passthrough="false"]',
    '.graph-settings-overlay',
    '.search-state-panel',
    '.insert-state-panel',
    '.path-chain-panel',
    '.qp-panel',
    '.node-info-panel',
    '.decomposition-panel',
    '.hypergraph-info-panel',
    '.hypergraph-hud',
    '.theme-settings',
    '.theme-settings-layout',
].join(', ');

function eventTargetElement(target: EventTarget | null): HTMLElement | null {
    if (!target) return null;
    if (target instanceof HTMLElement) return target;
    if (target instanceof Node) return target.parentElement;
    return null;
}

function isInteractionBlocked(target: EventTarget | null): boolean {
    const el = eventTargetElement(target);
    return !!el?.closest(INTERACTION_BLOCK_SELECTOR);
}

export interface InteractionState {
    dragIdx: number;
    dragPlanePoint: Vec3;
    dragPlaneNormal: Vec3;
    dragOffset: Vec3;
    orbiting: boolean;
    panning: boolean;
    lastMX: number;
    lastMY: number;
    mouseX: number;
    mouseY: number;
    selectedIdx: number;
    hoverIdx: number;
    /** Node index hit on mouseDown in layout mode; selection deferred to mouseUp. -1 = none. */
    clickedNode: number;
    /** Screen-space mouse position at mouseDown (for click vs drag threshold). */
    downMX: number;
    downMY: number;
}

export interface TooltipData {
    x: number;
    y: number;
    node: LayoutNode;
}

export interface MouseInteractionResult {
    selectedIdx: number;
    setSelectedIdx: (idx: number) => void;
    hoverIdx: number;
    tooltip: TooltipData | null;
    interRef: { current: InteractionState };
}

/**
 * Hook for handling mouse interactions in the hypergraph view.
 */
export function useMouseInteraction(
    containerRef: { current: HTMLDivElement | null },
    layoutRef: { current: GraphLayout | null },
    camera: CameraController,
    autoLayoutRef?: { current: boolean },
): MouseInteractionResult {
    const [selectedIdx, setSelectedIdx] = useState(-1);
    const [hoverIdx, setHoverIdx] = useState(-1);
    const [tooltip, setTooltip] = useState<TooltipData | null>(null);

    const interRef = useRef<InteractionState>({
        dragIdx: -1,
        dragPlanePoint: [0, 0, 0] as Vec3,
        dragPlaneNormal: [0, 0, 1] as Vec3,
        dragOffset: [0, 0, 0] as Vec3,
        orbiting: false,
        panning: false,
        lastMX: 0,
        lastMY: 0,
        mouseX: 0,
        mouseY: 0,
        selectedIdx: -1,
        hoverIdx: -1,
        clickedNode: -1,
        downMX: 0,
        downMY: 0,
    });

    // Sync external selection changes to internal state
    const setSelectedIdxWithSync = useCallback((idx: number) => {
        interRef.current.selectedIdx = idx;
        setSelectedIdx(idx);
    }, []);

    useEffect(() => {
        const container = containerRef.current;
        const layout = layoutRef.current;
        if (!container || !layout) return;

        const inter = interRef.current;

        // Minimum screen-space distance (px) before a mousedown+move is
        // considered a real drag rather than a sloppy click.
        const DRAG_THRESHOLD = 5;

        const onMouseDown = (e: MouseEvent) => {
            if (isInteractionBlocked(e.target)) {
                inter.dragIdx = -1;
                inter.orbiting = false;
                inter.panning = false;
                inter.clickedNode = -1;
                return;
            }

            const rect = container.getBoundingClientRect();
            inter.mouseX = e.clientX - rect.left;
            inter.mouseY = e.clientY - rect.top;
            inter.lastMX = e.clientX;
            inter.lastMY = e.clientY;
            inter.downMX = e.clientX;
            inter.downMY = e.clientY;

            const cw = container.clientWidth;
            const ch = container.clientHeight;

            if (e.button === 1 || (e.button === 0 && e.shiftKey)) {
                // Middle mouse or Shift+Left → pan
                inter.panning = true;
                camera.cancelFocus();
                e.preventDefault();
            } else if (e.button === 0) {
                const { viewProj } = camera.getViewProj(cw, ch);
                const invVP = mat4Inverse(viewProj);
                if (invVP) {
                    const ray = screenToRay(inter.mouseX, inter.mouseY, cw, ch, invVP);

                    // Check if the click landed on an expanded decomposition
                    // node (but NOT on a decomp child — those are handled by the
                    // DecompositionManager and never bubble here).  When it does,
                    // start a drag on the parent directly instead of relying on
                    // the ray-sphere hit test which has a tiny 3D radius.
                    const expandedEl = (e.target as HTMLElement).closest?.('.hg-expanded');
                    let expandedHit = false;
                    if (expandedEl) {
                        const nodeIdx = Number(expandedEl.getAttribute('data-node-idx'));
                        const node = layout.nodeMap.get(nodeIdx);
                        if (node) {
                            if (autoLayoutRef?.current) {
                                inter.clickedNode = nodeIdx;
                            } else {
                                inter.dragIdx = nodeIdx;
                                const nodePos: Vec3 = [node.x, node.y, node.z];
                                const camPos = camera.getCamPos();
                                inter.dragPlaneNormal = vec3Normalize(vec3Sub(camPos, nodePos));
                                inter.dragPlanePoint = nodePos;
                                const pt = rayPlaneIntersectGeneral(ray, nodePos, inter.dragPlaneNormal);
                                if (pt) inter.dragOffset = [node.x - pt[0], node.y - pt[1], node.z - pt[2]];
                            }
                            expandedHit = true;
                            e.preventDefault();
                        }
                    }

                    if (!expandedHit) {
                        const nodeEl = (e.target as HTMLElement).closest?.('.hg-node');
                        const bestIdx = nodeEl ? Number(nodeEl.getAttribute('data-node-idx')) : -1;
                    if (bestIdx >= 0) {
                        const node = layout.nodeMap.get(bestIdx);
                        if (node) {
                            if (autoLayoutRef?.current) {
                                // In auto-layout mode, nodes are positioned by the
                                // focused-layout algorithm — dragging would fight
                                // the layout projection. Record the hit; select on mouseUp.
                                inter.clickedNode = bestIdx;
                            } else {
                                inter.dragIdx = bestIdx;
                                // Drag plane perpendicular to view direction through the node
                                const nodePos: Vec3 = [node.x, node.y, node.z];
                                const camPos = camera.getCamPos();
                                inter.dragPlaneNormal = vec3Normalize(vec3Sub(camPos, nodePos));
                                inter.dragPlanePoint = nodePos;
                                const pt = rayPlaneIntersectGeneral(ray, nodePos, inter.dragPlaneNormal);
                                if (pt) inter.dragOffset = [node.x - pt[0], node.y - pt[1], node.z - pt[2]];
                            }
                        }
                        e.preventDefault();
                    } else {
                        inter.orbiting = true;
                    }
                    }
                }
            } else if (e.button === 2) {
                inter.orbiting = true;
            }
        };

        const onMouseMove = (e: MouseEvent) => {
            const rect = container.getBoundingClientRect();
            inter.mouseX = e.clientX - rect.left;
            inter.mouseY = e.clientY - rect.top;
            const cw = container.clientWidth;
            const ch = container.clientHeight;

            if (inter.dragIdx >= 0) {
                const node = layout.nodeMap.get(inter.dragIdx);
                if (node) {
                    const { viewProj } = camera.getViewProj(cw, ch);
                    const invVP = mat4Inverse(viewProj);
                    if (invVP) {
                        const ray = screenToRay(inter.mouseX, inter.mouseY, cw, ch, invVP);
                        const pt = rayPlaneIntersectGeneral(ray, inter.dragPlanePoint, inter.dragPlaneNormal);
                        if (pt) {
                            node.x = pt[0] + inter.dragOffset[0];
                            node.y = pt[1] + inter.dragOffset[1];
                            node.z = pt[2] + inter.dragOffset[2];
                            // Sync target so lerp doesn't fight the drag
                            node.tx = node.x;
                            node.ty = node.y;
                            node.tz = node.z;
                        }
                    }
                }
                return;
            }

            if (inter.panning) {
                const dx = e.clientX - inter.lastMX;
                const dy = e.clientY - inter.lastMY;
                inter.lastMX = e.clientX;
                inter.lastMY = e.clientY;

                // Move camera target along right/up vectors in world space
                const camState = camera.stateRef.current;
                const speed = camState.dist * 0.002;
                const cosY = Math.cos(camState.yaw);
                const sinY = Math.sin(camState.yaw);
                // Right vector (in XZ plane)
                const rx = cosY,
                    rz = -sinY;
                // Up vector (projected): approximate as world Y
                camState.target = [
                    camState.target[0] - dx * speed * rx,
                    camState.target[1] + dy * speed,
                    camState.target[2] - dx * speed * rz,
                ];
                return;
            }

            if (inter.orbiting) {
                const dx = e.clientX - inter.lastMX;
                const dy = e.clientY - inter.lastMY;
                const camState = camera.stateRef.current;
                camState.yaw += dx * 0.005;
                camState.pitch = Math.max(-1.2, Math.min(1.2, camState.pitch + dy * 0.005));
                inter.lastMX = e.clientX;
                inter.lastMY = e.clientY;
                return;
            }

            // Hover detection
            const { viewProj } = camera.getViewProj(cw, ch);
            const invVP = mat4Inverse(viewProj);
            if (invVP) {
                const ray = screenToRay(inter.mouseX, inter.mouseY, cw, ch, invVP);
                let bestT = Infinity;
                let bestIdx = -1;
                for (const n of layout.nodes) {
                    const t = raySphere(ray.origin, ray.direction, [n.x, n.y, n.z], n.radius * 1.5);
                    if (t !== null && t < bestT) {
                        bestT = t;
                        bestIdx = n.index;
                    }
                }
                if (bestIdx !== inter.hoverIdx) {
                    inter.hoverIdx = bestIdx;
                    setHoverIdx(bestIdx);
                    if (bestIdx >= 0) {
                        const n = layout.nodeMap.get(bestIdx);
                        if (n) setTooltip({ x: inter.mouseX, y: inter.mouseY, node: n });
                    } else {
                        setTooltip(null);
                    }
                } else if (bestIdx >= 0) {
                    const n = layout.nodeMap.get(bestIdx);
                    if (n) setTooltip({ x: inter.mouseX, y: inter.mouseY, node: n });
                }
            }
        };

        const onMouseUp = (e: MouseEvent) => {
            if (inter.clickedNode >= 0 && e.button === 0) {
                // Layout-mode deferred selection: only select if the mouse is
                // still over the same node that was hit on mouseDown.
                const cw = container.clientWidth;
                const ch = container.clientHeight;
                const { viewProj } = camera.getViewProj(cw, ch);
                const invVP = mat4Inverse(viewProj);
                if (invVP) {
                    const ray = screenToRay(inter.mouseX, inter.mouseY, cw, ch, invVP);
                    let bestT = Infinity;
                    let bestIdx = -1;
                    for (const n of layout.nodes) {
                        const t = raySphere(ray.origin, ray.direction, [n.x, n.y, n.z], n.radius * 1.5);
                        if (t !== null && t < bestT) {
                            bestT = t;
                            bestIdx = n.index;
                        }
                    }
                    if (bestIdx === inter.clickedNode) {
                        inter.selectedIdx = inter.clickedNode;
                        setSelectedIdx(inter.clickedNode);
                    }
                }
            } else if (inter.dragIdx >= 0 && e.button === 0) {
                // Only select if the mouse barely moved (click, not a real drag).
                const dx = e.clientX - inter.downMX;
                const dy = e.clientY - inter.downMY;
                if (dx * dx + dy * dy > DRAG_THRESHOLD * DRAG_THRESHOLD) {
                    // Real drag — don't change selection.
                } else {
                    inter.selectedIdx = inter.dragIdx;
                    setSelectedIdx(inter.dragIdx);
                }
            } else if (e.button === 0 && inter.clickedNode < 0 && inter.dragIdx < 0) {
                // Clicked empty space — deselect if it was just a click, not a real orbit/pan
                const dx = e.clientX - inter.downMX;
                const dy = e.clientY - inter.downMY;
                const wasClick = (!inter.orbiting && !inter.panning)
                    || (dx * dx + dy * dy <= DRAG_THRESHOLD * DRAG_THRESHOLD);
                if (wasClick) {
                    inter.selectedIdx = -1;
                    setSelectedIdx(-1);
                    setTooltip(null);
                }
            }
            inter.dragIdx = -1;
            inter.orbiting = false;
            inter.panning = false;
            inter.clickedNode = -1;
        };

        const onWheel = (e: WheelEvent) => {
            if (isInteractionBlocked(e.target)) return;
            const camState = camera.stateRef.current;
            camState.dist = Math.max(2, Math.min(80, camState.dist + e.deltaY * 0.02));
            e.preventDefault();
        };

        container.addEventListener('mousedown', onMouseDown);
        window.addEventListener('mousemove', onMouseMove);
        window.addEventListener('mouseup', onMouseUp);
        container.addEventListener('wheel', onWheel, { passive: false });

        return () => {
            container.removeEventListener('mousedown', onMouseDown);
            window.removeEventListener('mousemove', onMouseMove);
            window.removeEventListener('mouseup', onMouseUp);
            container.removeEventListener('wheel', onWheel);
            // Reset interaction state to prevent stale orbiting/panning after effect re-runs
            inter.orbiting = false;
            inter.panning = false;
            inter.dragIdx = -1;
        };
    }, [layoutRef.current, camera]);

    return {
        selectedIdx,
        setSelectedIdx: setSelectedIdxWithSync,
        hoverIdx,
        tooltip,
        interRef,
    };
}
