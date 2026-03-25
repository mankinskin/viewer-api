/// <reference types="@webgpu/types" />
/**
 * Scene3D — 3D cube scene rendered via the shared WgpuOverlay canvas.
 *
 * Uses the overlay render callback system to draw into a viewport region
 * (like HypergraphView).  The shared depth buffer enables proper depth
 * testing for opaque 3D objects.
 */
import { useRef, useEffect, useState, useCallback } from 'preact/hooks';
import shaderSource from './scene3d.wgsl?raw';
import './scene3d.css';
import {
    Vec3, Mat4,
    vec3Normalize,
    mat4Perspective, mat4LookAt, mat4Multiply,
    mat4Translate, mat4ScaleMat, mat4RotateY, mat4Inverse,
    mat4TransformPoint, mat4TransformDir,
    screenToRay, rayAABBIntersect, rayPlaneIntersect,
    Ray,
} from '../../utils/math3d';
import {
    overlayGpu,
    registerOverlayRenderer,
    unregisterOverlayRenderer,
    setOverlayParticleVP,
    setOverlayParticleViewport,
    setOverlayRefDepth,
    setOverlayWorldScale,
    setOverlayCameraPos,
    type OverlayRenderCallback,
} from '../WgpuOverlay/WgpuOverlay';

// ── types ──

interface SceneObject {
    position: Vec3;
    scale: Vec3;
    color: [number, number, number, number];
    rotationY: number;
}

// ── geometry ──

/** Unit cube (−0.5 → 0.5), 36 verts, interleaved pos+normal (6 floats each) */
function createCubeGeometry(): Float32Array {
    const faces: { n: Vec3; v: Vec3[] }[] = [
        { n: [0, 0, 1], v: [[-0.5, -0.5, 0.5], [0.5, -0.5, 0.5], [0.5, 0.5, 0.5], [-0.5, 0.5, 0.5]] },       // +Z
        { n: [0, 0, -1], v: [[0.5, -0.5, -0.5], [-0.5, -0.5, -0.5], [-0.5, 0.5, -0.5], [0.5, 0.5, -0.5]] },    // −Z
        { n: [1, 0, 0], v: [[0.5, -0.5, 0.5], [0.5, -0.5, -0.5], [0.5, 0.5, -0.5], [0.5, 0.5, 0.5]] },         // +X
        { n: [-1, 0, 0], v: [[-0.5, -0.5, -0.5], [-0.5, -0.5, 0.5], [-0.5, 0.5, 0.5], [-0.5, 0.5, -0.5]] },    // −X
        { n: [0, 1, 0], v: [[-0.5, 0.5, 0.5], [0.5, 0.5, 0.5], [0.5, 0.5, -0.5], [-0.5, 0.5, -0.5]] },         // +Y
        { n: [0, -1, 0], v: [[-0.5, -0.5, -0.5], [0.5, -0.5, -0.5], [0.5, -0.5, 0.5], [-0.5, -0.5, 0.5]] },     // −Y
    ];
    const d: number[] = [];
    for (const { n, v } of faces) {
        const [a, b, c, e] = v as [Vec3, Vec3, Vec3, Vec3];
        d.push(a[0], a[1], a[2], n[0], n[1], n[2]);
        d.push(b[0], b[1], b[2], n[0], n[1], n[2]);
        d.push(c[0], c[1], c[2], n[0], n[1], n[2]);
        d.push(a[0], a[1], a[2], n[0], n[1], n[2]);
        d.push(c[0], c[1], c[2], n[0], n[1], n[2]);
        d.push(e[0], e[1], e[2], n[0], n[1], n[2]);
    }
    return new Float32Array(d);
}

// ── constants ──

const UNIFORM_STRIDE = 256;   // WebGPU minUniformBufferOffsetAlignment
const MAX_DRAWS = 16;
const CUBE_VERTS = 36;

// Grid line constants (same format as HypergraphView)
const GRID_LINE_FLOATS = 12;  // startXYZ(3) + endXYZ(3) + RGBA(4) + width(1) + pad(1)
const GRID_EXTENT = 10;
const GRID_STEP = 1;

