import type { GraphLayout } from './layout';

/**
 * A single child token within a decomposition pattern row.
 */
export interface DecompositionChild {
    index: number;
    label: string;
    width: number;
    /** Fraction of the parent's width that this child occupies (0..1). */
    fraction: number;
    subIndex: number;
}

/**
 * One decomposition pattern (one way to split the parent into children).
 * All children's widths sum to the parent's width.
 */
export interface DecompositionPattern {
    patternIdx: number;
    children: DecompositionChild[];
}

/**
 * Get all decomposition patterns for a parent node.
 * Each pattern represents a different way the parent's string can be split
 * into smaller substrings. Within each pattern, children are ordered by
 * sub_index and their widths sum to the parent width.
 */
export function getDecompositionPatterns(
    layout: GraphLayout,
    parentIdx: number,
): DecompositionPattern[] {
    const parent = layout.nodeMap.get(parentIdx);
    if (!parent || parent.isAtom) return [];

    const byPattern = new Map<number, { to: number; subIndex: number }[]>();
    for (const edge of layout.edges) {
        if (edge.from === parentIdx) {
            if (!byPattern.has(edge.patternIdx)) {
                byPattern.set(edge.patternIdx, []);
            }
            byPattern.get(edge.patternIdx)!.push({
                to: edge.to,
                subIndex: edge.subIndex,
            });
        }
    }

    const patterns: DecompositionPattern[] = [];
    for (const [patternIdx, edgeList] of [...byPattern.entries()].sort(
        (left, right) => left[0] - right[0],
    )) {
        edgeList.sort((left, right) => left.subIndex - right.subIndex);
        const children: DecompositionChild[] = edgeList.map((edge) => {
            const child = layout.nodeMap.get(edge.to);
            return {
                index: edge.to,
                label: child?.label ?? `#${edge.to}`,
                width: child?.width ?? 1,
                fraction: (child?.width ?? 1) / parent.width,
                subIndex: edge.subIndex,
            };
        });
        patterns.push({ patternIdx, children });
    }

    return patterns;
}