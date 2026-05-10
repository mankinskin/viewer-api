/**
 * Touch interaction hook for 3D hypergraph view.
 * Handles single-finger orbit, two-finger pinch/pan, tap, double-tap, and long-press.
 *
 * Shares selection state with useMouseInteraction via the interRef.
 */
import { useRef, useEffect } from 'preact/hooks';
import type { MutableRef } from 'preact/hooks';
import { mat4Inverse, screenToRay } from '../../Scene3D/math3d';
import { raySphere } from '../utils/math';
import type { GraphLayout } from '../layout';
import type { CameraController } from './useCamera';
import type { InteractionState } from './useMouseInteraction';

const TOUCH_BLOCK_SELECTOR = [
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

function touchTargetElement(target: EventTarget | null): HTMLElement | null {
    if (!target) return null;
    if (target instanceof HTMLElement) return target;
    if (target instanceof Node) return target.parentElement;
    return null;
}

function isTouchBlocked(target: EventTarget | null): boolean {
    const el = touchTargetElement(target);
    return !!el?.closest(TOUCH_BLOCK_SELECTOR);
}

// ── Constants ──

/** Max movement (px) for a touch to count as a tap */
const TAP_MOVE_THRESHOLD = 10;
/** Max duration (ms) for a touch to count as a tap */
const TAP_DURATION = 200;
/** Max gap (ms) between taps for a double-tap */
const DOUBLE_TAP_GAP = 300;
/** Min hold duration (ms) for a long-press */
const LONG_PRESS_DURATION = 500;

// ── Hook ──

/**
 * Adds touch gesture support to the hypergraph container.
 * Reads/writes `interRef.current.selectedIdx` directly and calls `setSelectedIdx`
 * to trigger React re-renders, keeping state in sync with the mouse hook.
 */
export function useTouchInteraction(
    containerRef: { current: HTMLDivElement | null },
    layoutRef: { current: GraphLayout | null },
    camera: CameraController,
    interRef: MutableRef<InteractionState>,
    setSelectedIdx: (idx: number) => void,
): void {

    // Refs for gesture state
    const touchState = useRef({
        // Single-finger tracking
        startX: 0,
        startY: 0,
        lastX: 0,
        lastY: 0,
        startTime: 0,
        fingers: 0,

        // Two-finger tracking
        lastDist: 0,
        lastMidX: 0,
        lastMidY: 0,

        // Gesture detection
        longPressTimer: null as ReturnType<typeof setTimeout> | null,
        lastTapTime: 0,
        lastTapX: 0,
        lastTapY: 0,
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
                if (t !== null && t < bestT) {
                    bestT = t;
                    bestIdx = n.index;
                }
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

        // ── Touch Start ──
        const onTouchStart = (e: TouchEvent) => {
            if (isTouchBlocked(e.target)) return;

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

                // Start long-press timer
                clearLongPress();
                state.longPressTimer = setTimeout(() => {
                    // Long press: select the node under finger (shows info panel)
                    const idx = hitTest(state.startX, state.startY);
                    if (idx >= 0) {
                        interRef.current.selectedIdx = idx;
                        setSelectedIdx(idx);
                    }
                    state.longPressTimer = null;
                }, LONG_PRESS_DURATION);
            } else if (touches.length === 2) {
                clearLongPress();
                const t1 = touches[0];
                const t2 = touches[1];
                if (!t1 || !t2) return;
                state.lastDist = touchDist(t1, t2);
                const [mx, my] = touchMid(t1, t2);
                state.lastMidX = mx;
                state.lastMidY = my;
            }
        };

        // ── Touch Move ──
        const onTouchMove = (e: TouchEvent) => {
            if (isTouchBlocked(e.target)) return;

            e.preventDefault();
            const touches = e.touches;

            if (touches.length === 1 && state.fingers === 1) {
                // Single-finger: orbit
                const t = touches[0];
                if (!t) return;
                const dx = t.clientX - state.lastX;
                const dy = t.clientY - state.lastY;
                state.lastX = t.clientX;
                state.lastY = t.clientY;

                // Cancel long-press if moved enough
                const totalDx = t.clientX - (state.startX + container.getBoundingClientRect().left);
                const totalDy = t.clientY - (state.startY + container.getBoundingClientRect().top);
                if (Math.abs(totalDx) > TAP_MOVE_THRESHOLD || Math.abs(totalDy) > TAP_MOVE_THRESHOLD) {
                    clearLongPress();
                }

                // Orbit
                const camState = camera.stateRef.current;
                camState.yaw += dx * 0.005;
                camState.pitch = Math.max(-1.2, Math.min(1.2, camState.pitch + dy * 0.005));
                state.isOrbiting = true;
            } else if (touches.length === 2) {
                clearLongPress();
                const t1 = touches[0];
                const t2 = touches[1];
                if (!t1 || !t2) return;

                const dist = touchDist(t1, t2);
                const [mx, my] = touchMid(t1, t2);

                // Pinch → zoom
                if (state.lastDist > 0) {
                    const scale = state.lastDist / dist;
                    const camState = camera.stateRef.current;
                    camState.dist = Math.max(2, Math.min(80, camState.dist * scale));
                }
                state.lastDist = dist;

                // Two-finger pan
                const panDx = mx - state.lastMidX;
                const panDy = my - state.lastMidY;
                const camState = camera.stateRef.current;
                const speed = camState.dist * 0.002;
                const cosY = Math.cos(camState.yaw);
                const sinY = Math.sin(camState.yaw);
                const rx = cosY, rz = -sinY;
                camState.target = [
                    camState.target[0] - panDx * speed * rx,
                    camState.target[1] + panDy * speed,
                    camState.target[2] - panDx * speed * rz,
                ];

                state.lastMidX = mx;
                state.lastMidY = my;
            }
        };

        // ── Touch End ──
        const onTouchEnd = (e: TouchEvent) => {
            clearLongPress();

            if (e.touches.length === 0 && state.fingers === 1) {
                const dt = Date.now() - state.startTime;
                const rect = container.getBoundingClientRect();
                const changedTouch = e.changedTouches[0];
                if (!changedTouch) {
                    state.fingers = e.touches.length;
                    return;
                }
                const endX = changedTouch.clientX - rect.left;
                const endY = changedTouch.clientY - rect.top;
                const dx = endX - state.startX;
                const dy = endY - state.startY;
                const moved = Math.sqrt(dx * dx + dy * dy);

                if (dt < TAP_DURATION && moved < TAP_MOVE_THRESHOLD) {
                    // It's a tap!
                    const now = Date.now();
                    const tapDx = endX - state.lastTapX;
                    const tapDy = endY - state.lastTapY;
                    const tapDist = Math.sqrt(tapDx * tapDx + tapDy * tapDy);

                    if (now - state.lastTapTime < DOUBLE_TAP_GAP && tapDist < TAP_MOVE_THRESHOLD * 2) {
                        // Double-tap → focus on node
                        const idx = hitTest(endX, endY);
                        if (idx >= 0) {
                            const n = layout.nodeMap.get(idx);
                            if (n) {
                                camera.focusOn([n.x, n.y, n.z]);
                            }
                        }
                        state.lastTapTime = 0; // Reset to prevent triple-tap
                    } else {
                        // Single tap → select node
                        const idx = hitTest(endX, endY);
                        interRef.current.selectedIdx = idx >= 0 ? idx : -1;
                        setSelectedIdx(idx >= 0 ? idx : -1);
                        state.lastTapTime = now;
                        state.lastTapX = endX;
                        state.lastTapY = endY;
                    }
                }
            }

            state.fingers = e.touches.length;
        };

        const onTouchCancel = () => {
            clearLongPress();
            state.fingers = 0;
            state.isOrbiting = false;
        };

        container.addEventListener('touchstart', onTouchStart, { passive: false });
        container.addEventListener('touchmove', onTouchMove, { passive: false });
        container.addEventListener('touchend', onTouchEnd);
        container.addEventListener('touchcancel', onTouchCancel);

        return () => {
            clearLongPress();
            container.removeEventListener('touchstart', onTouchStart);
            container.removeEventListener('touchmove', onTouchMove);
            container.removeEventListener('touchend', onTouchEnd);
            container.removeEventListener('touchcancel', onTouchCancel);
        };
    }, [layoutRef.current, camera, interRef, setSelectedIdx]);
}
