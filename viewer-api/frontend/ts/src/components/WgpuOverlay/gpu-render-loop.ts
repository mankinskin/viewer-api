/// <reference types="@webgpu/types" />
/**
 * GPU render loop — rAF orchestration, uniform packing, compute/render passes.
 *
 * Single-pass architecture on one opaque canvas:
 *   1. **Compute pass** — particle physics simulation (when particles enabled)
 *   2. **Render pass** (single, on back canvas):
 *      a. Background quad — smoke, element rects, CRT, grain, vignette
 *      b. Overlay callbacks — 3D edges, grids, cubes (may change viewport)
 *      c. Particles — sparks, embers, beams, glitter (additive blend)
 *
 * Reads element data from the ElementScanner, uploads via GpuBufferManager,
 * and rebuilds bind groups when the buffer generation changes.
 *
 * Uniform packing: 88 × f32 (352 bytes) = 52 scalars + 4 camera floats + 2 × mat4x4.
 * See types.wgsl Uniforms struct for the field layout.
 *
 * App-specific signals (themeColors, effectSettings, view state) are injected
 * via `AppSchema` so this module has no dependency on any specific application.
 */

import type { GpuPipelines } from './gpu-init';
import type { GpuBufferManager } from './gpu-buffers';
import type { ElementScanner } from './element-scanner';
import type { AppSchema } from './schemas';
import {
    ELEM_FLOATS, NUM_PARTICLES, COMPUTE_WORKGROUP,
    KIND_SELECTED, KIND_PANIC,
    SPARK_START, SPARK_END,
    EMBER_START, EMBER_END,
    RAY_START, RAY_END,
    GLITTER_START, GLITTER_END,
} from './element-types';
import { getOverlayCallbacks, consumeCaptureRequest, hasCaptureRequest, fxEnabled } from './overlay-api';
import { packRenderUniforms } from './gpu-render-loop-uniforms';
import { captureFrame } from './thumbnail-capture';

export class RenderLoop {
    private readonly pipelines: GpuPipelines;
    private readonly buffers: GpuBufferManager;
    private readonly scanner: ElementScanner;
    private readonly canvas: HTMLCanvasElement;
    private readonly schema: AppSchema;

    private computeBindGroup: GPUBindGroup;
    private renderBindGroup: GPUBindGroup;
    private lastGeneration: number;

    private depthTexture: GPUTexture | null = null;
    private depthView: GPUTextureView | null = null;
    private depthW = 0;
    private depthH = 0;

    private animId = 0;
    private cancelled = false;
    private prevTime = performance.now() / 1000;
    private readonly startTime = performance.now();

    private mx = -9999;
    private my = -9999;

    private prevHoverIdx = -1;
    private hoverStartTime = 0;

    private prevRenderMx = -9999;
    private prevRenderMy = -9999;
    private prevCanvasW = 0;
    private prevCanvasH = 0;
    private prevSelectedIdx = -1;
    private prevEffects: object | null = null;
    private prevSparksEnabled = true;
    private prevEmbersEnabled = true;
    private prevBeamsEnabled = true;
    private prevGlitterEnabled = true;
    private frameSkipCounter = 0;  // For 30fps limiter in particles-only mode

    constructor(
        pipelines: GpuPipelines,
        buffers: GpuBufferManager,
        scanner: ElementScanner,
        canvas: HTMLCanvasElement,
        schema: AppSchema,
    ) {
        this.pipelines = pipelines;
        this.buffers = buffers;
        this.scanner = scanner;
        this.canvas = canvas;
        this.schema = schema;

        this.lastGeneration = buffers.generation;
        this.computeBindGroup = this.buildComputeBindGroup();
        this.renderBindGroup = this.buildRenderBindGroup();
    }

    private buildComputeBindGroup(): GPUBindGroup {
        const { device, computeBGL } = this.pipelines;
        const b = this.buffers;
        return device.createBindGroup({
            layout: computeBGL,
            entries: [
                { binding: 0, resource: { buffer: b.uniformBuffer  } },
                { binding: 1, resource: { buffer: b.elemBuffer     } },
                { binding: 2, resource: { buffer: b.particleBuffer } },
                { binding: 3, resource: { buffer: b.paletteBuffer  } },
            ],
        });
    }

