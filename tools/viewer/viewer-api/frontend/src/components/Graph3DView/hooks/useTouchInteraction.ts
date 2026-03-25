/**
 * Touch interaction hook for Graph3DView.
 * Single-finger orbit, two-finger pinch/pan, tap, double-tap, long-press.
 *
 * Ported from log-viewer HypergraphView; simplified (no UI panel selectors).
 */
import { useRef, useEffect } from 'preact/hooks';
import type { MutableRef } from 'preact/hooks';
import { mat4Inverse, screenToRay } from '../../../utils/math3d';
import { raySphere } from '../utils/math';
import type { InternalLayout } from '../types';
import type { CameraController } from './useCamera';
import type { InteractionState } from './useMouseInteraction';

// ── Constants ──

const TAP_MOVE_THRESHOLD = 10;
const TAP_DURATION = 200;
const DOUBLE_TAP_GAP = 300;
const LONG_PRESS_DURATION = 500;

// ── Hook ──

/**
 * Adds touch gesture support to the Graph3DView container.
 * Reads/writes `interRef.current.selectedIdx` directly and calls `setSelectedIdx`
 * to trigger Preact re-renders.
 */
export function useTouchInteraction(
    containerRef: { current: HTMLDivElement | null },
    layoutRef: { current: InternalLayout | null },
    camera: CameraController,
    interRef: MutableRef<InteractionState>,
    setSelectedIdx: (idx: number) => void,
): void {
    const touchState = useRef({
        startX: 0, startY: 0,
        lastX: 0, lastY: 0,
        startTime: 0,
        fingers: 0,
        lastDist: 0,
        lastMidX: 0, lastMidY: 0,
        longPressTimer: null as ReturnType<typeof setTimeout> | null,
        lastTapTime: 0,
        lastTapX: 0, lastTapY: 0,
        isOrbiting: false,
    });

    useEffect(() => {
        const container = containerRef.current;
        const layout = layoutRef.current;
        if (!container || !layout) return;

        const state = touchState.current;

        const hitTest = (x: number, y: number): number => {
            const cw = container.clientWidth;
            const ch = container.clientHeight;
            const { viewProj } = camera.getViewProj(cw, ch);
            const invVP = mat4Inverse(viewProj);
            if (!invVP) return -1;
            const ray = screenToRay(x, y, cw, ch, invVP);
            let bestT = Infinity;
            let bestIdx = -1;
            for (const n of layout.nodes) {
                const t = raySphere(ray.origin, ray.direction, [n.x, n.y, n.z], n.radius * 1.5);
                if (t !== null && t < bestT) { bestT = t; bestIdx = n.index; }
            }
            return bestIdx;
        };

        const clearLongPress = () => {
            if (state.longPressTimer) {
                clearTimeout(state.longPressTimer);
                state.longPressTimer = null;
            }
        };

        const touchDist = (t1: Touch, t2: Touch): number => {
            const dx = t1.clientX - t2.clientX;
            const dy = t1.clientY - t2.clientY;
            return Math.sqrt(dx * dx + dy * dy);
        };

        const touchMid = (t1: Touch, t2: Touch): [number, number] => [
            (t1.clientX + t2.clientX) / 2,
            (t1.clientY + t2.clientY) / 2,
        ];

        const onTouchStart = (e: TouchEvent) => {
            e.preventDefault();
            const touches = e.touches;
            state.fingers = touches.length;

            if (touches.length === 1) {
                const t = touches[0];
                if (!t) return;
                const rect = container.getBoundingClientRect();
                state.startX = t.clientX - rect.left;
                state.startY = t.clientY - rect.top;
                state.lastX = t.clientX;
                state.lastY = t.clientY;
                state.startTime = Date.now();
                state.isOrbiting = false;

                clearLongPress();
                state.longPressTimer = setTimeout(() => {
                    const idx = hitTest(state.startX, state.startY);
                    if (idx >= 0) {
                        interRef.current.selectedIdx = idx;
                        setSelectedIdx(idx);
                    }
                    state.longPressTimer = null;
                }, LONG_PRESS_DURATION);
            } else if (touches.length === 2) {
                clearLongPress();
                const t1 = touches[0], t2 = touches[1];
                if (!t1 || !t2) return;
                state.lastDist = touchDist(t1, t2);
                const [mx, my] = touchMid(t1, t2);
                state.lastMidX = mx;
                state.lastMidY = my;
            }
        };

        const onTouchMove = (e: TouchEvent) => {
            e.preventDefault();
            const touches = e.touches;

            if (touches.length === 1 && state.fingers === 1) {
                const t = touches[0];
                if (!t) return;
                const dx = t.clientX - state.lastX;
                const dy = t.clientY - state.lastY;
                state.lastX = t.clientX;
                state.lastY = t.clientY;

                const rect = container.getBoundingClientRect();
                const totalDx = t.clientX - (state.startX + rect.left);
                const totalDy = t.clientY - (state.startY + rect.top);
                if (Math.abs(totalDx) > TAP_MOVE_THRESHOLD || Math.abs(totalDy) > TAP_MOVE_THRESHOLD) {
                    clearLongPress();
                }

                const camState = camera.stateRef.current;
                camState.yaw += dx * 0.005;
                camState.pitch = Math.max(-1.2, Math.min(1.2, camState.pitch + dy * 0.005));
                state.isOrbiting = true;
            } else if (touches.length === 2) {
                clearLongPress();
                const t1 = touches[0], t2 = touches[1];
                if (!t1 || !t2) return;

                const dist = touchDist(t1, t2);
                const [mx, my] = touchMid(t1, t2);

                // Pinch → zoom
                if (state.lastDist > 0) {
                    const scale = state.lastDist / dist;
                    const camState = camera.stateRef.current;
                    camState.dist = Math.max(2, Math.min(80, camState.dist * scale));
                }

                // Two-finger pan
                if (state.lastMidX !== 0) {
                    const pdx = mx - state.lastMidX;
                    const pdy = my - state.lastMidY;
                    const camState = camera.stateRef.current;
                    const speed = camState.dist * 0.003;
                    const cosY = Math.cos(camState.yaw);
                    const sinY = Math.sin(camState.yaw);
                    camState.target = [
                        camState.target[0] - pdx * speed * cosY,
                        camState.target[1] + pdy * speed,
                        camState.target[2] + pdx * speed * sinY,
                    ];
                }

                state.lastDist = dist;
                state.lastMidX = mx;
                state.lastMidY = my;
            }
        };

        const onTouchEnd = (e: TouchEvent) => {
            clearLongPress();
            if (state.fingers === 1 && e.changedTouches.length === 1) {
                const t = e.changedTouches[0];
                if (!t) return;
                const duration = Date.now() - state.startTime;
                const rect = container.getBoundingClientRect();
                const endX = t.clientX - rect.left;
                const endY = t.clientY - rect.top;
                const dx = endX - state.startX;
                const dy = endY - state.startY;
                const moved = dx * dx + dy * dy > TAP_MOVE_THRESHOLD * TAP_MOVE_THRESHOLD;

                if (!moved && duration < TAP_DURATION) {
                    // Check double-tap
                    const now = Date.now();
                    const tdx = t.clientX - state.lastTapX;
                    const tdy = t.clientY - state.lastTapY;
                    const isDoubleTap = now - state.lastTapTime < DOUBLE_TAP_GAP
                        && tdx * tdx + tdy * tdy < TAP_MOVE_THRESHOLD * TAP_MOVE_THRESHOLD;

                    if (isDoubleTap) {
                        // Double-tap: reset camera
                        camera.cancelFocus();
                        state.lastTapTime = 0;
                    } else {
                        // Single tap: select node
                        const idx = hitTest(endX, endY);
                        interRef.current.selectedIdx = idx;
                        setSelectedIdx(idx);
                        state.lastTapTime = now;
                        state.lastTapX = t.clientX;
                        state.lastTapY = t.clientY;
                    }
                }
            }
            state.fingers = e.touches.length;
            if (state.fingers < 2) {
                state.lastDist = 0;
                state.lastMidX = 0;
                state.lastMidY = 0;
            }
        };

        container.addEventListener('touchstart', onTouchStart, { passive: false });
        container.addEventListener('touchmove', onTouchMove, { passive: false });
        container.addEventListener('touchend', onTouchEnd);

        return () => {
            container.removeEventListener('touchstart', onTouchStart);
            container.removeEventListener('touchmove', onTouchMove);
            container.removeEventListener('touchend', onTouchEnd);
            clearLongPress();
        };
    }, [layoutRef.current, camera]);
}
