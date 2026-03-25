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
import { getOverlayCallbacks, consumeCaptureRequest, hasCaptureRequest, getOverlayParticleVP, getOverlayParticleInvVP, consumeParticleVPDirty, getOverlayParticleViewport, getOverlayRefDepth, getOverlayWorldScale, getOverlayCameraPos, fxEnabled } from './overlay-api';
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

    // Depth buffer for 3D callbacks
    private depthTexture: GPUTexture | null = null;
    private depthView: GPUTextureView | null = null;
    private depthW = 0;
    private depthH = 0;

    private animId = 0;
    private cancelled = false;
    private prevTime = performance.now() / 1000;
    private readonly startTime = performance.now();

    // Mouse position
    private mx = -9999;
    private my = -9999;

    // Hover tracking — detect impact (new hover start) for metal spark burst
    private prevHoverIdx = -1;
    private hoverStartTime = 0;

    // Dirty tracking — skip GPU submission when scene is static
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

    // --- Bind group builders -----------------------------------------------

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

    // --- Public API --------------------------------------------------------

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

    // --- Frame -------------------------------------------------------------

    private frame = (): void => {
        if (this.cancelled) return;

        const scanner = this.scanner;

        // Let the scanner update rect measurements for stale/visible elements
        const dataChanged = scanner.updateFrame();

        // Note: No particle reset on full rescan — particles now have view_id
        // and are automatically hidden when not matching the current view.

        // Consume accumulated scroll delta for this frame
        const scrollDelta = scanner.consumeScrollDelta();

        const nowSec = performance.now() / 1000;
        const time = (performance.now() - this.startTime) / 1000;
        const dt   = Math.min(nowSec - this.prevTime, 0.05);
        this.prevTime = nowSec;

        const count = scanner.count;
        const data  = scanner.data;
        const mx = this.mx;
        const my = this.my;

        // --- Hover detection -----------------------------------------------
        // When multiple elements overlap at the mouse position, prefer the
        // one with the highest kind value.  This ensures effect-preview
        // containers (kind 8–11) win over structural parents (kind 0) that
        // also contain the cursor.
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

        // --- Selected element detection ------------------------------------
        // When multiple elements have KIND_SELECTED (e.g. an "always on"
        // demo element plus a user-toggled one), prefer the one that is
        // also being hovered — that's the element the user is interacting
        // with.  Otherwise fall back to the last match so newly-toggled
        // elements (later in DOM order) win over permanent ones.
        let selectedIdx = -1;
        for (let i = 0; i < count; i++) {
            const base = i * ELEM_FLOATS;
            const k = data[base + 5]!;
            if (k === KIND_SELECTED) {
                selectedIdx = i;
                if (i === hoverIdx) break; // hovered selected element wins
            } else if (k === KIND_PANIC && selectedIdx < 0) {
                selectedIdx = i;
            }
        }

        // --- Frame skip: avoid GPU work when scene is static ---------------
        const eff = this.schema.effectSettings.value;
        const fx = fxEnabled.value;
        const particlesEnabled = fx && (eff.sparksEnabled || eff.embersEnabled
            || eff.beamsEnabled || eff.glitterEnabled);

        // "Minimal background" mode — smoke and grain both disabled (or FX off)
        const minimalBackground = !fx || ((!eff.smokeEnabled || eff.smokeIntensity === 0)
            && eff.grainIntensity === 0);

        // Time-dependent effects that require continuous rendering
        const anyAnimated = fx && (
            (eff.smokeEnabled && eff.smokeIntensity > 0)
            || (eff.crtEnabled && eff.crtFlicker > 0)
            || (hoverIdx >= 0 && (eff.cinderEnabled || particlesEnabled))
            || (selectedIdx >= 0 && eff.beamsEnabled));

        // Input changes since last rendered frame
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

        // Always render when overlay callbacks are registered (3D views need
        // continuous frames for camera movement even when FX is off).
        if (!anyAnimated && !inputDirty && !hasOverlayCallbacks) {
            this.animId = requestAnimationFrame(this.frame);
            return;
        }

        // Frame rate limiter: skip every other frame when background is minimal
        // and only particles are animating. This halves GPU load.
        const particlesOnlyAnimated = anyAnimated && minimalBackground
            && !(eff.smokeEnabled && eff.smokeIntensity > 0)
            && !(eff.crtEnabled && eff.crtFlicker > 0)
            && true;  // cursor removed
        if (particlesOnlyAnimated && !inputDirty) {
            this.frameSkipCounter = (this.frameSkipCounter + 1) % 2;
            if (this.frameSkipCounter !== 0) {
                this.animId = requestAnimationFrame(this.frame);
                return;
            }
        } else {
            this.frameSkipCounter = 0;
        }

        // Track state for next frame's dirty check
        this.prevRenderMx = mx;
        this.prevRenderMy = my;
        this.prevCanvasW = this.canvas.width;
        this.prevCanvasH = this.canvas.height;
        this.prevSelectedIdx = selectedIdx;
        this.prevEffects = eff;

        // Zero-fill particle ranges for types that just got disabled
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

        // --- Pack uniforms -------------------------------------------------
        const u = this.buffers.uniformF32;
        u[0] = time;
        u[1] = this.canvas.width;
        u[2] = this.canvas.height;
        u[3] = fx ? count : 0;  // zero element count when FX off — disables background effects
        u[4] = fx ? mx : -9999;
        u[5] = fx ? my : -9999;
        u[6] = dt;
        u[7] = fx ? hoverIdx : -1;
        u[8] = this.hoverStartTime;
        u[9] = fx ? selectedIdx : -1;
        const crtOn = fx && eff.crtEnabled;
        u[10] = crtOn ? eff.crtScanlinesH / 100 : 0.0;
        u[11] = crtOn ? eff.crtScanlinesV / 100 : 0.0;
        u[12] = crtOn ? eff.crtEdgeShadow / 100 : 0.0;
        u[13] = crtOn ? eff.crtFlicker / 100 : 0.0;
        u[14] = crtOn ? eff.crtLineWidth / 100 : 0.0;  // scanline width/thickness
        u[15] = (fx && eff.smokeEnabled) ? eff.smokeIntensity / 100 : 0.0;
        u[16] = (fx && eff.smokeEnabled) ? eff.smokeSpeed / 100 : 0.0;
        u[17] = (fx && eff.smokeEnabled) ? eff.smokeWarmScale / 100 : 0.0;
        u[18] = (fx && eff.smokeEnabled) ? eff.smokeCoolScale / 100 : 0.0;
        u[19] = (fx && eff.smokeEnabled) ? eff.smokeMossScale / 100 : 0.0;
        u[20] = fx ? eff.grainIntensity / 100 : 0.0;
        u[21] = fx ? eff.grainCoarseness / 100 : 0.0;
        u[22] = fx ? eff.grainSize / 100 : 0.0;
        u[23] = fx ? eff.vignetteStrength / 100 : 0.0;
        u[24] = fx ? eff.underglowStrength / 100 : 0.0;
        u[25] = (fx && eff.sparksEnabled) ? eff.sparkSpeed / 100 : 0.0;
        u[26] = (fx && eff.embersEnabled) ? eff.emberSpeed / 100 : 0.0;
        u[27] = (fx && eff.beamsEnabled) ? eff.beamSpeed / 100 : 0.0;
        u[28] = (fx && eff.glitterEnabled) ? eff.glitterSpeed / 100 : 0.0;
        u[29] = fx ? eff.beamHeight : 0;
        u[30] = fx ? eff.beamCount : 0;
        u[31] = fx ? eff.beamDrift / 100 : 0;
        // Scroll delta (no longer merged with camera delta — 3D views don't need it)
        u[32] = scrollDelta.dx;
        u[33] = scrollDelta.dy;
        u[34] = (fx && eff.sparksEnabled) ? eff.sparkCount / 100 : 0.0;
        u[35] = (fx && eff.sparksEnabled) ? eff.sparkSize / 100 : 0.0;
        u[36] = (fx && eff.embersEnabled) ? eff.emberCount / 100 : 0.0;
        u[37] = (fx && eff.embersEnabled) ? eff.emberSize / 100 : 0.0;
        u[38] = (fx && eff.glitterEnabled) ? eff.glitterCount / 100 : 0.0;
        u[39] = (fx && eff.glitterEnabled) ? eff.glitterSize / 100 : 0.0;
        u[40] = (fx && eff.cinderEnabled) ? eff.cinderSize / 100 : 0.0;

        // CRT scanline color (u[48-50]) + padding (u[51])
        const crtCol = eff.crtColor ?? [100, 80, 60];
        u[48] = crtOn ? crtCol[0] / 255 : 0.0;
        u[49] = crtOn ? crtCol[1] / 255 : 0.0;
        u[50] = crtOn ? crtCol[2] / 255 : 0.0;
        u[51] = 0;  // padding

        // Particle projection — 3D views set these each frame in their
        // overlay callbacks (1-frame latency, imperceptible).
        const is3DView = this.schema.isActive3DView();
        const W = this.canvas.width;
        const H = this.canvas.height;
        if (is3DView && consumeParticleVPDirty()) {
            // 3D view provided a custom viewProj — use its values
            const vp3d = getOverlayParticleViewport();
            u[41] = getOverlayRefDepth();
            u[42] = getOverlayWorldScale();
            u[43] = vp3d[0];  // vp_x
            u[44] = vp3d[1];  // vp_y
            u[45] = vp3d[2];  // vp_w
            u[46] = vp3d[3];  // vp_h
            // Camera position for skybox rendering
            const camPos = getOverlayCameraPos();
            u[52] = camPos[0];  // camera_pos_x
            u[53] = camPos[1];  // camera_pos_y
            u[54] = camPos[2];  // camera_pos_z
            u[55] = 0;          // padding
            // Copy particle_vp and particle_inv_vp (mat4 × 2, 32 floats)
            u.set(getOverlayParticleVP(), 56);
            u.set(getOverlayParticleInvVP(), 72);
        } else if (!is3DView) {
            // 2D views: orthographic screen→clip matrices
            u[41] = 0;     // ref_depth = 0 (particles at z=0)
            u[42] = 1;     // world_scale = 1 (world ≡ screen pixels)
            u[43] = 0;     // vp_x
            u[44] = 0;     // vp_y
            u[45] = W;     // vp_w
            u[46] = H;     // vp_h
            // Camera position (not used in 2D, set to 0)
            u[52] = 0; u[53] = 0; u[54] = 0; u[55] = 0;
            // Ortho viewProj: screen pixels → clip [-1,1]
            // ndc_x = x * 2/W - 1,  ndc_y = -y * 2/H + 1,  ndc_z = z
            // Column-major mat4:
            u[56] = 2 / W; u[57] = 0; u[58] = 0; u[59] = 0;   // col 0
            u[60] = 0; u[61] = -2 / H; u[62] = 0; u[63] = 0;   // col 1
            u[64] = 0; u[65] = 0; u[66] = 1; u[67] = 0;   // col 2
            u[68] = -1; u[69] = 1; u[70] = 0; u[71] = 1;   // col 3
            // Inverse ortho: clip → screen pixels
            u[72] = W / 2; u[73] = 0; u[74] = 0; u[75] = 0;  // col 0
            u[76] = 0; u[77] = -H / 2; u[78] = 0; u[79] = 0;  // col 1
            u[80] = 0; u[81] = 0; u[82] = 1; u[83] = 0;  // col 2
            u[84] = W / 2; u[85] = H / 2; u[86] = 0; u[87] = 1;  // col 3
        }
        // else: is3DView but no dirty update this frame — keep previous values

        u[47] = this.schema.getCurrentViewId();  // current_view for per-view particle filtering
        this.buffers.uploadUniforms();

        // Upload palette
        this.buffers.uploadPalette(this.schema.themeColors.value);

        // Upload element data (may grow the buffer)
        this.buffers.uploadElements(data, count);

        // Rebuild bind groups if buffer was reallocated
        this.rebuildBindGroupsIfNeeded();

        // --- GPU encoding --------------------------------------------------
        const { device, context, computePipeline, renderPipeline, particlePipeline } = this.pipelines;
        const enc = device.createCommandEncoder();

        // Ensure depth buffer for 3D callbacks
        this.ensureDepth(this.canvas.width, this.canvas.height);

        // Compute pass: simulate particles (skip when all particle types disabled)
        if (particlesEnabled) {
            const computePass = enc.beginComputePass();
            computePass.setPipeline(computePipeline);
            computePass.setBindGroup(0, this.computeBindGroup);
            computePass.dispatchWorkgroups(Math.ceil(NUM_PARTICLES / COMPUTE_WORKGROUP));
            computePass.end();
        }

        // ── Back render pass (z-index -1, BEHIND DOM) ──
        // Background effects, element rect decorations, 3D overlays (edges, grids, cubes).
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

        // Draw 1: full-screen quad (aurora + elements + CRT)
        backPass.setPipeline(renderPipeline);
        backPass.setBindGroup(0, this.renderBindGroup);
        backPass.draw(6);

        // Draw 2+: external overlay renderers (edges, grids, cubes — behind DOM)
        for (const cb of getOverlayCallbacks()) {
            cb(backPass, device, time, dt, this.canvas.width, this.canvas.height, this.depthView!);
        }

        // Draw 3: particles (additive, on top of everything in the back canvas).
        // Reset viewport/scissor — overlay callbacks may have changed them.
        if (particlesEnabled) {
            backPass.setViewport(0, 0, W, H, 0, 1);
            backPass.setScissorRect(0, 0, W, H);
            backPass.setPipeline(particlePipeline);
            backPass.setBindGroup(0, this.renderBindGroup);
            backPass.draw(6, NUM_PARTICLES);
        }

        backPass.end();

        device.queue.submit([enc.finish()]);

        // --- One-shot capture ---
        const captureResolve = consumeCaptureRequest();
        if (captureResolve) {
            captureResolve(captureFrame(this.canvas));
        }

        this.animId = requestAnimationFrame(this.frame);
    };
}
