/**
 * Node lerp animation — smoothly animates node positions toward their targets.
 */
import type { LayoutNode } from '../layout';

/**
 * Advance all nodes toward their target positions using exponential decay.
 *
 * @param nodes — layout nodes to animate (mutated in-place)
 * @param dt — frame delta-time in seconds
 * @param lerpSpeed — exponential decay rate (higher = snappier, default 12)
 * @returns `true` if any node moved more than `epsilon` (default 0.0001)
 */
export function animateNodes(nodes: LayoutNode[], dt: number, lerpSpeed = 12): boolean {
    const lerpFactor = 1 - Math.exp(-lerpSpeed * dt);
    const epsilon = 0.0001;
    let anyMoved = false;
    for (const n of nodes) {
        const dx = (n.tx - n.x) * lerpFactor;
        const dy = (n.ty - n.y) * lerpFactor;
        const dz = (n.tz - n.z) * lerpFactor;
        n.x += dx;
        n.y += dy;
        n.z += dz;
        if (Math.abs(dx) > epsilon || Math.abs(dy) > epsilon || Math.abs(dz) > epsilon) {
            anyMoved = true;
        }
    }
    return anyMoved;
}
