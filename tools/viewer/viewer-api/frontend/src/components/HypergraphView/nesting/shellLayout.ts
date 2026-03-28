/**
 * shellLayout — computes nesting shell positions for parent hierarchy.
 *
 * The selected node sits at the center. Each parent level forms a larger
 * container shell drawn behind the previous level (Russian-doll nesting).
 * When multiple parents exist at the same level, they are spread horizontally.
 */
import type { GraphLayout } from '../layout';
import type { ShellNode } from '../types';

/** Padding constants (pixels at scale 1). */
const BASE_PADDING = 40;
const LEVEL_PADDING = 20;

/**
 * Walk parent hierarchy from `centerIdx` outward, producing one ShellNode
 * per parent. Each shell's size wraps around the content of the previous level.
 */
export function computeShellLayout(
    layout: GraphLayout,
    centerIdx: number,
    parentDepth: number,
    selectedNodeWidth: number,
    selectedNodeHeight: number,
): ShellNode[] {
    const shells: ShellNode[] = [];
    const visited = new Set<number>([centerIdx]);

    let contentWidth = selectedNodeWidth;
    let contentHeight = selectedNodeHeight;
    let currentLevel = [centerIdx];

    for (let level = 1; level <= parentDepth; level++) {
        const nextLevel: number[] = [];
        for (const idx of currentLevel) {
            const node = layout.nodeMap.get(idx);
            if (!node) continue;
            for (const parentIdx of node.parentIndices) {
                if (visited.has(parentIdx)) continue;
                visited.add(parentIdx);
                nextLevel.push(parentIdx);
            }
        }
        if (nextLevel.length === 0) break;

        const padding = BASE_PADDING + level * LEVEL_PADDING;
        const shellWidth = contentWidth + padding * 2;
        const shellHeight = contentHeight + padding * 2;

        // Spread siblings horizontally
        const count = nextLevel.length;
        for (let i = 0; i < count; i++) {
            const offsetX = count > 1
                ? (i - (count - 1) / 2) * shellWidth
                : 0;
            shells.push({
                nodeIdx: nextLevel[i]!,
                shellLevel: level,
                width: shellWidth,
                height: shellHeight,
                centerX: offsetX,
                centerY: 0,
            });
        }

        // Next level wraps around all siblings at this level
        contentWidth = shellWidth * count + padding * 2;
        contentHeight = shellHeight;
        currentLevel = nextLevel;
    }

    return shells;
}
