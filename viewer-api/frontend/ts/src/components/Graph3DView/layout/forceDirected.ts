/**
 * Force-directed 3D layout for Graph3DView.
 *
 * Spring-electrical simulation: global repulsion + per-edge attraction.
 * Positions are computed once at build time; animation lerps toward them
 * each frame via nodeAnimator.
 */
import type { Graph3DNode, Graph3DEdge, InternalNode, InternalEdge, InternalLayout } from '../types';
import { hexToRgba, defaultNodeRgba } from '../utils/nodeStyles';

export function buildForceLayout(nodes: Graph3DNode[], edges: Graph3DEdge[]): InternalLayout {
    const nodeIdMap = new Map<string, number>();
    nodes.forEach((n, i) => nodeIdMap.set(n.id, i));

    // Build adjacency lists
    const sourceMap = new Map<number, number[]>();
    const targetMap = new Map<number, number[]>();
    for (const e of edges) {
        const fi = nodeIdMap.get(e.source);
        const ti = nodeIdMap.get(e.target);
        if (fi == null || ti == null) continue;
        if (!sourceMap.has(fi)) sourceMap.set(fi, []);
        if (!targetMap.has(ti)) targetMap.set(ti, []);
        sourceMap.get(fi)!.push(ti);
        targetMap.get(ti)!.push(fi);
    }

    // Initial positions on a sphere shell
    const n = nodes.length;
    const shellRadius = 0.5 + n * 0.5;
    const internalNodes: InternalNode[] = nodes.map((node, i) => {
        const angle = (i / Math.max(n, 1)) * Math.PI * 2;
        const tilt = ((i * 0.618) % 1) * Math.PI - Math.PI / 2;
        const cosT = Math.cos(tilt);
        return {
            index: i,
            id: node.id,
            label: node.label,
            color: node.color,
            size: node.size,
            data: node.data,
            x: Math.cos(angle) * cosT * shellRadius,
            y: Math.sin(tilt) * shellRadius * 0.5,
            z: Math.sin(angle) * cosT * shellRadius,
            tx: 0, ty: 0, tz: 0,
            vx: 0, vy: 0, vz: 0,
            radius: 0.4 + Math.min((node.size ?? 1) * 0.1, 0.3),
            rgba: node.color ? hexToRgba(node.color) : defaultNodeRgba(i, n),
            sourceIndices: sourceMap.get(i) ?? [],
            targetIndices: targetMap.get(i) ?? [],
        };
    });

    const nodeMap = new Map<number, InternalNode>();
    for (const nd of internalNodes) nodeMap.set(nd.index, nd);

    // Deduplicate edges
    const edgeSet = new Set<string>();
    const internalEdges: InternalEdge[] = [];
    for (const e of edges) {
        const fi = nodeIdMap.get(e.source);
        const ti = nodeIdMap.get(e.target);
        if (fi == null || ti == null) continue;
        const key = `${fi}:${ti}`;
        if (edgeSet.has(key)) continue;
        edgeSet.add(key);
        internalEdges.push({ from: fi, to: ti, color: e.color, style: e.style });
    }

    // Run force simulation to find equilibrium
    simulate(internalNodes, internalEdges);

    // Freeze animation targets at final positions
    for (const nd of internalNodes) {
        nd.tx = nd.x;
        nd.ty = nd.y;
        nd.tz = nd.z;
    }

    return {
        nodes: internalNodes,
        nodeMap,
        nodeIdMap,
        edges: internalEdges,
        center: computeCenter(internalNodes),
    };
}

function simulate(nodes: InternalNode[], edges: InternalEdge[]): void {
    const REPULSION = 8.0;
    const ATTRACTION = 0.04;
    const DAMPING = 0.85;
    const STEPS = 100;

    for (let step = 0; step < STEPS; step++) {
        // Repulsion between all pairs
        for (let i = 0; i < nodes.length; i++) {
            for (let j = i + 1; j < nodes.length; j++) {
                const a = nodes[i]!, b = nodes[j]!;
                const dx = a.x - b.x;
                const dy = a.y - b.y;
                const dz = a.z - b.z;
                const dist2 = dx * dx + dy * dy + dz * dz + 0.01;
                const force = REPULSION / dist2;
                a.vx += dx * force; a.vy += dy * force; a.vz += dz * force;
                b.vx -= dx * force; b.vy -= dy * force; b.vz -= dz * force;
            }
        }

        // Attraction along edges (spring)
        for (const e of edges) {
            const a = nodes[e.from], b = nodes[e.to];
            if (!a || !b) continue;
            const dx = b.x - a.x;
            const dy = b.y - a.y;
            const dz = b.z - a.z;
            const dist = Math.sqrt(dx * dx + dy * dy + dz * dz) + 0.001;
            const force = dist * ATTRACTION;
            const fx = (dx / dist) * force;
            const fy = (dy / dist) * force;
            const fz = (dz / dist) * force;
            a.vx += fx; a.vy += fy; a.vz += fz;
            b.vx -= fx; b.vy -= fy; b.vz -= fz;
        }

        // Integrate
        for (const nd of nodes) {
            nd.x += nd.vx * DAMPING;
            nd.y += nd.vy * DAMPING;
            nd.z += nd.vz * DAMPING;
            nd.vx = 0; nd.vy = 0; nd.vz = 0;
        }
    }
}

function computeCenter(nodes: InternalNode[]): [number, number, number] {
    if (nodes.length === 0) return [0, 0, 0];
    let cx = 0, cy = 0, cz = 0;
    for (const nd of nodes) { cx += nd.x; cy += nd.y; cz += nd.z; }
    return [cx / nodes.length, cy / nodes.length, cz / nodes.length];
}
