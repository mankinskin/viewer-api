/**
 * Element kind constants, buffer layout, and particle configuration.
 * Pure data — no DOM access, no GPU dependency, no app-specific selectors.
 *
 * App-specific selectors live in AppSchema (see schemas.ts).
 */

// ---------------------------------------------------------------------------
// Element kind constants — passed to the shader so it can vary the glow
// style per element category.
// ---------------------------------------------------------------------------

export const KIND_STRUCTURAL = 0;
export const KIND_ERROR      = 1;
export const KIND_WARN       = 2;
export const KIND_INFO       = 3;
export const KIND_DEBUG      = 4;
export const KIND_SPAN_HL    = 5;
export const KIND_SELECTED   = 6;
export const KIND_PANIC      = 7;
export const KIND_FX_SPARK   = 8;
export const KIND_FX_EMBER   = 9;
export const KIND_FX_BEAM    = 10;
export const KIND_FX_GLITTER = 11;

/** f32 values per element in the storage buffer: [x, y, w, h, hue, kind, depth, _p2] */
export const ELEM_FLOATS = 8;
export const ELEM_BYTES  = ELEM_FLOATS * 4;  // 32 bytes, 16-byte aligned

/** Number of particles simulated by the compute shader. */
export const NUM_PARTICLES = 640;
/**
 * Floats per particle (48 bytes total).
 * Layout: [pos.x, pos.y, pos.z, life, vel.x, vel.y, vel.z, max_life,
 *          hue, size, kind_view, spawn_t]
 */
export const PARTICLE_FLOATS  = 12;
export const PARTICLE_BYTES   = PARTICLE_FLOATS * 4;  // 48 bytes
export const PARTICLE_BUF_SIZE = NUM_PARTICLES * PARTICLE_BYTES;
export const COMPUTE_WORKGROUP = 64;

/** Particle index ranges per type (must match WGSL constants). */
export const SPARK_START = 0;
export const SPARK_END = 96;
export const EMBER_START = SPARK_END;
export const EMBER_END = 288;
export const RAY_START = EMBER_END;
export const RAY_END = 544;
export const GLITTER_START = RAY_END;
export const GLITTER_END = 640;

// ---------------------------------------------------------------------------
// Selector metadata type (used by ElementScanner)
// ---------------------------------------------------------------------------

/** Pre-computed selector entry used at runtime by the ElementScanner. */
export interface SelectorEntry {
    sel: string;
    hue: number;
    kind: number;
}