function createGridLineData(): { data: Float32Array; count: number } {
    const lines: number[] = [];
    // regular grid lines
    for (let i = -GRID_EXTENT; i <= GRID_EXTENT; i += GRID_STEP) {
        // X-parallel lines
        lines.push(i, 0, -GRID_EXTENT,  i, 0, GRID_EXTENT,  0.25, 0.24, 0.22, 0.08,  0, 0);
        // Z-parallel lines
        lines.push(-GRID_EXTENT, 0, i,  GRID_EXTENT, 0, i,  0.25, 0.24, 0.22, 0.08,  0, 0);
    }
    // axis lines (highlighted)
    lines.push(-GRID_EXTENT, 0, 0,  GRID_EXTENT, 0, 0,  0.55, 0.22, 0.18, 0.18,  1, 0); // X red
    lines.push(0, 0, -GRID_EXTENT,  0, 0, GRID_EXTENT,  0.18, 0.22, 0.55, 0.18,  1, 0); // Z blue
    return { data: new Float32Array(lines), count: lines.length / GRID_LINE_FLOATS };
}

const INITIAL_OBJECTS: SceneObject[] = [
    { position: [0, 0.5, 0], scale: [1, 1, 1], color: [0.95, 0.25, 0.21, 1], rotationY: 0 },
    { position: [2.5, 1.0, 0.8], scale: [0.5, 2.0, 0.5], color: [0.18, 0.60, 0.95, 1], rotationY: 0.2 },
    { position: [-2.2, 0.2, -1.2], scale: [1.8, 0.4, 1.5], color: [0.22, 0.88, 0.38, 1], rotationY: 0.5 },
    { position: [1.2, 0.25, -2.3], scale: [0.5, 0.5, 0.5], color: [0.98, 0.85, 0.15, 1], rotationY: 0.8 },
    { position: [-1.5, 0.5, 2.2], scale: [1, 1, 1], color: [0.72, 0.32, 0.95, 1], rotationY: 1.1 },
    { position: [3.2, 0.45, -2.0], scale: [0.9, 0.9, 1.4], color: [0.15, 0.85, 0.82, 1], rotationY: 0.4 },
];

// ── component ──

