/**
 * Rendering constants and color palettes for the hypergraph WebGPU overlay.
 */

// ── Geometry ──

export const QUAD_VERTS = new Float32Array([-1, -1, 1, -1, 1, 1, -1, -1, 1, 1, -1, 1]);
export const EDGE_INSTANCE_FLOATS = 12;
export const GRID_LINE_FLOATS = 12;

// ── Edge color palettes ──

/** Per-pattern base colors for regular edges. */
export const PATTERN_COLORS: [number, number, number][] = [
    [0.45, 0.55, 0.7],
    [0.7, 0.45, 0.55],
    [0.5, 0.7, 0.45],
    [0.65, 0.55, 0.7],
    [0.7, 0.65, 0.4],
    [0.4, 0.7, 0.65],
];

/** Legacy trace_path edge color. */
export const PATH_EDGE_COLOR: [number, number, number] = [0.1, 0.75, 0.95];

// Search path edge colors (VizPathGraph-based, more precise than trace_path pairs)
// Start & end paths share a uniform teal – arrows in the shader distinguish direction
export const SP_PATH_EDGE_COLOR: [number, number, number] = [0.25, 0.75, 1.0];
export const SP_ROOT_EDGE_COLOR: [number, number, number] = [1.0, 0.85, 0.3];
export const CANDIDATE_EDGE_COLOR: [number, number, number] = [0.55, 0.4, 0.8];

// Parent/child edge colors for selection-mode highlighting
export const PARENT_EDGE_COLOR: [number, number, number] = [0.95, 0.65, 0.2];
export const CHILD_EDGE_COLOR: [number, number, number] = [0.3, 0.7, 0.9];

// Insert-specific edge colors
export const INSERT_EDGE_COLOR: [number, number, number] = [1.0, 0.55, 0.2];
export const INSERT_JOIN_EDGE_COLOR: [number, number, number] = [0.5, 0.85, 0.5];

// ── Grid parameters ──

export const GRID_EXTENT = 20;
export const GRID_STEP = 2;

// ── Decomposition row colours ──

export const ROW_COLORS = [
    'rgba(80, 140, 200, 0.12)',
    'rgba(200, 120, 80, 0.12)',
    'rgba(100, 180, 100, 0.12)',
    'rgba(160, 120, 200, 0.12)',
    'rgba(200, 180, 80, 0.12)',
    'rgba(80, 200, 180, 0.12)',
];
