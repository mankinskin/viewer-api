/**
 * Rendering constants for the Graph3DView WebGPU overlay.
 */

// ── Geometry ──

export const QUAD_VERTS = new Float32Array([-1, -1, 1, -1, 1, 1, -1, -1, 1, 1, -1, 1]);

/** Number of float32 values per edge instance in the GPU vertex buffer. */
export const EDGE_INSTANCE_FLOATS = 12;
/** Number of float32 values per grid line instance. */
export const GRID_LINE_FLOATS = 12;

// ── Default edge colors ──

/** Normal (unselected) edge color. */
export const NORMAL_EDGE_COLOR: [number, number, number] = [0.45, 0.55, 0.7];
/** Connected edge color (adjacent to selected node). */
export const CONNECTED_EDGE_COLOR: [number, number, number] = [0.3, 0.7, 0.9];

// ── Grid parameters ──

export const GRID_EXTENT = 20;
export const GRID_STEP = 2;