export function Scene3D() {
    const containerRef = useRef<HTMLDivElement>(null);
    const resetRef = useRef<(() => void) | null>(null);
    const [noWebGpu, setNoWebGpu] = useState(false);

    const gpu = overlayGpu.value;

    // Scene state refs (survive re-renders, accessed by callback)
    const stateRef = useRef({
        objects: INITIAL_OBJECTS.map(o => ({
            ...o,
            position: [...o.position] as Vec3,
            scale: [...o.scale] as Vec3,
            color: [...o.color] as [number, number, number, number],
        })),
        camYaw: 0.6,
        camPitch: 0.35,
        camDist: 10,
        camTarget: [0, 0.5, 0] as Vec3,
        dragIdx: -1,
        dragPlaneY: 0,
        dragOffset: [0, 0, 0] as Vec3,
        orbiting: false,
        lastMX: 0,
        lastMY: 0,
        hoverIdx: -1,
        mouseX: 0,
        mouseY: 0,
        startTime: performance.now() / 1000,
    });

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

    // Register overlay callback
    useEffect(() => {
        const container = containerRef.current;
        if (!gpu || !container) {
            if (!gpu && !('gpu' in navigator)) setNoWebGpu(true);
            return;
        }

        const { device, format } = gpu;
        const s = stateRef.current;

        // ── Create pipeline & buffers ──
        const shader = device.createShaderModule({ code: shaderSource });
        const cubeVerts = createCubeGeometry();

        const vb = device.createBuffer({
            size: cubeVerts.byteLength,
            usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
        });
        device.queue.writeBuffer(vb, 0, cubeVerts.buffer as ArrayBuffer);

        const ub = device.createBuffer({
            size: UNIFORM_STRIDE * MAX_DRAWS,
            usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
        });

        const bgl = device.createBindGroupLayout({
            entries: [{
                binding: 0,
                visibility: GPUShaderStage.VERTEX | GPUShaderStage.FRAGMENT,
                buffer: { type: 'uniform', hasDynamicOffset: true, minBindingSize: 192 },
            }],
        });

        const bg = device.createBindGroup({
            layout: bgl,
            entries: [{ binding: 0, resource: { buffer: ub, size: 192 } }],
        });

        const pipeline = device.createRenderPipeline({
            layout: device.createPipelineLayout({ bindGroupLayouts: [bgl] }),
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
            fragment: {
                module: shader,
                entryPoint: 'fs_main',
                targets: [{ format }],
            },
            depthStencil: {
                format: 'depth24plus',
                depthWriteEnabled: true,
                depthCompare: 'less',
            },
            primitive: { topology: 'triangle-list', cullMode: 'back' },
        });

        // ── Grid pipeline & buffers ──
        const { data: gridData, count: gridCount } = createGridLineData();

        // Quad vertices for line expansion
        const quadVerts = new Float32Array([
            -1, -1,  1, -1,  1, 1,
            -1, -1,  1, 1,  -1, 1,
        ]);
        const quadVB = device.createBuffer({
            size: quadVerts.byteLength,
            usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
        });
        device.queue.writeBuffer(quadVB, 0, quadVerts.buffer as ArrayBuffer);

        // Grid instance buffer
        const gridIB = device.createBuffer({
            size: gridData.byteLength,
            usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
        });
        device.queue.writeBuffer(gridIB, 0, gridData.buffer as ArrayBuffer);

        // Grid uniform buffer (viewProj + cameraPos + screenSize = 16 + 4 + 4 = 24 floats = 96 bytes)
        const gridUB = device.createBuffer({
            size: 128,  // padded
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

        const gridVertexBuffers: GPUVertexBufferLayout[] = [
            {
                arrayStride: 8, stepMode: 'vertex',
                attributes: [{ shaderLocation: 0, offset: 0, format: 'float32x2' as GPUVertexFormat }],
            },
            {
                arrayStride: GRID_LINE_FLOATS * 4, stepMode: 'instance',
                attributes: [
                    { shaderLocation: 1, offset: 0,  format: 'float32x3' as GPUVertexFormat },  // startPos
                    { shaderLocation: 2, offset: 12, format: 'float32x3' as GPUVertexFormat },  // endPos
                    { shaderLocation: 3, offset: 24, format: 'float32x4' as GPUVertexFormat },  // color
                    { shaderLocation: 4, offset: 40, format: 'float32'   as GPUVertexFormat },  // width
                ],
            },
        ];

        const gridPipeline = device.createRenderPipeline({
            layout: device.createPipelineLayout({ bindGroupLayouts: [gridBGL] }),
            vertex: {
                module: shader,
                entryPoint: 'vs_grid',
                buffers: gridVertexBuffers,
            },
            fragment: {
                module: shader,
                entryPoint: 'fs_grid',
                targets: [{
                    format,
                    blend: {
                        color: { srcFactor: 'src-alpha', dstFactor: 'one-minus-src-alpha', operation: 'add' },
                        alpha: { srcFactor: 'one', dstFactor: 'one-minus-src-alpha', operation: 'add' },
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

        // ── Render state ──
        const uniformBuf = new ArrayBuffer(UNIFORM_STRIDE * MAX_DRAWS);
        const lightDir: Vec3 = vec3Normalize([0.5, 0.8, 0.3]);

        // ── Render callback ──
        const renderCallback: OverlayRenderCallback = (pass, dev, time, _dt, canvasW, canvasH, _depthView) => {
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

            // ── Pass viewProj to particle system for world-space projection ──
            const W = canvasW, H = canvasH;
            {
                const sx = vw / W, sy = vh / H;
                const tx = (2 * vx + vw) / W - 1;
                const ty = 1 - (2 * vy + vh) / H;
                const post = new Float32Array(16);
                post[0] = sx; post[5] = sy; post[10] = 1; post[15] = 1;
                post[12] = tx; post[13] = ty;
                const fullVP = mat4Multiply(post, viewProj);
                const invSubVP = mat4Inverse(viewProj);
                if (invSubVP) {
                    const invPost = new Float32Array(16);
                    invPost[0] = 1 / sx; invPost[5] = 1 / sy; invPost[10] = 1; invPost[15] = 1;
                    invPost[12] = -tx / sx; invPost[13] = -ty / sy;
                    const fullInvVP = mat4Multiply(invSubVP, invPost);
                    setOverlayParticleVP(fullVP, fullInvVP);
                    setOverlayCameraPos(camPos[0], camPos[1], camPos[2]);
                }
                setOverlayParticleViewport(vx, vy, vw, vh);
            }

            // Compute reference depth and world scale
            {
                const ttx = s.camTarget[0], tty = s.camTarget[1], ttz = s.camTarget[2];
                const vp = viewProj;
                const tw = vp[3]*ttx + vp[7]*tty + vp[11]*ttz + vp[15];
                const refZ = tw > 0.001 ? (vp[2]*ttx + vp[6]*tty + vp[10]*ttz + vp[14]) / tw : 0;
                setOverlayRefDepth(refZ);

                const dist = Math.sqrt(
                    (camPos[0] - ttx) ** 2 + (camPos[1] - tty) ** 2 + (camPos[2] - ttz) ** 2
                );
                const fov = Math.PI / 4;
                const worldScale = 2 * dist * Math.tan(fov / 2) / vh;
                setOverlayWorldScale(worldScale);
            }

            // ── Hover detection ──
            if (s.dragIdx < 0 && !s.orbiting) {
                const invVP = mat4Inverse(viewProj);
                if (invVP) {
                    const ray = screenToRay(s.mouseX, s.mouseY, vw, vh, invVP);
                    let bestT = Infinity;
                    let bestIdx = -1;
                    for (let i = 0; i < s.objects.length; i++) {
                        const obj = s.objects[i];
                        if (!obj) continue;
                        const model = getModelMatrix(obj, time, i);
                        const invModel = mat4Inverse(model);
                        if (!invModel) continue;
                        const localRay: Ray = {
                            origin: mat4TransformPoint(invModel, ray.origin),
                            direction: mat4TransformDir(invModel, ray.direction),
                        };
                        const t = rayAABBIntersect(localRay, [-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]);
                        if (t !== null && t < bestT) { bestT = t; bestIdx = i; }
                    }
                    s.hoverIdx = bestIdx;
                }
            }

            // Cursor style
            if (s.dragIdx >= 0) container.style.cursor = 'grabbing';
            else if (s.hoverIdx >= 0) container.style.cursor = 'grab';
            else container.style.cursor = 'default';

            // ── Fill uniform data ──
            let draws = 0;

            // Objects
            for (let i = 0; i < s.objects.length; i++) {
                const obj = s.objects[i];
                if (!obj) continue;
                const model = getModelMatrix(obj, time, i);
                const fv = new Float32Array(uniformBuf, UNIFORM_STRIDE * draws, 48);
                fv.set(viewProj, 0);
                fv.set(model, 16);
                fv.set(obj.color, 32);
                fv.set([lightDir[0], lightDir[1], lightDir[2], 0], 36);
                fv.set([camPos[0], camPos[1], camPos[2], 0], 40);
                fv.set([0, s.hoverIdx === i ? 1 : 0, time, s.dragIdx === i ? 1 : 0], 44);
                draws++;
            }

            dev.queue.writeBuffer(ub, 0, new Uint8Array(uniformBuf), 0, UNIFORM_STRIDE * draws);

            // ── Update grid uniforms ──
            const gridUniformData = new Float32Array(24);
            gridUniformData.set(viewProj, 0);
            gridUniformData.set([camPos[0], camPos[1], camPos[2], 0], 16);
            gridUniformData.set([vw, vh, time, 0], 20);
            dev.queue.writeBuffer(gridUB, 0, gridUniformData);

            // ── Draw grid lines ──
            pass.setPipeline(gridPipeline);
            pass.setVertexBuffer(0, quadVB);
            pass.setVertexBuffer(1, gridIB);
            pass.setBindGroup(0, gridBG);
            pass.draw(6, gridCount);

            // ── Draw cubes ──
            pass.setPipeline(pipeline);
            pass.setVertexBuffer(0, vb);
            for (let i = 0; i < draws; i++) {
                pass.setBindGroup(0, bg, [UNIFORM_STRIDE * i]);
                pass.draw(CUBE_VERTS);
            }

            // Restore viewport/scissor
            pass.setViewport(0, 0, canvasW, canvasH, 0, 1);
            pass.setScissorRect(0, 0, canvasW, canvasH);
        };

        registerOverlayRenderer(renderCallback);

        // ── Mouse handlers ──
        const onMouseDown = (e: MouseEvent) => {
            const rect = container.getBoundingClientRect();
            s.mouseX = e.clientX - rect.left;
            s.mouseY = e.clientY - rect.top;
            s.lastMX = e.clientX;
            s.lastMY = e.clientY;

            const cw = container.clientWidth;
            const ch = container.clientHeight;

            if (e.button === 0) {
                const time = performance.now() / 1000 - s.startTime;
                const vp = getViewProj(cw / ch);
                const invVP = mat4Inverse(vp);
                if (invVP) {
                    const ray = screenToRay(s.mouseX, s.mouseY, cw, ch, invVP);
                    let bestT = Infinity;
                    let bestIdx = -1;
                    for (let i = 0; i < s.objects.length; i++) {
                        const obj = s.objects[i];
                        if (!obj) continue;
                        const model = getModelMatrix(obj, time, i);
                        const invModel = mat4Inverse(model);
                        if (!invModel) continue;
                        const localRay: Ray = {
                            origin: mat4TransformPoint(invModel, ray.origin),
                            direction: mat4TransformDir(invModel, ray.direction),
                        };
                        const t = rayAABBIntersect(localRay, [-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]);
                        if (t !== null && t < bestT) { bestT = t; bestIdx = i; }
                    }
                    const bestObj = bestIdx >= 0 ? s.objects[bestIdx] : undefined;
                    if (bestObj) {
                        s.dragIdx = bestIdx;
                        s.dragPlaneY = bestObj.position[1];
                        const pt = rayPlaneIntersect(ray, s.dragPlaneY);
                        if (pt) {
                            s.dragOffset = [
                                bestObj.position[0] - pt[0],
                                0,
                                bestObj.position[2] - pt[2],
                            ];
                        }
                        e.preventDefault();
                    } else {
                        s.orbiting = true;
                    }
                }
            } else if (e.button === 2) {
                s.orbiting = true;
                e.preventDefault();
            }
        };

        const onMouseMove = (e: MouseEvent) => {
            const rect = container.getBoundingClientRect();
            s.mouseX = e.clientX - rect.left;
            s.mouseY = e.clientY - rect.top;

            if (s.dragIdx >= 0) {
                const dragObj = s.objects[s.dragIdx];
                if (!dragObj) { s.dragIdx = -1; return; }
                const cw = container.clientWidth;
                const ch = container.clientHeight;
                const vp = getViewProj(cw / ch);
                const invVP = mat4Inverse(vp);
                if (invVP) {
                    const ray = screenToRay(s.mouseX, s.mouseY, cw, ch, invVP);
                    const pt = rayPlaneIntersect(ray, s.dragPlaneY);
                    if (pt) {
                        dragObj.position[0] = pt[0] + s.dragOffset[0];
                        dragObj.position[2] = pt[2] + s.dragOffset[2];
                    }
                }
                return;
            }

            if (s.orbiting) {
                const dx = e.clientX - s.lastMX;
                const dy = e.clientY - s.lastMY;
                s.camYaw += dx * 0.005;
                s.camPitch = Math.max(-1.4, Math.min(1.4, s.camPitch + dy * 0.005));
                s.lastMX = e.clientX;
                s.lastMY = e.clientY;
            }
        };

        const onMouseUp = () => {
            s.dragIdx = -1;
            s.orbiting = false;
        };

        const onWheel = (e: WheelEvent) => {
            s.camDist = Math.max(3, Math.min(30, s.camDist + e.deltaY * 0.01));
            e.preventDefault();
        };

        const onCtx = (e: Event) => e.preventDefault();

        // ── Touch handlers ──
        let touchFingers = 0;
        let touchLastX = 0, touchLastY = 0;
        let touchLastDist = 0;
        let touchLastMidX = 0, touchLastMidY = 0;

        // Do not consume touches on interactive HUD controls (e.g. reset button).
        const TOUCH_UI_SELECTOR = '.scene3d-hud button, .scene3d-hud a, .scene3d-hud input';

        const touchDist2 = (t1: Touch, t2: Touch): number => {
            const dx = t1.clientX - t2.clientX;
            const dy = t1.clientY - t2.clientY;
            return Math.sqrt(dx * dx + dy * dy);
        };

        const onTouchStart = (e: TouchEvent) => {
            const target = e.target as HTMLElement;
            if (target.closest(TOUCH_UI_SELECTOR)) return;

            e.preventDefault();
            touchFingers = e.touches.length;
            if (e.touches.length === 1) {
                const t = e.touches[0];
                if (!t) return;
                touchLastX = t.clientX;
                touchLastY = t.clientY;
            } else if (e.touches.length === 2) {
                const t1 = e.touches[0];
                const t2 = e.touches[1];
                if (!t1 || !t2) return;
                touchLastDist = touchDist2(t1, t2);
                touchLastMidX = (t1.clientX + t2.clientX) / 2;
                touchLastMidY = (t1.clientY + t2.clientY) / 2;
            }
        };

        const onTouchMove = (e: TouchEvent) => {
            const target = e.target as HTMLElement;
            if (target.closest(TOUCH_UI_SELECTOR)) return;

            e.preventDefault();
            if (e.touches.length === 1 && touchFingers === 1) {
                // Single finger: orbit
                const t = e.touches[0];
                if (!t) return;
                const dx = t.clientX - touchLastX;
                const dy = t.clientY - touchLastY;
                touchLastX = t.clientX;
                touchLastY = t.clientY;
                s.camYaw += dx * 0.005;
                s.camPitch = Math.max(-1.4, Math.min(1.4, s.camPitch + dy * 0.005));
            } else if (e.touches.length === 2) {
                const t1 = e.touches[0];
                const t2 = e.touches[1];
                if (!t1 || !t2) return;
                const dist = touchDist2(t1, t2);
                const mx = (t1.clientX + t2.clientX) / 2;
                const my = (t1.clientY + t2.clientY) / 2;

                // Pinch → zoom
                if (touchLastDist > 0) {
                    const scale = touchLastDist / dist;
                    s.camDist = Math.max(3, Math.min(30, s.camDist * scale));
                }
                touchLastDist = dist;

                // Two-finger pan
                const panDx = mx - touchLastMidX;
                const panDy = my - touchLastMidY;
                const speed = s.camDist * 0.002;
                const cosY = Math.cos(s.camYaw);
                const sinY = Math.sin(s.camYaw);
                s.camTarget = [
                    s.camTarget[0] - panDx * speed * cosY,
                    s.camTarget[1] + panDy * speed,
                    s.camTarget[2] + panDx * speed * sinY,
                ];
                touchLastMidX = mx;
                touchLastMidY = my;
            }
        };

        const onTouchEnd = (e: TouchEvent) => {
            touchFingers = e.touches.length;
        };

        const onTouchCancel = () => {
            touchFingers = 0;
            touchLastDist = 0;
        };

        container.addEventListener('mousedown', onMouseDown);
        window.addEventListener('mousemove', onMouseMove);
        window.addEventListener('mouseup', onMouseUp);
        container.addEventListener('wheel', onWheel, { passive: false });
        container.addEventListener('contextmenu', onCtx);
        container.addEventListener('touchstart', onTouchStart, { passive: false });
        container.addEventListener('touchmove', onTouchMove, { passive: false });
        container.addEventListener('touchend', onTouchEnd);
        container.addEventListener('touchcancel', onTouchCancel);

        // ── Reset function ──
        resetRef.current = () => {
            INITIAL_OBJECTS.forEach((init, i) => {
                const obj = s.objects[i];
                if (obj) obj.position = [...init.position] as Vec3;
            });
            s.camYaw = 0.6;
            s.camPitch = 0.35;
            s.camDist = 10;
        };

        return () => {
            unregisterOverlayRenderer(renderCallback);
            container.removeEventListener('mousedown', onMouseDown);
            window.removeEventListener('mousemove', onMouseMove);
            window.removeEventListener('mouseup', onMouseUp);
            container.removeEventListener('wheel', onWheel);
            container.removeEventListener('contextmenu', onCtx);
            container.removeEventListener('touchstart', onTouchStart);
            container.removeEventListener('touchmove', onTouchMove);
            container.removeEventListener('touchend', onTouchEnd);
            container.removeEventListener('touchcancel', onTouchCancel);
            vb.destroy();
            ub.destroy();
        };
    }, [gpu, getCamPos, getViewProj, getModelMatrix]);

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
