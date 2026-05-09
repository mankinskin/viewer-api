import { useEffect } from 'preact/hooks';

import type { Mat4, Vec3 } from '../../utils/math3d';
import {
    mat4Multiply,
    mat4Inverse,
    mat4TransformDir,
    mat4TransformPoint,
    rayAABBIntersect,
    screenToRay,
    vec3Normalize,
    type Ray,
} from '../../utils/math3d';
import {
    overlayGpu,
    registerOverlayRenderer,
    setOverlayCameraPos,
    setOverlayParticleVP,
    setOverlayParticleViewport,
    setOverlayRefDepth,
    setOverlayWorldScale,
    type OverlayRenderCallback,
    unregisterOverlayRenderer,
} from '../WgpuOverlay/WgpuOverlay';

import shaderSource from './scene3d.wgsl?raw';
import {
    createCubeGeometry,
    createGridLineData,
    CUBE_VERTS,
    GRID_LINE_FLOATS,
    INITIAL_OBJECTS,
    MAX_DRAWS,
    type SceneObject,
    type SceneState,
    UNIFORM_STRIDE,
} from './scene3d-data';
import { registerScene3dInputHandlers } from './scene3d-input';

interface Scene3dOverlayOptions {
    containerRef: { current: HTMLDivElement | null };
    resetRef: { current: (() => void) | null };
    stateRef: { current: SceneState };
    gpu: typeof overlayGpu.value;
    setNoWebGpu: (value: boolean) => void;
    getCamPos: () => Vec3;
    getViewProj: (aspectRatio: number) => Mat4;
    getModelMatrix: (obj: SceneObject, time: number, idx: number) => Mat4;
}

