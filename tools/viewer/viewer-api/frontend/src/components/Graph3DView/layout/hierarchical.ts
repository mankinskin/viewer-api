/**
 * Hierarchical BFS-depth layered layout for Graph3DView.
 *
 * Assigns depth via BFS from all source nodes (nodes with no incoming edges).
 * Nodes at the same depth are spread evenly in the XZ plane, Y encodes depth.
 * Good for visualising DAGs.
 */
import type { Graph3DNode, Graph3DEdge, InternalNode, InternalEdge, InternalLayout } from '../types';
import { hexToRgba, defaultNodeRgba } from '../utils/nodeStyles';

export function buildHierarchicalLayout(nodes: Graph3DNode[], edges: Graph3DEdge[]): InternalLayout {
    const nodeIdMap = new Map<string, number>();
    nodes.forEach((n, i) => nodeIdMap.set(n.id, i));

    // Build adjacency
    const outgoing = new Map<number, number[]>();
    const inDegree = new Map<number, number>();
    for (let i = 0; i < nodes.length; i++) {
        outgoing.set(i, []);
        inDegree.set(i, 0);
    }

    const edgeSet = new Set<string>();
    const internalEdges: InternalEdge[] = [];
    const sourceMap = new Map<number, number[]>();
    const targetMap = new Map<number, number[]>();

    for (const e of edges) {
        const fi = nodeIdMap.get(e.source);
        const ti = nodeIdMap.get(e.target);
        if (fi == null || ti == null) continue;
        const key = `${fi}:${ti}`;
        if (edgeSet.has(key)) continue;
        edgeSet.add(key);
        internalEdges.push({ from: fi, to: ti, color: e.color, style: e.style });
        outgoing.get(fi)!.push(ti);
        inDegree.set(ti, (inDegree.get(ti) ?? 0) + 1);
        if (!sourceMap.has(fi)) sourceMap.set(fi, []);
        if (!targetMap.has(ti)) targetMap.set(ti, []);
        sourceMap.get(fi)!.push(ti);
        targetMap.get(ti)!.push(fi);
    }

    // BFS from roots (nodes with no incoming edges)
    const depth = new Map<number, number>();
    const queue: number[] = [];
    for (let i = 0; i < nodes.length; i++) {
        if ((inDegree.get(i) ?? 0) === 0) {
            depth.set(i, 0);
            queue.push(i);
        }
    }
    // Fallback: if there are no roots (cycle), start from node 0
    if (queue.length === 0 && nodes.length > 0) {
        depth.set(0, 0);
        queue.push(0);
    }

    let qi = 0;
    while (qi < queue.length) {
        const cur = queue[qi++]!;
        const d = depth.get(cur) ?? 0;
        for (const next of (outgoing.get(cur) ?? [])) {
            if (!depth.has(next)) {
                depth.set(next, d + 1);
                queue.push(next);
            }
        }
    }

    // Assign default depth 0 to unreachable nodes
    for (let i = 0; i < nodes.length; i++) {
        if (!depth.has(i)) depth.set(i, 0);
    }

    // Group nodes by depth layer
    const layers = new Map<number, number[]>();
    for (let i = 0; i < nodes.length; i++) {
        const d = depth.get(i)!;
        if (!layers.has(d)) layers.set(d, []);
        layers.get(d)!.push(i);
    }

    const maxDepth = Math.max(...layers.keys(), 0);
    const LAYER_Y_GAP = 2.5;
    const NODE_SPREAD = 2.0;

    // Compute positions
    const positions = new Map<number, [number, number, number]>();
    for (const [d, layerNodes] of layers) {
        const y = (maxDepth - d) * LAYER_Y_GAP; // top nodes have higher Y
        const count = layerNodes.length;
        const totalWidth = (count - 1) * NODE_SPREAD;
        layerNodes.forEach((idx, slot) => {
            const x = -totalWidth / 2 + slot * NODE_SPREAD;
            // Slight Z-offset by depth to add 3D depth cue
            const z = (d - maxDepth / 2) * 0.5;
            positions.set(idx, [x, y, z]);
        });
    }

    const n = nodes.length;
    const internalNodes: InternalNode[] = nodes.map((node, i) => {
        const [x, y, z] = positions.get(i) ?? [0, 0, 0];
        return {
            index: i,
            id: node.id,
            label: node.label,
            color: node.color,
            size: node.size,
            data: node.data,
            x, y, z,
            tx: x, ty: y, tz: z,
            vx: 0, vy: 0, vz: 0,
            radius: 0.4 + Math.min((node.size ?? 1) * 0.1, 0.3),
            rgba: node.color ? hexToRgba(node.color) : defaultNodeRgba(i, n),
            sourceIndices: sourceMap.get(i) ?? [],
            targetIndices: targetMap.get(i) ?? [],
        };
    });

    const nodeMap = new Map<number, InternalNode>();
    for (const nd of internalNodes) nodeMap.set(nd.index, nd);

    let cx = 0, cy = 0, cz = 0;
    for (const nd of internalNodes) { cx += nd.x; cy += nd.y; cz += nd.z; }
    const center: [number, number, number] = n > 0
        ? [cx / n, cy / n, cz / n]
        : [0, 0, 0];

    return { nodes: internalNodes, nodeMap, nodeIdMap, edges: internalEdges, center };
}
