/**
 * NodeLayer - DOM node rendering with visualization state classes.
 *
 * All nodes are rendered as flat siblings. The overlay renderer handles
 * imperatively reparenting child DOM elements into the expanded parent
 * when a compound node is selected, so that edges stay connected.
 *
 * When nesting view is enabled, additional duplicate DOM elements are
 * rendered for children shown inside expanded parents. Originals that
 * have active duplicates are dimmed.
 */
import type { LayoutNode } from '../layout';
import { nodeWidthClass } from '../utils/nodeStyles';
import { getNodeVizClasses, type VisualizationState } from '../hooks/useVisualizationState';
import type { DuplicateNode, ShellNode } from '../types';

export interface NodeLayerProps {
    nodes: LayoutNode[];
    maxWidth: number;
    vizState: VisualizationState;
    /** Duplicate nodes to render inside expanded parents (nesting view). */
    duplicates?: DuplicateNode[];
    /** Set of original node indices that have an active duplicate (rendered dimmed). */
    duplicatedOriginals?: Set<number>;
    /** Shell parent nodes to render as nesting containers. */
    shells?: ShellNode[];
}

export function NodeLayer({ nodes, maxWidth, vizState, duplicates, duplicatedOriginals, shells }: NodeLayerProps) {
    const nodeMap = new Map<number, LayoutNode>();
    for (const n of nodes) nodeMap.set(n.index, n);

    return (
        <>
            {/* Shell containers (rendered first = behind everything) */}
            {shells?.map(shell => {
                const node = nodeMap.get(shell.nodeIdx);
                if (!node) return null;
                return (
                    <div
                        key={`shell-${shell.nodeIdx}`}
                        class="hg-nesting-shell"
                        data-shell-idx={shell.nodeIdx}
                        data-shell-level={shell.shellLevel}
                    >
                        <span class="hg-shell-label">{node.label}</span>
                    </div>
                );
            })}

            {/* Regular nodes */}
            {nodes.map(n => {
                const levelClass = nodeWidthClass(n.width, maxWidth);
                const vizClasses = getNodeVizClasses(n.index, vizState);
                const isDimmedOriginal = duplicatedOriginals?.has(n.index);

                return (
                    <div
                        key={n.index}
                        class={`hg-node ${levelClass} ${n.isAtom ? 'hg-atom' : 'hg-compound'} ${vizClasses}${isDimmedOriginal ? ' hg-has-duplicate' : ''}`}
                        data-node-idx={n.index}
                    >
                        <div class="hg-node-content">
                            <span class={`level-badge ${levelClass}`}>{n.isAtom ? 'ATOM' : `W${n.width}`}</span>
                            <span class="hg-node-label">{n.label}</span>
                            <span class="hg-node-idx">#{n.index}</span>
                        </div>
                    </div>
                );
            })}

            {/* Duplicate nodes (inside expanded parents in nesting view) */}
            {duplicates?.map(dup => {
                const n = nodeMap.get(dup.originalIdx);
                if (!n) return null;
                const levelClass = nodeWidthClass(n.width, maxWidth);
                const vizClasses = getNodeVizClasses(n.index, vizState);

                return (
                    <div
                        key={dup.duplicateId}
                        class={`hg-node hg-duplicate ${levelClass} ${n.isAtom ? 'hg-atom' : 'hg-compound'} ${vizClasses}`}
                        data-node-idx={n.index}
                        data-duplicate-id={dup.duplicateId}
                        data-duplicate-parent={dup.parentIdx}
                    >
                        <div class="hg-node-content">
                            <span class={`level-badge ${levelClass}`}>{n.isAtom ? 'ATOM' : `W${n.width}`}</span>
                            <span class="hg-node-label">{n.label}</span>
                            <span class="hg-node-idx">#{n.index}</span>
                        </div>
                    </div>
                );
            })}
        </>
    );
}
