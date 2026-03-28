/**
 * GraphInfoOverlay - Node/edge/atom count display.
 */
import type { GraphSnapshot as HypergraphSnapshot } from '@context-engine/types';

export interface GraphInfoOverlayProps {
    snapshot: HypergraphSnapshot;
}

export function GraphInfoOverlay({ snapshot }: GraphInfoOverlayProps) {
    return (
        <div class="hypergraph-info">
            <div class="hg-title">Hypergraph</div>
            <div class="hg-row">
                <span class="hg-label">Nodes</span>
                <span class="hg-value">{snapshot.nodes.length}</span>
            </div>
            <div class="hg-row">
                <span class="hg-label">Edges</span>
                <span class="hg-value">{snapshot.edges.length}</span>
            </div>
            <div class="hg-row">
                <span class="hg-label">Atoms</span>
                <span class="hg-value">{snapshot.nodes.filter(n => n.width === 1).length}</span>
            </div>
        </div>
    );
}