    private buildRenderBindGroup(): GPUBindGroup {
        const { device, renderBGL } = this.pipelines;
        const b = this.buffers;
        return device.createBindGroup({
            layout: renderBGL,
            entries: [
                { binding: 0, resource: { buffer: b.uniformBuffer  } },
                { binding: 1, resource: { buffer: b.elemBuffer     } },
                { binding: 2, resource: { buffer: b.particleBuffer } },
                { binding: 3, resource: { buffer: b.paletteBuffer  } },
            ],
        });
    }

    private rebuildBindGroupsIfNeeded(): void {
        const gen = this.buffers.generation;
        if (gen !== this.lastGeneration) {
            this.computeBindGroup = this.buildComputeBindGroup();
            this.renderBindGroup = this.buildRenderBindGroup();
            this.lastGeneration = gen;
        }
    }

    /** Ensure depth texture exists and matches canvas size. */
    private ensureDepth(w: number, h: number): void {
        if (w === this.depthW && h === this.depthH && this.depthTexture) return;
        this.depthTexture?.destroy();
        this.depthTexture = this.pipelines.device.createTexture({
            size: [w, h],
            format: 'depth24plus',
            usage: GPUTextureUsage.RENDER_ATTACHMENT,
        });
        this.depthView = this.depthTexture.createView();
        this.depthW = w;
        this.depthH = h;
    }

    setMouse(x: number, y: number): void {
        this.mx = x;
        this.my = y;
    }

    start(): void {
        this.cancelled = false;
        this.animId = requestAnimationFrame(this.frame);
    }

    stop(): void {
        this.cancelled = true;
        cancelAnimationFrame(this.animId);
    }

