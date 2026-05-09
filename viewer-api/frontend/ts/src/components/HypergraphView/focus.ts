import type { VizPathGraph } from "@context-engine/types";

import type { GraphLayout } from "./layout";
import type { NestingSettings } from "./types";

/**
 * In dup=off nesting mode, child nodes are shown as clones inside their
 * expanded parent. When the user selects such a child, the camera should
 * target the parent's 3-D position instead of the hidden original.
 */
export function getCameraFocusIdx(
    layout: GraphLayout,
    selectedIdx: number,
    nesting: NestingSettings,
    searchPath: VizPathGraph | null,
): number {
    if (selectedIdx < 0) return selectedIdx;
    if (!nesting.enabled || nesting.duplicateMode) return selectedIdx;

    const desiredExpanded = new Set<number>();
    desiredExpanded.add(selectedIdx);
    const searchPathRootIdx = searchPath?.root?.index;
    if (searchPathRootIdx != null) desiredExpanded.add(searchPathRootIdx);

    for (const idx of [...desiredExpanded]) {
        if (idx === searchPathRootIdx) continue;
        const node = layout.nodeMap.get(idx);
        if (!node) continue;
        for (const otherIdx of desiredExpanded) {
            if (otherIdx === idx) continue;
            const other = layout.nodeMap.get(otherIdx);
            if (other && other.childIndices.includes(idx)) {
                desiredExpanded.delete(idx);
                break;
            }
        }
    }

    if (desiredExpanded.has(selectedIdx)) return selectedIdx;

    for (const expandedIdx of desiredExpanded) {
        const expandedNode = layout.nodeMap.get(expandedIdx);
        if (expandedNode && expandedNode.childIndices.includes(selectedIdx)) {
            return expandedIdx;
        }
    }

    return selectedIdx;
}