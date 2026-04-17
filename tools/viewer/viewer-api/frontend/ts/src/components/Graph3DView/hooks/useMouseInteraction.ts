/**
 * Mouse interaction hook for Graph3DView.
 * Handles node dragging, camera orbiting/panning, hover detection, and selection.
 *
 * Ported from log-viewer HypergraphView; simplified (no auto-layout mode,
 * no decomposition expanded-node logic).
 */
import { useRef, useEffect, useState, useCallback } from 'preact/hooks';
import type { Vec3 } from '../../../utils/math3d';
import { mat4Inverse, screenToRay, rayPlaneIntersectGeneral, vec3Sub, vec3Normalize } from '../../../utils/math3d';
import { raySphere } from '../utils/math';
import type { InternalLayout, InternalNode } from '../types';
import type { CameraController } from './useCamera';

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
    clickedNode: number;
    downMX: number;
    downMY: number;
}

export interface TooltipData {
    x: number;
    y: number;
    node: InternalNode;
}

export interface MouseInteractionResult {
    selectedIdx: number;
    setSelectedIdx: (idx: number) => void;
    hoverIdx: number;
    tooltip: TooltipData | null;
    interRef: { current: InteractionState };
}

/**
 * Hook for handling mouse interactions in the Graph3DView.
 */
export function useMouseInteraction(
    containerRef: { current: HTMLDivElement | null },
    layoutRef: { current: InternalLayout | null },
    camera: CameraController,
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

    const setSelectedIdxWithSync = useCallback((idx: number) => {
        interRef.current.selectedIdx = idx;
        setSelectedIdx(idx);
    }, []);

    useEffect(() => {
        const container = containerRef.current;
        const layout = layoutRef.current;
        if (!container || !layout) return;

        const inter = interRef.current;
        const DRAG_THRESHOLD = 5;

        const onMouseDown = (e: MouseEvent) => {
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
                inter.panning = true;
                camera.cancelFocus();
                e.preventDefault();
            } else if (e.button === 0) {
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
                    if (bestIdx >= 0) {
                        const node = layout.nodeMap.get(bestIdx);
                        if (node) {
                            // Allow dragging — accumulate a pending click-node for selection on mouseUp
                            inter.clickedNode = bestIdx;
                            inter.dragIdx = bestIdx;
                            const nodePos: Vec3 = [node.x, node.y, node.z];
                            const camPos = camera.getCamPos();
                            inter.dragPlaneNormal = vec3Normalize(vec3Sub(camPos, nodePos));
                            inter.dragPlanePoint = nodePos;
                            const pt = rayPlaneIntersectGeneral(ray, nodePos, inter.dragPlaneNormal);
                            if (pt) inter.dragOffset = [node.x - pt[0], node.y - pt[1], node.z - pt[2]];
                        }
                        e.preventDefault();
                    } else {
                        inter.orbiting = true;
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
                const camState = camera.stateRef.current;
                const speed = camState.dist * 0.002;
                const cosY = Math.cos(camState.yaw);
                const sinY = Math.sin(camState.yaw);
                const rx = cosY, rz = -sinY;
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
            if (e.button === 0) {
                const dx = e.clientX - inter.downMX;
                const dy = e.clientY - inter.downMY;
                const wasClick = dx * dx + dy * dy <= DRAG_THRESHOLD * DRAG_THRESHOLD;

                if (inter.clickedNode >= 0 && wasClick) {
                    inter.selectedIdx = inter.clickedNode;
                    setSelectedIdx(inter.clickedNode);
                } else if (inter.clickedNode < 0 && inter.dragIdx < 0 && !inter.orbiting && !inter.panning && wasClick) {
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
