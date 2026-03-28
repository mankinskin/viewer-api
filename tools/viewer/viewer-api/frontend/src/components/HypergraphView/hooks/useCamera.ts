/**
 * Camera state management hook for 3D hypergraph view.
 * Handles orbit camera positioning and smooth focus animation.
 */
import { useRef, useCallback, useMemo } from 'preact/hooks';
import type { Vec3 } from '../../Scene3D/math3d';
import { mat4Perspective, mat4LookAt, mat4Multiply } from '../../Scene3D/math3d';

export interface CameraState {
    yaw: number;
    pitch: number;
    dist: number;
    target: Vec3;
    focusTarget: Vec3 | null;
    focusSpeed: number;
}

export interface CameraController {
    /** Current camera state ref (mutable) */
    stateRef: { current: CameraState };
    /** Get camera world position */
    getCamPos: () => Vec3;
    /** Get view-projection matrix, optionally with focus lerp update */
    getViewProj: (cw: number, ch: number, dt?: number) => { viewProj: Float32Array; camPos: Vec3 };
    /** Trigger smooth focus animation to target position */
    focusOn: (target: Vec3) => void;
    /** Cancel any active focus animation */
    cancelFocus: () => void;
    /** Reset camera to view a new layout */
    resetForLayout: (nodeCount: number, center: [number, number, number]) => void;
    /** Get the camera's screen-space right and up axes in world coordinates */
    getAxes: () => { right: [number, number, number]; up: [number, number, number] };
}

/**
 * Hook for managing 3D orbit camera with smooth focus animation.
 */
export function useCamera(): CameraController {
    const stateRef = useRef<CameraState>({
        yaw: 0.5,
        pitch: 0.4,
        dist: 6,
        target: [0, 0, 0] as Vec3,
        focusTarget: null,
        focusSpeed: 4.0, // lerp speed (units/sec)
    });

    const getCamPos = useCallback((): Vec3 => {
        const c = stateRef.current;
        return [
            c.target[0] + c.dist * Math.cos(c.pitch) * Math.sin(c.yaw),
            c.target[1] + c.dist * Math.sin(c.pitch),
            c.target[2] + c.dist * Math.cos(c.pitch) * Math.cos(c.yaw),
        ];
    }, []);

    const getViewProj = useCallback(
        (cw: number, ch: number, dt?: number) => {
            const c = stateRef.current;

            // Smooth-lerp camera target toward focusTarget if set
            if (c.focusTarget && dt && dt > 0) {
                const alpha = 1 - Math.exp(-c.focusSpeed * dt);
                c.target = [
                    c.target[0] + (c.focusTarget[0] - c.target[0]) * alpha,
                    c.target[1] + (c.focusTarget[1] - c.target[1]) * alpha,
                    c.target[2] + (c.focusTarget[2] - c.target[2]) * alpha,
                ];
                // Stop animating once close enough
                const dx = c.focusTarget[0] - c.target[0];
                const dy = c.focusTarget[1] - c.target[1];
                const dz = c.focusTarget[2] - c.target[2];
                if (dx * dx + dy * dy + dz * dz < 0.0001) {
                    c.target = [...c.focusTarget] as Vec3;
                    c.focusTarget = null;
                }
            }

            const camPos = getCamPos();
            const view = mat4LookAt(camPos, c.target, [0, 1, 0]);
            const proj = mat4Perspective(Math.PI / 4, cw / Math.max(ch, 1), 0.1, 200);
            return { viewProj: mat4Multiply(proj, view), camPos };
        },
        [getCamPos]
    );

    const focusOn = useCallback((target: Vec3) => {
        const c = stateRef.current;
        // Skip if already targeting (or at) essentially the same position.
        // This prevents unnecessary animation restarts when the same node
        // is re-focused across consecutive search steps.
        const ref = c.focusTarget ?? c.target;
        const dx = target[0] - ref[0];
        const dy = target[1] - ref[1];
        const dz = target[2] - ref[2];
        if (dx * dx + dy * dy + dz * dz < 0.001) return;
        c.focusTarget = target;
    }, []);

    const cancelFocus = useCallback(() => {
        stateRef.current.focusTarget = null;
    }, []);

    const resetForLayout = useCallback((nodeCount: number, center: [number, number, number]) => {
        const c = stateRef.current;
        c.dist = Math.max(6, nodeCount * 0.5);
        c.target = [center[0], center[1], center[2]];
        c.focusTarget = null;
    }, []);

    /** Compute camera right and up axes from current yaw/pitch. */
    const getAxes = useCallback((): { right: [number, number, number]; up: [number, number, number] } => {
        const c = stateRef.current;
        const cosY = Math.cos(c.yaw);
        const sinY = Math.sin(c.yaw);
        const cosP = Math.cos(c.pitch);
        const sinP = Math.sin(c.pitch);

        // Forward = target - camPos (normalized), but we derive from spherical coords:
        // camPos offset = (cosP*sinY, sinP, cosP*cosY) * dist
        // forward = -normalized(offset) = (-cosP*sinY, -sinP, -cosP*cosY)
        // right = cross(forward, worldUp=[0,1,0]) normalized
        // up = cross(right, forward) normalized
        const fx = -cosP * sinY;
        const fy = -sinP;
        const fz = -cosP * cosY;

        // right = cross(forward, worldUp=(0,1,0)) = (-fz, 0, fx)
        let rx = -fz;
        let ry = 0;
        let rz = fx;
        const rLen = Math.sqrt(rx * rx + ry * ry + rz * rz);
        if (rLen > 1e-6) { rx /= rLen; ry /= rLen; rz /= rLen; }

        // up = cross(right, forward)
        let ux = ry * fz - rz * fy;
        let uy = rz * fx - rx * fz;
        let uz = rx * fy - ry * fx;
        const uLen = Math.sqrt(ux * ux + uy * uy + uz * uz);
        if (uLen > 1e-6) { ux /= uLen; uy /= uLen; uz /= uLen; }

        return { right: [rx, ry, rz], up: [ux, uy, uz] };
    }, []);

    return useMemo<CameraController>(
        () => ({
            stateRef,
            getCamPos,
            getViewProj,
            focusOn,
            cancelFocus,
            resetForLayout,
            getAxes,
        }),
        [stateRef, getCamPos, getViewProj, focusOn, cancelFocus, resetForLayout, getAxes]
    );
}