    private frame = (): void => {
        if (this.cancelled) return;

        const scanner = this.scanner;
        const dataChanged = scanner.updateFrame();
        const scrollDelta = scanner.consumeScrollDelta();

        const nowSec = performance.now() / 1000;
        const time = (performance.now() - this.startTime) / 1000;
        const dt   = Math.min(nowSec - this.prevTime, 0.05);
        this.prevTime = nowSec;

        const count = scanner.count;
        const data  = scanner.data;
        const mx = this.mx;
        const my = this.my;

        let hoverIdx = -1;
        let hoverKind = -1;
        for (let i = 0; i < count; i++) {
            const base = i * ELEM_FLOATS;
            const ex = data[base]!;
            const ey = data[base + 1]!;
            const ew = data[base + 2]!;
            const eh = data[base + 3]!;
            if (mx >= ex && mx < ex + ew && my >= ey && my < ey + eh) {
                const k = data[base + 5]!;
                if (k >= hoverKind) {
                    hoverIdx = i;
                    hoverKind = k;
                }
            }
        }

        let hoverChanged = false;
        if (hoverIdx !== this.prevHoverIdx) {
            this.hoverStartTime = time;
            this.prevHoverIdx = hoverIdx;
            hoverChanged = true;
        }

        let selectedIdx = -1;
        for (let i = 0; i < count; i++) {
            const base = i * ELEM_FLOATS;
            const k = data[base + 5]!;
            if (k === KIND_SELECTED) {
                selectedIdx = i;
                if (i === hoverIdx) break;
            } else if (k === KIND_PANIC && selectedIdx < 0) {
                selectedIdx = i;
            }
        }

        const eff = this.schema.effectSettings.value;
        const fx = fxEnabled.value;
        const particlesEnabled = fx && (eff.sparksEnabled || eff.embersEnabled
            || eff.beamsEnabled || eff.glitterEnabled);

        const minimalBackground = !fx || ((!eff.smokeEnabled || eff.smokeIntensity === 0)
            && eff.grainIntensity === 0);

        const anyAnimated = fx && (
            (eff.smokeEnabled && eff.smokeIntensity > 0)
            || (eff.crtEnabled && eff.crtFlicker > 0)
            || (hoverIdx >= 0 && (eff.cinderEnabled || particlesEnabled))
            || (selectedIdx >= 0 && eff.beamsEnabled));

        const hasOverlayCallbacks = getOverlayCallbacks().size > 0;
        const inputDirty =
            mx !== this.prevRenderMx || my !== this.prevRenderMy
            || this.canvas.width !== this.prevCanvasW
            || this.canvas.height !== this.prevCanvasH
            || scrollDelta.dx !== 0 || scrollDelta.dy !== 0
            || scanner.didFullRescan || dataChanged
            || hoverChanged
            || selectedIdx !== this.prevSelectedIdx
            || eff !== this.prevEffects
            || hasCaptureRequest();

        if (!anyAnimated && !inputDirty && !hasOverlayCallbacks) {
            this.animId = requestAnimationFrame(this.frame);
            return;
        }

        const particlesOnlyAnimated = anyAnimated && minimalBackground
            && !(eff.smokeEnabled && eff.smokeIntensity > 0)
            && !(eff.crtEnabled && eff.crtFlicker > 0)
            && true;
        if (particlesOnlyAnimated && !inputDirty) {
            this.frameSkipCounter = (this.frameSkipCounter + 1) % 2;
            if (this.frameSkipCounter !== 0) {
                this.animId = requestAnimationFrame(this.frame);
                return;
            }
        } else {
            this.frameSkipCounter = 0;
        }

        this.prevRenderMx = mx;
        this.prevRenderMy = my;
        this.prevCanvasW = this.canvas.width;
        this.prevCanvasH = this.canvas.height;
        this.prevSelectedIdx = selectedIdx;
        this.prevEffects = eff;

        if (!eff.sparksEnabled && this.prevSparksEnabled) {
            this.buffers.resetParticleRange(SPARK_START, SPARK_END - SPARK_START);
        }
        if (!eff.embersEnabled && this.prevEmbersEnabled) {
            this.buffers.resetParticleRange(EMBER_START, EMBER_END - EMBER_START);
        }
        if (!eff.beamsEnabled && this.prevBeamsEnabled) {
            this.buffers.resetParticleRange(RAY_START, RAY_END - RAY_START);
        }
        if (!eff.glitterEnabled && this.prevGlitterEnabled) {
            this.buffers.resetParticleRange(GLITTER_START, GLITTER_END - GLITTER_START);
        }
        this.prevSparksEnabled = eff.sparksEnabled;
        this.prevEmbersEnabled = eff.embersEnabled;
        this.prevBeamsEnabled = eff.beamsEnabled;
        this.prevGlitterEnabled = eff.glitterEnabled;

        const u = this.buffers.uniformF32;
        packRenderUniforms({
            uniforms: u,
            canvas: this.canvas,
            schema: this.schema,
            effects: eff,
            fxEnabled: fx,
            count,
            mx,
            my,
            dt,
            time,
            hoverIdx,
            hoverStartTime: this.hoverStartTime,
            selectedIdx,
            scrollDelta,
        });
        this.buffers.uploadUniforms();

        this.buffers.uploadPalette(this.schema.themeColors.value);
        this.buffers.uploadElements(data, count);
        this.rebuildBindGroupsIfNeeded();

        const { device, context, computePipeline, renderPipeline, particlePipeline } = this.pipelines;
        const enc = device.createCommandEncoder();
        this.ensureDepth(this.canvas.width, this.canvas.height);
        if (particlesEnabled) {
            const computePass = enc.beginComputePass();
            computePass.setPipeline(computePipeline);
            computePass.setBindGroup(0, this.computeBindGroup);
            computePass.dispatchWorkgroups(Math.ceil(NUM_PARTICLES / COMPUTE_WORKGROUP));
            computePass.end();
        }

        const backPass = enc.beginRenderPass({
            colorAttachments: [{
                view:       context.getCurrentTexture().createView(),
                loadOp:     'clear',
                storeOp:    'store',
                clearValue: { r: 0, g: 0, b: 0, a: 1 },
            }],
            depthStencilAttachment: {
                view: this.depthView!,
                depthClearValue: 1,
                depthLoadOp: 'clear',
                depthStoreOp: 'store',
            },
        });

        backPass.setPipeline(renderPipeline);
        backPass.setBindGroup(0, this.renderBindGroup);
        backPass.draw(6);

        for (const cb of getOverlayCallbacks()) {
            cb(backPass, device, time, dt, this.canvas.width, this.canvas.height, this.depthView!);
        }

        if (particlesEnabled) {
            backPass.setViewport(0, 0, this.canvas.width, this.canvas.height, 0, 1);
            backPass.setScissorRect(0, 0, this.canvas.width, this.canvas.height);
            backPass.setPipeline(particlePipeline);
            backPass.setBindGroup(0, this.renderBindGroup);
            backPass.draw(6, NUM_PARTICLES);
        }

        backPass.end();

        device.queue.submit([enc.finish()]);

        const captureResolve = consumeCaptureRequest();
        if (captureResolve) {
            captureResolve(captureFrame(this.canvas));
        }

        this.animId = requestAnimationFrame(this.frame);
    };
}
