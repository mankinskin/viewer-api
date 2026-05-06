/// <reference types="@webgpu/types" />
/**
 * One-time GPU resource creation for Graph3DView edge and grid rendering.
 *
 * All pipelines share a camera + palette bind-group layout so that a
 * single `setBindGroup(0, ...)` call covers both draw calls.
 */
import { QUAD_VERTS, EDGE_INSTANCE_FLOATS, GRID_LINE_FLOATS, GRID_EXTENT, GRID_STEP } from './constants';
import paletteWgsl from '../../../effects/palette.wgsl?raw';
import shaderSource from '../graph3d.wgsl?raw';
import { PALETTE_BYTE_SIZE } from '../../../effects/palette';

// ── Types ──

export interface GpuResources {
    device: GPUDevice;
    quadVB: GPUBuffer;
    camUB: GPUBuffer;
    paletteUB: GPUBuffer;
    camBG: GPUBindGroup;
    edgePipeline: GPURenderPipeline;
    gridPipeline: GPURenderPipeline;
    edgeIB: GPUBuffer;
    gridIB: GPUBuffer;
    gridCount: number;
    edgeDataBuf: Float32Array;
    maxEdges: number;
}

// ── Pipeline creation ──

export function createGpuResources(
    device: GPUDevice,
    format: GPUTextureFormat,
    maxEdges: number,
): GpuResources {
    const fullShader = paletteWgsl + '\n' + shaderSource;
    const shader = device.createShaderModule({ code: fullShader });

    // Quad vertex buffer (shared by edge + grid pipelines)
    const quadVB = device.createBuffer({
        size: QUAD_VERTS.byteLength,
        usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
    });
    device.queue.writeBuffer(quadVB, 0, QUAD_VERTS);

    // Camera + palette uniforms
    const camUB = device.createBuffer({
        size: 128,
        usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
    });
    const paletteUB = device.createBuffer({
        size: PALETTE_BYTE_SIZE,
        usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
    });

    const camBGL = device.createBindGroupLayout({
        entries: [
            {
                binding: 0,
                visibility: GPUShaderStage.VERTEX | GPUShaderStage.FRAGMENT,
                buffer: { type: 'uniform' },
            },
            { binding: 1, visibility: GPUShaderStage.FRAGMENT, buffer: { type: 'uniform' } },
        ],
    });
    const camBG = device.createBindGroup({
        layout: camBGL,
        entries: [
            { binding: 0, resource: { buffer: camUB } },
            { binding: 1, resource: { buffer: paletteUB } },
        ],
    });
    const pipelineLayout = device.createPipelineLayout({ bindGroupLayouts: [camBGL] });

    // Vertex buffer layouts (shared by edge + grid)
    const edgeVertexBuffers: GPUVertexBufferLayout[] = [
        {
            arrayStride: 8,
            stepMode: 'vertex',
            attributes: [{ shaderLocation: 0, offset: 0, format: 'float32x2' as GPUVertexFormat }],
        },
        {
            arrayStride: EDGE_INSTANCE_FLOATS * 4,
            stepMode: 'instance',
            attributes: [
                { shaderLocation: 6, offset: 0, format: 'float32x3' as GPUVertexFormat },
                { shaderLocation: 7, offset: 12, format: 'float32x3' as GPUVertexFormat },
                { shaderLocation: 8, offset: 24, format: 'float32x4' as GPUVertexFormat },
                { shaderLocation: 9, offset: 40, format: 'float32' as GPUVertexFormat },
                { shaderLocation: 10, offset: 44, format: 'float32' as GPUVertexFormat },
            ],
        },
    ];
    const edgeBlend: GPUBlendState = {
        color: { srcFactor: 'src-alpha', dstFactor: 'one-minus-src-alpha', operation: 'add' },
        alpha: { srcFactor: 'one', dstFactor: 'one-minus-src-alpha', operation: 'add' },
    };

    const edgePipeline = device.createRenderPipeline({
        layout: pipelineLayout,
        vertex: { module: shader, entryPoint: 'vs_edge', buffers: edgeVertexBuffers },
        fragment: { module: shader, entryPoint: 'fs_edge', targets: [{ format, blend: edgeBlend }] },
        primitive: { topology: 'triangle-list' },
        depthStencil: { format: 'depth24plus', depthWriteEnabled: false, depthCompare: 'always' },
    });

    const gridPipeline = device.createRenderPipeline({
        layout: pipelineLayout,
        vertex: { module: shader, entryPoint: 'vs_edge', buffers: edgeVertexBuffers },
        fragment: { module: shader, entryPoint: 'fs_edge', targets: [{ format, blend: edgeBlend }] },
        primitive: { topology: 'triangle-list' },
        depthStencil: { format: 'depth24plus', depthWriteEnabled: false, depthCompare: 'always' },
    });

    // Edge instance buffer
    const edgeIB = device.createBuffer({
        size: Math.max(maxEdges * EDGE_INSTANCE_FLOATS * 4, 48),
        usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
    });

    // Grid instance buffer
    const { gridData, gridCount } = buildGridData();
    const gridIB = device.createBuffer({
        size: gridData.byteLength,
        usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
    });
    device.queue.writeBuffer(gridIB, 0, gridData.buffer, gridData.byteOffset, gridData.byteLength);

    // Pre-allocated edge data CPU buffer
    const edgeDataBuf = new Float32Array(Math.max(maxEdges * EDGE_INSTANCE_FLOATS, 1));

    return {
        device,
        quadVB,
        camUB,
        paletteUB,
        camBG,
        edgePipeline,
        gridPipeline,
        edgeIB,
        gridIB,
        gridCount,
        edgeDataBuf,
        maxEdges,
    };
}

/** Build the static grid line instance data. */
function buildGridData(): { gridData: Float32Array; gridCount: number } {
    const gridLines: number[] = [];
    for (let i = -GRID_EXTENT; i <= GRID_EXTENT; i += GRID_STEP) {
        gridLines.push(i, 0, -GRID_EXTENT, i, 0, GRID_EXTENT, 0.25, 0.22, 0.18, 0.06, 0, 0);
        gridLines.push(-GRID_EXTENT, 0, i, GRID_EXTENT, 0, i, 0.25, 0.22, 0.18, 0.06, 0, 0);
    }
    // Axis lines
    gridLines.push(-GRID_EXTENT, 0, 0, GRID_EXTENT, 0, 0, 0.55, 0.25, 0.15, 0.12, 0, 0); // X red
    gridLines.push(0, 0, -GRID_EXTENT, 0, 0, GRID_EXTENT, 0.15, 0.25, 0.55, 0.12, 0, 0); // Z blue
    const gridData = new Float32Array(gridLines);
    const gridCount = gridLines.length / GRID_LINE_FLOATS;
    return { gridData, gridCount };
}

/** Destroy all GPU resources. */
export function destroyGpuResources(res: GpuResources): void {
    res.quadVB.destroy();
    res.camUB.destroy();
    res.paletteUB.destroy();
    res.edgeIB.destroy();
    res.gridIB.destroy();
}
