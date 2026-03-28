/**
 * Math utilities for 3D hypergraph projection and interaction.
 *
 * 3D projection and ray-sphere functions are canonical in Scene3D/math3d.ts
 * and re-exported here for convenience. Edge key helpers are hypergraph-specific.
 */

// Re-export shared 3D math from the canonical location
export { worldToScreen, worldScaleAtDepth, raySphere } from '../../Scene3D/math3d';

// ── Edge key helpers (hypergraph-specific) ──

/** Encode two node indices into a single numeric key (supports up to 65535 nodes). */
export function edgePairKey(from: number, to: number): number {
    return (from << 16) | to;
}

/** Encode edge identity (from, to, patternIdx) into a single numeric key. */
export function edgeTripleKey(from: number, to: number, patternIdx: number): number {
    return from * 1_000_000 + to * 1_000 + patternIdx;
}
