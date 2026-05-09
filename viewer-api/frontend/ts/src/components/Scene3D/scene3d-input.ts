import type { Mat4, Ray } from '../../utils/math3d';
import {
    mat4Inverse,
    mat4TransformDir,
    mat4TransformPoint,
    rayAABBIntersect,
    rayPlaneIntersect,
    screenToRay,
} from '../../utils/math3d';

import type { SceneObject, SceneState } from './scene3d-data';

const TOUCH_UI_SELECTOR = '.scene3d-hud button, .scene3d-hud a, .scene3d-hud input';

interface Scene3dInputOptions {
    container: HTMLDivElement;
    state: SceneState;
    getViewProj: (aspectRatio: number) => Mat4;
    getModelMatrix: (obj: SceneObject, time: number, idx: number) => Mat4;
}

export function registerScene3dInputHandlers({
    container,
    state,
    getViewProj,
    getModelMatrix,
}: Scene3dInputOptions): () => void {
    const onMouseDown = (event: MouseEvent) => {
        const rect = container.getBoundingClientRect();
        state.mouseX = event.clientX - rect.left;
        state.mouseY = event.clientY - rect.top;
        state.lastMX = event.clientX;
        state.lastMY = event.clientY;

        const canvasWidth = container.clientWidth;
        const canvasHeight = container.clientHeight;

        if (event.button === 0) {
            const time = performance.now() / 1000 - state.startTime;
            const viewProj = getViewProj(canvasWidth / canvasHeight);
            const invViewProj = mat4Inverse(viewProj);
            if (invViewProj) {
                const ray = screenToRay(
                    state.mouseX,
                    state.mouseY,
                    canvasWidth,
                    canvasHeight,
                    invViewProj,
                );
                let bestT = Infinity;
                let bestIdx = -1;
                for (let i = 0; i < state.objects.length; i++) {
                    const obj = state.objects[i];
                    if (!obj) continue;
                    const model = getModelMatrix(obj, time, i);
                    const invModel = mat4Inverse(model);
                    if (!invModel) continue;
                    const localRay: Ray = {
                        origin: mat4TransformPoint(invModel, ray.origin),
                        direction: mat4TransformDir(invModel, ray.direction),
                    };
                    const t = rayAABBIntersect(
                        localRay,
                        [-0.5, -0.5, -0.5],
                        [0.5, 0.5, 0.5],
                    );
                    if (t !== null && t < bestT) {
                        bestT = t;
                        bestIdx = i;
                    }
                }
                const bestObj = bestIdx >= 0 ? state.objects[bestIdx] : undefined;
                if (bestObj) {
                    state.dragIdx = bestIdx;
                    state.dragPlaneY = bestObj.position[1];
                    const point = rayPlaneIntersect(ray, state.dragPlaneY);
                    if (point) {
                        state.dragOffset = [
                            bestObj.position[0] - point[0],
                            0,
                            bestObj.position[2] - point[2],
                        ];
                    }
                    event.preventDefault();
                } else {
                    state.orbiting = true;
                }
            }
        } else if (event.button === 2) {
            state.orbiting = true;
            event.preventDefault();
        }
    };

    const onMouseMove = (event: MouseEvent) => {
        const rect = container.getBoundingClientRect();
        state.mouseX = event.clientX - rect.left;
        state.mouseY = event.clientY - rect.top;

        if (state.dragIdx >= 0) {
            const dragObj = state.objects[state.dragIdx];
            if (!dragObj) {
                state.dragIdx = -1;
                return;
            }
            const canvasWidth = container.clientWidth;
            const canvasHeight = container.clientHeight;
            const viewProj = getViewProj(canvasWidth / canvasHeight);
            const invViewProj = mat4Inverse(viewProj);
            if (invViewProj) {
                const ray = screenToRay(
                    state.mouseX,
                    state.mouseY,
                    canvasWidth,
                    canvasHeight,
                    invViewProj,
                );
                const point = rayPlaneIntersect(ray, state.dragPlaneY);
                if (point) {
                    dragObj.position[0] = point[0] + state.dragOffset[0];
                    dragObj.position[2] = point[2] + state.dragOffset[2];
                }
            }
            return;
        }

        if (state.orbiting) {
            const dx = event.clientX - state.lastMX;
            const dy = event.clientY - state.lastMY;
            state.camYaw += dx * 0.005;
            state.camPitch = Math.max(
                -1.4,
                Math.min(1.4, state.camPitch + dy * 0.005),
            );
            state.lastMX = event.clientX;
            state.lastMY = event.clientY;
        }
    };

    const onMouseUp = () => {
        state.dragIdx = -1;
        state.orbiting = false;
    };

    const onWheel = (event: WheelEvent) => {
        state.camDist = Math.max(
            3,
            Math.min(30, state.camDist + event.deltaY * 0.01),
        );
        event.preventDefault();
    };

    const onContextMenu = (event: Event) => event.preventDefault();

    let touchFingers = 0;
    let touchLastX = 0;
    let touchLastY = 0;
    let touchLastDist = 0;
    let touchLastMidX = 0;
    let touchLastMidY = 0;

    const touchDist2 = (left: Touch, right: Touch): number => {
        const dx = left.clientX - right.clientX;
        const dy = left.clientY - right.clientY;
        return Math.sqrt(dx * dx + dy * dy);
    };

    const onTouchStart = (event: TouchEvent) => {
        const target = event.target as HTMLElement;
        if (target.closest(TOUCH_UI_SELECTOR)) return;

        event.preventDefault();
        touchFingers = event.touches.length;
        if (event.touches.length === 1) {
            const touch = event.touches[0];
            if (!touch) return;
            touchLastX = touch.clientX;
            touchLastY = touch.clientY;
        } else if (event.touches.length === 2) {
            const left = event.touches[0];
            const right = event.touches[1];
            if (!left || !right) return;
            touchLastDist = touchDist2(left, right);
            touchLastMidX = (left.clientX + right.clientX) / 2;
            touchLastMidY = (left.clientY + right.clientY) / 2;
        }
    };

    const onTouchMove = (event: TouchEvent) => {
        const target = event.target as HTMLElement;
        if (target.closest(TOUCH_UI_SELECTOR)) return;

        event.preventDefault();
        if (event.touches.length === 1 && touchFingers === 1) {
            const touch = event.touches[0];
            if (!touch) return;
            const dx = touch.clientX - touchLastX;
            const dy = touch.clientY - touchLastY;
            touchLastX = touch.clientX;
            touchLastY = touch.clientY;
            state.camYaw += dx * 0.005;
            state.camPitch = Math.max(
                -1.4,
                Math.min(1.4, state.camPitch + dy * 0.005),
            );
        } else if (event.touches.length === 2) {
            const left = event.touches[0];
            const right = event.touches[1];
            if (!left || !right) return;
            const dist = touchDist2(left, right);
            const midX = (left.clientX + right.clientX) / 2;
            const midY = (left.clientY + right.clientY) / 2;

            if (touchLastDist > 0) {
                const scale = touchLastDist / dist;
                state.camDist = Math.max(3, Math.min(30, state.camDist * scale));
            }
            touchLastDist = dist;

            const panDx = midX - touchLastMidX;
            const panDy = midY - touchLastMidY;
            const speed = state.camDist * 0.002;
            const cosYaw = Math.cos(state.camYaw);
            const sinYaw = Math.sin(state.camYaw);
            state.camTarget = [
                state.camTarget[0] - panDx * speed * cosYaw,
                state.camTarget[1] + panDy * speed,
                state.camTarget[2] + panDx * speed * sinYaw,
            ];
            touchLastMidX = midX;
            touchLastMidY = midY;
        }
    };

    const onTouchEnd = (event: TouchEvent) => {
        touchFingers = event.touches.length;
    };

    const onTouchCancel = () => {
        touchFingers = 0;
        touchLastDist = 0;
    };

    container.addEventListener('mousedown', onMouseDown);
    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
    container.addEventListener('wheel', onWheel, { passive: false });
    container.addEventListener('contextmenu', onContextMenu);
    container.addEventListener('touchstart', onTouchStart, { passive: false });
    container.addEventListener('touchmove', onTouchMove, { passive: false });
    container.addEventListener('touchend', onTouchEnd);
    container.addEventListener('touchcancel', onTouchCancel);

    return () => {
        container.removeEventListener('mousedown', onMouseDown);
        window.removeEventListener('mousemove', onMouseMove);
        window.removeEventListener('mouseup', onMouseUp);
        container.removeEventListener('wheel', onWheel);
        container.removeEventListener('contextmenu', onContextMenu);
        container.removeEventListener('touchstart', onTouchStart);
        container.removeEventListener('touchmove', onTouchMove);
        container.removeEventListener('touchend', onTouchEnd);
        container.removeEventListener('touchcancel', onTouchCancel);
    };
}