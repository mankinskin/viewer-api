/**
 * NodeInfoPanel - Selected node details with clickable parent/child links.
 */
import type { LayoutNode, GraphLayout } from '../layout';
import { getNodeVizStates, type VisualizationState } from '../hooks/useVisualizationState';

export interface NodeInfoPanelProps {
    node: LayoutNode;
    layout: GraphLayout;
    vizState: VisualizationState;
    onFocusNode: (nodeIndex: number) => void;
}

export function NodeInfoPanel({ node, layout, vizState, onFocusNode }: NodeInfoPanelProps) {
    const vizStates = getNodeVizStates(node.index, vizState);

    const handleLinkClick = (idx: number) => {
        const target = layout.nodeMap.get(idx);
        if (target) {
            onFocusNode(idx);
        }
    };

    return (
        <div class="node-info-panel">
            <div class="nip-header">
                <span class="nip-title">{node.label}</span>
                <span class={`nip-badge ${node.isAtom ? 'atom' : 'compound'}`}>
                    {node.isAtom ? 'Atom' : `W${node.width}`}
                </span>
            </div>

            <div class="nip-section">
                <div class="nip-row">
                    <span class="nip-label">Index</span>
                    <span class="nip-value">#{node.index}</span>
                </div>
                <div class="nip-row">
                    <span class="nip-label">Width</span>
                    <span class="nip-value">{node.width}</span>
                </div>
            </div>

            {node.parentIndices.length > 0 && (
                <div class="nip-section">
                    <div class="nip-section-title">Parents ({node.parentIndices.length})</div>
                    <div class="nip-indices">
                        {node.parentIndices.slice(0, 8).map(idx => (
                            <span key={idx} class="nip-idx nip-link" onClick={() => handleLinkClick(idx)}>
                                #{idx}
                            </span>
                        ))}
                        {node.parentIndices.length > 8 && (
                            <span class="nip-idx">+{node.parentIndices.length - 8}</span>
                        )}
                    </div>
                </div>
            )}

            {node.childIndices.length > 0 && (
                <div class="nip-section">
                    <div class="nip-section-title">Children ({node.childIndices.length})</div>
                    <div class="nip-indices">
                        {node.childIndices.slice(0, 8).map(idx => (
                            <span key={idx} class="nip-idx nip-link" onClick={() => handleLinkClick(idx)}>
                                #{idx}
                            </span>
                        ))}
                        {node.childIndices.length > 8 && (
                            <span class="nip-idx">+{node.childIndices.length - 8}</span>
                        )}
                    </div>
                </div>
            )}

            {vizStates.length > 0 && (
                <div class="nip-viz-state">
                    <div class="nip-section-title">Visualization State</div>
                    <div class="nip-indices">
                        {vizStates.map(state => (
                            <span key={state} class={`nip-viz-badge ${state}`}>
                                {state.replace('-', ' ')}
                            </span>
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
}
