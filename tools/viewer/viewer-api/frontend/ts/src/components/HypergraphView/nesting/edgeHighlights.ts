/**
 * edgeHighlights — converts parent↔child edges to node highlights when
 * children are rendered inside a nesting shell.
 *
 * Edges between a parent and its children are hidden when the children are
 * shown inside the parent; instead, both the parent and each nested child
 * receive a subtle glow highlight to indicate the relationship.
 */
import type { GraphLayout } from '../layout';
import type { EdgeHighlight } from '../types';
import { edgePairKey } from '../utils/math';

/**
 * Compute which edges should be hidden (returned as pair-key set) and
 * which nodes should receive a glow highlight instead.
 */
export function computeNestingEdgeHighlights(
    _layout: GraphLayout,
    expandedIdx: number,
    nestedChildIndices: number[],
): { hiddenEdgeKeys: Set<number>; highlights: EdgeHighlight[] } {
    const hiddenEdgeKeys = new Set<number>();
    const highlights: EdgeHighlight[] = [];

    if (nestedChildIndices.length === 0) {
        return { hiddenEdgeKeys, highlights };
    }

    // Parent highlight
    highlights.push({ nodeIdx: expandedIdx, role: 'parent' });

    for (const childIdx of nestedChildIndices) {
        // Hide both directions of the parent↔child edge
        hiddenEdgeKeys.add(edgePairKey(expandedIdx, childIdx));
        hiddenEdgeKeys.add(edgePairKey(childIdx, expandedIdx));

        // Child highlight
        highlights.push({ nodeIdx: childIdx, role: 'child' });
    }

    return { hiddenEdgeKeys, highlights };
}
