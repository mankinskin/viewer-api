/// <reference types="@webgpu/types" />
/**
 * GPU device, pipeline, and shader module factory.
 *
 * Creates the WebGPU adapter/device, concatenates WGSL shader sources,
 * and builds all three pipelines:
 *   - **background** — full-screen quad: smoke, element rects, CRT, grain
 *   - **particle**   — instanced quads: sparks, embers, beams, glitter
 *   - **compute**    — particle physics simulation
 *
 * Everything renders on a single opaque canvas (z-index -1, behind DOM).
 * Particles use additive blending so they glow through the transparent
 * DOM backgrounds without requiring a separate transparent canvas.
 *
 * Does NOT create buffers or bind groups — those belong to GpuBufferManager.
 */

import paletteWgsl from '../../effects/palette.wgsl?raw';
import particleShadingWgsl from '../../effects/particle-shading.wgsl?raw';
import typesCode from './types.wgsl?raw';
import noiseCode from './noise.wgsl?raw';
import bgCode from './background.wgsl?raw';
import particleCode from './particles.wgsl?raw';
import csCode from './compute.wgsl?raw';

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

export interface GpuPipelines {
    device: GPUDevice;
    format: GPUTextureFormat;
    context: GPUCanvasContext;
    renderPipeline: GPURenderPipeline;
    particlePipeline: GPURenderPipeline;
    computePipeline: GPUComputePipeline;
    computeBGL: GPUBindGroupLayout;
    renderBGL: GPUBindGroupLayout;
}

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

/**
 * Request a WebGPU adapter + device, build shader modules and pipelines.
 * Returns `null` if WebGPU is not available or the adapter request fails.
 */
export async function initGpu(
    canvas: HTMLCanvasElement,
): Promise<GpuPipelines | null> {
    if (!('gpu' in navigator)) {
        console.warn('[WgpuOverlay] WebGPU not supported in this browser.');
        return null;
    }

    const adapter = await navigator.gpu.requestAdapter();
    if (!adapter) return null;

    const device = await adapter.requestDevice();

    const ctx    = canvas.getContext('webgpu') as GPUCanvasContext;
    const format = navigator.gpu.getPreferredCanvasFormat();
    ctx.configure({ device, format, alphaMode: 'opaque' });

    // --- Shader modules (concatenated from split files) --------------------
    const sharedCode = paletteWgsl + '\n' + typesCode + '\n' + noiseCode + '\n';
    const renderShared = sharedCode + particleShadingWgsl + '\n';

    const renderShader = device.createShaderModule({
        label: 'background-shader',
        code:  renderShared + bgCode,
    });

    const particleShader = device.createShaderModule({
        label: 'particle-shader',
        code:  renderShared + particleCode,
    });

    const computeShader = device.createShaderModule({
        label: 'compute-shader',
        code:  sharedCode + csCode,
    });

    // --- Bind group layouts ------------------------------------------------
    const computeBGL = device.createBindGroupLayout({
        entries: [
            { binding: 0, visibility: GPUShaderStage.COMPUTE, buffer: { type: 'uniform' } },
            { binding: 1, visibility: GPUShaderStage.COMPUTE, buffer: { type: 'read-only-storage' } },
            { binding: 2, visibility: GPUShaderStage.COMPUTE, buffer: { type: 'storage' } },
            { binding: 3, visibility: GPUShaderStage.COMPUTE, buffer: { type: 'uniform' } },
        ],
    });

    const renderBGL = device.createBindGroupLayout({
        entries: [
            { binding: 0, visibility: GPUShaderStage.VERTEX | GPUShaderStage.FRAGMENT, buffer: { type: 'uniform' } },
            { binding: 1, visibility: GPUShaderStage.VERTEX | GPUShaderStage.FRAGMENT, buffer: { type: 'read-only-storage' } },
            { binding: 2, visibility: GPUShaderStage.VERTEX | GPUShaderStage.FRAGMENT, buffer: { type: 'read-only-storage' } },
            { binding: 3, visibility: GPUShaderStage.FRAGMENT, buffer: { type: 'uniform' } },
        ],
    });

    // --- Pipelines ---------------------------------------------------------
    const computePipeline = device.createComputePipeline({
        layout:  device.createPipelineLayout({ bindGroupLayouts: [computeBGL] }),
        compute: { module: computeShader, entryPoint: 'cs_main' },
    });

    const renderPipelineLayout = device.createPipelineLayout({ bindGroupLayouts: [renderBGL] });

    const renderPipeline = device.createRenderPipeline({
        layout:   renderPipelineLayout,
        vertex:   { module: renderShader, entryPoint: 'vs_main' },
        fragment: {
            module:     renderShader,
            entryPoint: 'fs_main',
            targets: [{ format }],
        },
        primitive: { topology: 'triangle-list' },
        depthStencil: { format: 'depth24plus', depthWriteEnabled: false, depthCompare: 'always' },
    });

    const particlePipeline = device.createRenderPipeline({
        layout:   renderPipelineLayout,
        vertex:   { module: particleShader, entryPoint: 'vs_particle' },
        fragment: {
            module:     particleShader,
            entryPoint: 'fs_particle',
            targets: [{
                format,
                blend: {
                    // Additive blending: particles add light on top of the
                    // opaque background.  No alpha compositing issues since
                    // the back canvas is opaque (alphaMode: 'opaque').
                    color: { srcFactor: 'one', dstFactor: 'one', operation: 'add' },
                    alpha: { srcFactor: 'one', dstFactor: 'one', operation: 'add' },
                },
            }],
        },
        primitive: { topology: 'triangle-list' },
        depthStencil: { format: 'depth24plus', depthWriteEnabled: false, depthCompare: 'always' },
    });

    return {
        device,
        format,
        context: ctx,
        renderPipeline,
        particlePipeline,
        computePipeline,
        computeBGL,
        renderBGL,
    };
}
