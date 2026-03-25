/**
 * Edge instance buffer builder for Graph3DView.
 *
 * Outputs a packed Float32Array ready to upload to the GPU edge instance buffer.
 * Edge types used: 0 = grid (not here), 1 = normal, flag=1 = connected.
 */
import { EDGE_INSTANCE_FLOATS, NORMAL_EDGE_COLOR, CONNECTED_EDGE_COLOR } from './constants';
import type { InternalLayout } from '../types';
import { hexToRgba } from '../utils/nodeStyles';

/**
 * Fill `edgeDataBuf` with per-edge instance data.
 *
 * @param edgeDataBuf — preallocated Float32Array (maxEdges * EDGE_INSTANCE_FLOATS)
 * @param layout — current internal layout
 * @param selectedIdx — index of selected node (-1 = none)
 * @param connectedSet — set of node indices connected to selected
 * @returns number of edges written
 */
export function buildEdgeInstances(
    edgeDataBuf: Float32Array,
    layout: InternalLayout,
    selectedIdx: number,
    connectedSet: Set<number>,
): number {
    for (let i = 0; i < layout.edges.length; i++) {
        const e = layout.edges[i]!;
        const a = layout.nodeMap.get(e.from);
        const b = layout.nodeMap.get(e.to);
        if (!a || !b) continue;
        const off = i * EDGE_INSTANCE_FLOATS;

        // Positions
        edgeDataBuf[off + 0] = a.x;
        edgeDataBuf[off + 1] = a.y;
        edgeDataBuf[off + 2] = a.z;
        edgeDataBuf[off + 3] = b.x;
        edgeDataBuf[off + 4] = b.y;
        edgeDataBuf[off + 5] = b.z;

        // Color
        const isConnected = selectedIdx >= 0
            && (e.from === selectedIdx || e.to === selectedIdx
                || connectedSet.has(e.from) || connectedSet.has(e.to));
        let r: number, g: number, bl: number, a_: number;
        if (e.color) {
            const rgba = hexToRgba(e.color);
            [r, g, bl, a_] = [rgba[0], rgba[1], rgba[2], isConnected ? 0.95 : 0.6];
        } else {
            const base = isConnected ? CONNECTED_EDGE_COLOR : NORMAL_EDGE_COLOR;
            [r, g, bl] = base;
            a_ = isConnected ? 0.9 : 0.5;
        }
        edgeDataBuf[off + 6] = r;
        edgeDataBuf[off + 7] = g;
        edgeDataBuf[off + 8] = bl;
        edgeDataBuf[off + 9] = a_;

        // flags (highlighted = connected), edgeType = 1 (normal beam)
        edgeDataBuf[off + 10] = isConnected ? 1.0 : 0.0;
        edgeDataBuf[off + 11] = 1.0;
    }
    return layout.edges.length;
}