export function useScene3dOverlay({
    containerRef,
    resetRef,
    stateRef,
    gpu,
    setNoWebGpu,
    getCamPos,
    getViewProj,
    getModelMatrix,
}: Scene3dOverlayOptions) {
    useEffect(() => {
        const container = containerRef.current;
        if (!gpu || !container) {
            if (!gpu && !('gpu' in navigator)) setNoWebGpu(true);
            return;
        }

        const { device, format } = gpu;
        const state = stateRef.current;

        const shader = device.createShaderModule({ code: shaderSource });
        const cubeVerts = createCubeGeometry();
        const vertexBuffer = device.createBuffer({
            size: cubeVerts.byteLength,
            usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
        });
        device.queue.writeBuffer(vertexBuffer, 0, cubeVerts.buffer as ArrayBuffer);

        const uniformBuffer = device.createBuffer({
            size: UNIFORM_STRIDE * MAX_DRAWS,
            usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
        });

        const bindGroupLayout = device.createBindGroupLayout({
            entries: [{
                binding: 0,
                visibility: GPUShaderStage.VERTEX | GPUShaderStage.FRAGMENT,
                buffer: { type: 'uniform', hasDynamicOffset: true, minBindingSize: 192 },
            }],
        });

        const bindGroup = device.createBindGroup({
            layout: bindGroupLayout,
            entries: [{ binding: 0, resource: { buffer: uniformBuffer, size: 192 } }],
        });

        const pipeline = device.createRenderPipeline({
            layout: device.createPipelineLayout({ bindGroupLayouts: [bindGroupLayout] }),
            vertex: {
                module: shader,
                entryPoint: 'vs_main',
                buffers: [{
                    arrayStride: 24,
                    attributes: [
                        { shaderLocation: 0, offset: 0, format: 'float32x3' as GPUVertexFormat },
                        { shaderLocation: 1, offset: 12, format: 'float32x3' as GPUVertexFormat },
                    ],
                }],
            },
            fragment: { module: shader, entryPoint: 'fs_main', targets: [{ format }] },
            depthStencil: {
                format: 'depth24plus',
                depthWriteEnabled: true,
                depthCompare: 'less',
            },
            primitive: { topology: 'triangle-list', cullMode: 'back' },
        });

        const { data: gridData, count: gridCount } = createGridLineData();
        const quadVerts = new Float32Array([
            -1, -1, 1, -1, 1, 1,
            -1, -1, 1, 1, -1, 1,
        ]);
        const quadVB = device.createBuffer({
            size: quadVerts.byteLength,
            usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
        });
        device.queue.writeBuffer(quadVB, 0, quadVerts.buffer as ArrayBuffer);

        const gridIB = device.createBuffer({
            size: gridData.byteLength,
            usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
        });
        device.queue.writeBuffer(gridIB, 0, gridData.buffer as ArrayBuffer);

        const gridUB = device.createBuffer({
            size: 128,
            usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
        });

        const gridBGL = device.createBindGroupLayout({
            entries: [{
                binding: 1,
                visibility: GPUShaderStage.VERTEX | GPUShaderStage.FRAGMENT,
                buffer: { type: 'uniform', minBindingSize: 96 },
            }],
        });

        const gridBG = device.createBindGroup({
            layout: gridBGL,
            entries: [{ binding: 1, resource: { buffer: gridUB } }],
        });

        const gridPipeline = device.createRenderPipeline({
            layout: device.createPipelineLayout({ bindGroupLayouts: [gridBGL] }),
            vertex: {
                module: shader,
                entryPoint: 'vs_grid',
                buffers: [
                    {
                        arrayStride: 8,
                        stepMode: 'vertex',
                        attributes: [{ shaderLocation: 0, offset: 0, format: 'float32x2' as GPUVertexFormat }],
                    },
                    {
                        arrayStride: GRID_LINE_FLOATS * 4,
                        stepMode: 'instance',
                        attributes: [
                            { shaderLocation: 1, offset: 0, format: 'float32x3' as GPUVertexFormat },
                            { shaderLocation: 2, offset: 12, format: 'float32x3' as GPUVertexFormat },
                            { shaderLocation: 3, offset: 24, format: 'float32x4' as GPUVertexFormat },
                            { shaderLocation: 4, offset: 40, format: 'float32' as GPUVertexFormat },
                        ],
                    },
                ],
            },
            fragment: {
                module: shader,
                entryPoint: 'fs_grid',
                targets: [{
                    format,
                    blend: {
                        color: {
                            srcFactor: 'src-alpha',
                            dstFactor: 'one-minus-src-alpha',
                            operation: 'add',
                        },
                        alpha: {
                            srcFactor: 'one',
                            dstFactor: 'one-minus-src-alpha',
                            operation: 'add',
                        },
                    },
                }],
            },
            depthStencil: {
                format: 'depth24plus',
                depthWriteEnabled: false,
                depthCompare: 'less-equal',
            },
            primitive: { topology: 'triangle-list' },
        });

        const uniformData = new ArrayBuffer(UNIFORM_STRIDE * MAX_DRAWS);
        const lightDir: Vec3 = vec3Normalize([0.5, 0.8, 0.3]);

        const renderCallback: OverlayRenderCallback = (
            pass,
            dev,
            time,
            _dt,
            canvasW,
            canvasH,
        ) => {
            const rect = container.getBoundingClientRect();
            const vx = Math.max(0, Math.round(rect.left));
            const vy = Math.max(0, Math.round(rect.top));
            const vw = Math.min(Math.round(rect.width), canvasW - vx);
            const vh = Math.min(Math.round(rect.height), canvasH - vy);
            if (vw <= 0 || vh <= 0) return;

            pass.setViewport(vx, vy, vw, vh, 0, 1);
            pass.setScissorRect(vx, vy, vw, vh);

            const camPos = getCamPos();
            const viewProj = getViewProj(vw / vh);

            const canvasWidth = canvasW;
            const canvasHeight = canvasH;
            const sx = vw / canvasWidth;
            const sy = vh / canvasHeight;
            const tx = (2 * vx + vw) / canvasWidth - 1;
            const ty = 1 - (2 * vy + vh) / canvasHeight;
            const post = new Float32Array(16);
            post[0] = sx;
            post[5] = sy;
            post[10] = 1;
            post[15] = 1;
            post[12] = tx;
            post[13] = ty;

            {
                const invSubVP = mat4Inverse(viewProj);
                if (invSubVP) {
                    const invPost = new Float32Array(16);
                    invPost[0] = 1 / sx;
                    invPost[5] = 1 / sy;
                    invPost[10] = 1;
                    invPost[15] = 1;
                    invPost[12] = -tx / sx;
                    invPost[13] = -ty / sy;
                    const fullVPOut = mat4Multiply(post, viewProj);
                    const fullInvVP = mat4Multiply(invSubVP, invPost);
                    setOverlayParticleVP(fullVPOut, fullInvVP);
                    setOverlayCameraPos(camPos[0], camPos[1], camPos[2]);
                }
                setOverlayParticleViewport(vx, vy, vw, vh);
            }

            {
                const targetX = state.camTarget[0];
                const targetY = state.camTarget[1];
                const targetZ = state.camTarget[2];
                const tw = viewProj[3] * targetX + viewProj[7] * targetY + viewProj[11] * targetZ + viewProj[15];
                const refZ = tw > 0.001
                    ? (viewProj[2] * targetX + viewProj[6] * targetY + viewProj[10] * targetZ + viewProj[14]) / tw
                    : 0;
                setOverlayRefDepth(refZ);

                const dist = Math.sqrt(
                    (camPos[0] - targetX) ** 2 +
                    (camPos[1] - targetY) ** 2 +
                    (camPos[2] - targetZ) ** 2,
                );
                const fov = Math.PI / 4;
                const worldScale = 2 * dist * Math.tan(fov / 2) / vh;
                setOverlayWorldScale(worldScale);
            }

            if (state.dragIdx < 0 && !state.orbiting) {
                const invVP = mat4Inverse(viewProj);
                if (invVP) {
                    const ray = screenToRay(state.mouseX, state.mouseY, vw, vh, invVP);
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
                        const t = rayAABBIntersect(localRay, [-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]);
                        if (t !== null && t < bestT) {
                            bestT = t;
                            bestIdx = i;
                        }
                    }
                    state.hoverIdx = bestIdx;
                }
            }

            container.style.cursor = state.dragIdx >= 0
                ? 'grabbing'
                : state.hoverIdx >= 0
                    ? 'grab'
                    : 'default';

            let draws = 0;
            for (let i = 0; i < state.objects.length; i++) {
                const obj = state.objects[i];
                if (!obj) continue;
                const model = getModelMatrix(obj, time, i);
                const fv = new Float32Array(uniformData, UNIFORM_STRIDE * draws, 48);
                fv.set(viewProj, 0);
                fv.set(model, 16);
                fv.set(obj.color, 32);
                fv.set([lightDir[0], lightDir[1], lightDir[2], 0], 36);
                fv.set([camPos[0], camPos[1], camPos[2], 0], 40);
                fv.set([0, state.hoverIdx === i ? 1 : 0, time, state.dragIdx === i ? 1 : 0], 44);
                draws++;
            }
            dev.queue.writeBuffer(
                uniformBuffer,
                0,
                new Uint8Array(uniformData),
                0,
                UNIFORM_STRIDE * draws,
            );

            const gridUniformData = new Float32Array(24);
            gridUniformData.set(viewProj, 0);
            gridUniformData.set([camPos[0], camPos[1], camPos[2], 0], 16);
            gridUniformData.set([vw, vh, time, 0], 20);
            dev.queue.writeBuffer(gridUB, 0, gridUniformData);

            pass.setPipeline(gridPipeline);
            pass.setVertexBuffer(0, quadVB);
            pass.setVertexBuffer(1, gridIB);
            pass.setBindGroup(0, gridBG);
            pass.draw(6, gridCount);

            pass.setPipeline(pipeline);
            pass.setVertexBuffer(0, vertexBuffer);
            for (let i = 0; i < draws; i++) {
                pass.setBindGroup(0, bindGroup, [UNIFORM_STRIDE * i]);
                pass.draw(CUBE_VERTS);
            }

            pass.setViewport(0, 0, canvasW, canvasH, 0, 1);
            pass.setScissorRect(0, 0, canvasW, canvasH);
        };

        registerOverlayRenderer(renderCallback);
        const cleanupInputs = registerScene3dInputHandlers({
            container,
            state,
            getViewProj,
            getModelMatrix,
        });

        resetRef.current = () => {
            INITIAL_OBJECTS.forEach((initial, i) => {
                const obj = state.objects[i];
                if (obj) obj.position = [...initial.position] as Vec3;
            });
            state.camYaw = 0.6;
            state.camPitch = 0.35;
            state.camDist = 10;
        };

        return () => {
            unregisterOverlayRenderer(renderCallback);
            cleanupInputs();
            vertexBuffer.destroy();
            uniformBuffer.destroy();
            quadVB.destroy();
            gridIB.destroy();
            gridUB.destroy();
        };
    }, [containerRef, getCamPos, getModelMatrix, getViewProj, gpu, resetRef, setNoWebGpu, stateRef]);
}
