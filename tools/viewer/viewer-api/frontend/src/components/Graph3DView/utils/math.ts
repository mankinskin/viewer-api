/**
 * Math utilities for 3D graph projection and interaction.
 *
 * Shared 3D math is re-exported from the canonical location in viewer-api.
 * Edge key helpers are generic graph utilities.
 */

// Re-export shared 3D math
export { worldToScreen, worldScaleAtDepth, raySphere } from '../../../utils/math3d';

// ── Edge key helpers ──

/** Encode two node indices into a single numeric key (supports up to 65535 nodes). */
export function edgePairKey(from: number, to: number): number {
    return (from << 16) | to;
}
