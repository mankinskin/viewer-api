/**
 * DOM node layer for Graph3DView.
 *
 * Renders each graph node as a positioned div with a label. The actual 3D
 * transforms are applied every frame by `positionDOMNodes` via CSS.
 */
import type { InternalNode } from '../types';
import { nodeSizeClass } from '../utils/nodeStyles';

interface NodeLayerProps {
    nodes: InternalNode[];
    maxSize: number;
}

export function NodeLayer({ nodes, maxSize }: NodeLayerProps) {
    return (
        <>
            {nodes.map((n) => (
                <div
                    key={n.index}
                    class={`g3d-node ${nodeSizeClass(n.size ?? 1, maxSize)}`}
                    data-node-idx={String(n.index)}
                    style={{ position: 'absolute', top: 0, left: 0, pointerEvents: 'none', willChange: 'transform, opacity' }}
                >
                    <span class="g3d-node-label">{n.label}</span>
                </div>
            ))}
        </>
    );
}
