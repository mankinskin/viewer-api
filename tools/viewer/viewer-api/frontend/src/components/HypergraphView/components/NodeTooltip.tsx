/**
 * NodeTooltip - Hover tooltip showing node details.
 */
import type { TooltipData } from '../hooks/useMouseInteraction';

export interface NodeTooltipProps {
    tooltip: TooltipData | null;
}

export function NodeTooltip({ tooltip }: NodeTooltipProps) {
    if (!tooltip) return null;

    return (
        <div class="hypergraph-tooltip" style={{ left: `${tooltip.x}px`, top: `${tooltip.y}px` }}>
            <div class="tt-label">{tooltip.node.label}</div>
            <div class="tt-detail">
                idx={tooltip.node.index} width={tooltip.node.width}{' '}
                {tooltip.node.isAtom ? '(atom)' : `(${tooltip.node.childIndices.length} children)`}
            </div>
        </div>
    );
}
