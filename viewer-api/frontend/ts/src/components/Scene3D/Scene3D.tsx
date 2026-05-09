/// <reference types="@webgpu/types" />
/**
 * Scene3D — 3D cube scene rendered via the shared WgpuOverlay canvas.
 *
 * Uses the overlay render callback system to draw into a viewport region
 * (like HypergraphView).  The shared depth buffer enables proper depth
 * testing for opaque 3D objects.
 */
import { useRef, useState, useCallback } from 'preact/hooks';
import './scene3d.css';
import {
    Vec3, Mat4,
    mat4Perspective, mat4LookAt, mat4Multiply,
    mat4Translate, mat4ScaleMat, mat4RotateY,
} from '../../utils/math3d';
import { overlayGpu } from '../WgpuOverlay/WgpuOverlay';

import {
    createInitialSceneState,
    type SceneObject,
} from './scene3d-data';
import { useScene3dOverlay } from './scene3d-overlay';

// ── component ──

export function Scene3D() {
    const containerRef = useRef<HTMLDivElement>(null);
    const resetRef = useRef<(() => void) | null>(null);
    const [noWebGpu, setNoWebGpu] = useState(false);

    const gpu = overlayGpu.value;
    const stateRef = useRef(createInitialSceneState());

    const getCamPos = useCallback((): Vec3 => {
        const s = stateRef.current;
        return [
            s.camTarget[0] + s.camDist * Math.cos(s.camPitch) * Math.sin(s.camYaw),
            s.camTarget[1] + s.camDist * Math.sin(s.camPitch),
            s.camTarget[2] + s.camDist * Math.cos(s.camPitch) * Math.cos(s.camYaw),
        ];
    }, []);

    const getViewProj = useCallback((aspectRatio: number): Mat4 => {
        const camPos = getCamPos();
        const s = stateRef.current;
        const view = mat4LookAt(camPos, s.camTarget, [0, 1, 0]);
        const proj = mat4Perspective(Math.PI / 4, aspectRatio, 0.1, 100);
        return mat4Multiply(proj, view);
    }, [getCamPos]);

    const getModelMatrix = useCallback((obj: SceneObject, time: number, idx: number): Mat4 => {
        const s = stateRef.current;
        const isDragged = idx === s.dragIdx;
        const bob = isDragged ? 0 : Math.sin(time * 1.2 + obj.position[0] * 3 + obj.position[2] * 2) * 0.04;
        const lift = isDragged ? 0.2 : 0;
        const t = mat4Translate([obj.position[0], obj.position[1] + bob + lift, obj.position[2]]);
        const r = mat4RotateY(obj.rotationY);
        const sc = mat4ScaleMat(obj.scale);
        return mat4Multiply(mat4Multiply(t, r), sc);
    }, []);

    useScene3dOverlay({
        containerRef,
        resetRef,
        stateRef,
        gpu,
        setNoWebGpu,
        getCamPos,
        getViewProj,
        getModelMatrix,
    });

    if (noWebGpu) {
        return (
            <div ref={containerRef} class="scene3d-container">
                <div class="scene3d-error">WebGPU is not supported in this browser</div>
            </div>
        );
    }

    return (
        <div ref={containerRef} class="scene3d-container">
            <div class="scene3d-hud">
                <span>Left drag: Move objects</span>
                <span>Right drag: Orbit</span>
                <span>Scroll: Zoom</span>
                <button class="scene3d-reset" onClick={() => resetRef.current?.()}>
                    Reset
                </button>
            </div>
        </div>
    );
}
