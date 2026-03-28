/**
 * duplicateManager — creates and tracks duplicate nodes for nesting view.
 *
 * In duplicate mode, children of an expanded parent appear both at their
 * original graph position AND inside the parent's row layout. This module
 * creates the DuplicateNode descriptors and handles viewport culling so
 * off-screen duplicates are excluded from the render list entirely.
 */
import type { GraphLayout } from '../layout';
import type { DuplicateNode } from '../types';

/**
 * Build the list of DuplicateNodes for a given expanded parent.
 * One duplicate per direct child, in order of their childIndices.
 */
export function buildDuplicates(
    layout: GraphLayout,
    expandedIdx: number,
    childDepth: number,
): DuplicateNode[] {
    const node = layout.nodeMap.get(expandedIdx);
    if (!node) return [];

    const duplicates: DuplicateNode[] = [];
    const queue: { idx: number; parentIdx: number; depth: number }[] =
        node.childIndices.map(ci => ({ idx: ci, parentIdx: expandedIdx, depth: 1 }));

    let slot = 0;
    while (queue.length > 0) {
        const item = queue.shift()!;
        if (item.depth > childDepth) continue;

        duplicates.push({
            originalIdx: item.idx,
            duplicateId: `dup-${item.parentIdx}-${item.idx}`,
            parentIdx: item.parentIdx,
            slotIndex: slot++,
        });

        // If depth allows, also include grandchildren
        if (item.depth < childDepth) {
            const child = layout.nodeMap.get(item.idx);
            if (child && !child.isAtom) {
                for (const grandchildIdx of child.childIndices) {
                    queue.push({ idx: grandchildIdx, parentIdx: item.idx, depth: item.depth + 1 });
                }
            }
        }
    }
    return duplicates;
}

export interface DuplicatePosition {
    x: number;
    y: number;
}

/**
 * Filter duplicates to only those whose position falls within the
 * viewport (plus margin). Returns the set of visible duplicateIds.
 */
export function cullDuplicates(
    duplicates: DuplicateNode[],
    positions: Map<string, DuplicatePosition>,
    viewportWidth: number,
    viewportHeight: number,
    margin = 100,
): Set<string> {
    const visible = new Set<string>();
    for (const dup of duplicates) {
        const pos = positions.get(dup.duplicateId);
        if (!pos) continue;
        if (pos.x >= -margin && pos.x <= viewportWidth + margin &&
            pos.y >= -margin && pos.y <= viewportHeight + margin) {
            visible.add(dup.duplicateId);
        }
    }
    return visible;
}
