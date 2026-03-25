/// <reference types="@webgpu/types" />
/**
 * Dynamic GPU buffer management.
 *
 * Owns the four GPU buffers (uniform, elements, particles, palette) and
 * handles dynamic element-buffer resizing.  Exposes a `generation` counter
 * that the render loop checks each frame — when it changes, bind groups
 * must be rebuilt because the elem buffer was reallocated.
 */

import { ELEM_BYTES, NUM_PARTICLES, PARTICLE_FLOATS, PARTICLE_BYTES, PARTICLE_BUF_SIZE } from './element-types';
import { buildPaletteBuffer, PALETTE_BYTE_SIZE } from '../../effects/palette';
import type { ThemeColors } from '../../store/theme';

/** Initial element capacity (doubled on overflow). */
const INITIAL_ELEM_CAPACITY = 128;

/** Pre-allocated zero buffer for particle resets (shared, never mutated). */
const ZERO_PARTICLES = new Float32Array(NUM_PARTICLES * PARTICLE_FLOATS);

export class GpuBufferManager {
    readonly device: GPUDevice;

    // Uniform buffer (fixed 352 bytes = 52 scalars + 4 camera floats + 2 × mat4x4, 16-byte aligned)
    readonly uniformBuffer: GPUBuffer;
    readonly uniformF32 = new Float32Array(88);

    // Element buffer (dynamically resizable)
    private _elemBuffer: GPUBuffer;
    private _elemCapacity: number;

    // Particle buffer (fixed)
    readonly particleBuffer: GPUBuffer;

    // Palette buffer (fixed)
    readonly paletteBuffer: GPUBuffer;

    // Palette caching — avoid rebuilding when theme colors haven't changed
    private _cachedPaletteColors: ThemeColors | null = null;
    private _cachedPaletteBuf: Float32Array | null = null;

    // Generation counter — incremented when elem buffer is reallocated
    private _generation = 0;

    constructor(device: GPUDevice) {
        this.device = device;

        this.uniformBuffer = device.createBuffer({
            size: 352,
            usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
        });

        this._elemCapacity = INITIAL_ELEM_CAPACITY;
        this._elemBuffer = device.createBuffer({
            size:  INITIAL_ELEM_CAPACITY * ELEM_BYTES,
            usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST,
        });

        this.particleBuffer = device.createBuffer({
            size:  PARTICLE_BUF_SIZE,
            usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST,
        });
        // Zero-init (all dead particles) — reuse shared zero buffer
        device.queue.writeBuffer(this.particleBuffer, 0, ZERO_PARTICLES);

        this.paletteBuffer = device.createBuffer({
            size:  PALETTE_BYTE_SIZE,
            usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
        });
    }

    /** Current element buffer (may change on reallocation). */
    get elemBuffer(): GPUBuffer { return this._elemBuffer; }

    /** Current capacity in number of elements. */
    get elemCapacity(): number { return this._elemCapacity; }

    /** Monotonically increasing — changes when elem buffer is reallocated. */
    get generation(): number { return this._generation; }

    /**
     * Ensure element buffer can hold `count` elements.
     * Returns `true` if the buffer was reallocated (bind groups need rebuild).
     */
    ensureElemCapacity(count: number): boolean {
        if (count <= this._elemCapacity) return false;

        const newCap = Math.max(count, this._elemCapacity * 2);
        const oldBuf = this._elemBuffer;
        this._elemBuffer = this.device.createBuffer({
            size:  newCap * ELEM_BYTES,
            usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST,
        });
        this._elemCapacity = newCap;
        this._generation++;
        oldBuf.destroy();
        return true;
    }

    /**
     * Upload element data to GPU.  Grows the buffer if needed.
     * Returns `true` if the buffer was reallocated.
     */
    uploadElements(data: Float32Array, count: number): boolean {
        const grew = this.ensureElemCapacity(count);
        if (count > 0) {
            this.device.queue.writeBuffer(
                this._elemBuffer, 0,
                data.buffer, data.byteOffset, count * ELEM_BYTES,
            );
        }
        return grew;
    }

    /** Upload the uniform buffer from `this.uniformF32`. */
    uploadUniforms(): void {
        this.device.queue.writeBuffer(this.uniformBuffer, 0, this.uniformF32.buffer);
    }

    /** Upload theme palette colors to the GPU. Skips if colors haven't changed. */
    uploadPalette(colors: ThemeColors): void {
        if (colors === this._cachedPaletteColors) return;
        this._cachedPaletteColors = colors;
        this._cachedPaletteBuf = buildPaletteBuffer(colors);
        this.device.queue.writeBuffer(this.paletteBuffer, 0, this._cachedPaletteBuf.buffer);
    }

    /** Zero-fill the particle buffer, killing all live particles instantly. */
    resetParticles(): void {
        this.device.queue.writeBuffer(this.particleBuffer, 0, ZERO_PARTICLES);
    }

    /** Zero-fill a range of particles [startIdx, startIdx + count). */
    resetParticleRange(startIdx: number, count: number): void {
        const offset = startIdx * PARTICLE_BYTES;
        // Use a subarray view of the shared zero buffer (no allocation)
        this.device.queue.writeBuffer(
            this.particleBuffer,
            offset,
            ZERO_PARTICLES.buffer,
            0,
            count * PARTICLE_BYTES,
        );
    }

    /** Destroy all GPU buffers. */
    destroy(): void {
        this.uniformBuffer.destroy();
        this._elemBuffer.destroy();
        this.particleBuffer.destroy();
        this.paletteBuffer.destroy();
    }
}
