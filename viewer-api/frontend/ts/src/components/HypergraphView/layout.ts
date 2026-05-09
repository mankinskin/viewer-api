/**
 * Force-directed 3D layout engine for hypergraph nodes.
 *
 * Atoms (leaf nodes) are placed at the bottom level (y=0).
 * Compound nodes are placed at y proportional to their width.
 * XZ positions computed via spring-electrical force simulation.
 */

import type { GraphSnapshot as HypergraphSnapshot, VizPathGraph } from '@context-engine/types';

export type {
    DecompositionChild,
    DecompositionPattern,
} from './layout-decomposition';
export { getDecompositionPatterns } from './layout-decomposition';

export interface LayoutNode {
    index: number;
    label: string;
    width: number;
    isAtom: boolean;
    x: number;
    y: number;
    z: number;
    /** Animation target X — lerp toward this each frame */
    tx: number;
    /** Animation target Y */
    ty: number;
    /** Animation target Z */
    tz: number;
    vx: number;
    vy: number;
    vz: number;
    radius: number;
    color: [number, number, number, number];
    parentIndices: number[];
    childIndices: number[];
}

export interface LayoutEdge {
    from: number;
    to: number;
    patternIdx: number;
    subIndex: number;
}

export interface GraphLayout {
    nodes: LayoutNode[];
    nodeMap: Map<number, LayoutNode>;
    edges: LayoutEdge[];
    maxWidth: number;
    /** Center of mass of all nodes (for initial camera target) */
    center: [number, number, number];
}

// ── Color palette (hue-based on node width) ──

function widthColor(width: number, maxWidth: number): [number, number, number, number] {
    if (width === 1) return [0.55, 0.75, 0.95, 1]; // atoms: soft blue
    const t = Math.min((width - 1) / Math.max(maxWidth - 1, 1), 1);
    // gradient from green → orange → red-ish as width grows
    const r = 0.3 + t * 0.6;
    const g = 0.8 - t * 0.4;
    const b = 0.3 + (1 - t) * 0.3;
    return [r, g, b, 1];
}

export function buildLayout(snapshot: HypergraphSnapshot): GraphLayout {
    const maxWidth = Math.max(...snapshot.nodes.map(n => n.width), 1);

    // Build adjacency
    const childMap = new Map<number, Set<number>>();
    const parentMap = new Map<number, Set<number>>();
    for (const e of snapshot.edges) {
        if (!childMap.has(e.from)) childMap.set(e.from, new Set());
        childMap.get(e.from)!.add(e.to);
        if (!parentMap.has(e.to)) parentMap.set(e.to, new Set());
        parentMap.get(e.to)!.add(e.from);
    }

    // Initial positions: circular in XZ, Y by width
    const nodes: LayoutNode[] = snapshot.nodes.map((n, i) => {
        const angle = (i / snapshot.nodes.length) * Math.PI * 2;
        const r = 0.45 + snapshot.nodes.length * 0.5;
        return {
            index: n.index,
            label: n.label,
            width: n.width,
            isAtom: n.width === 1,
            x: Math.cos(angle) * r * (0.5 + Math.random() * 0.5),
            y: (n.width - 1) * 1.67,
            z: Math.sin(angle) * r * (0.5 + Math.random() * 0.5),
            tx: 0, ty: 0, tz: 0, // set after simulate
            vx: 0, vy: 0, vz: 0,
            radius: 0.45 + Math.min(n.width * 0.5, 0.3),
            color: widthColor(n.width, maxWidth),
            parentIndices: [...(parentMap.get(n.index) || [])],
            childIndices: [...(childMap.get(n.index) || [])],
        };
    });

    const nodeMap = new Map<number, LayoutNode>();
    for (const n of nodes) nodeMap.set(n.index, n);

    // Deduplicate edges
    const edgeSet = new Set<string>();
    const edges: LayoutEdge[] = [];
    for (const e of snapshot.edges) {
        const key = `${e.from}-${e.to}-${e.pattern_idx}`;
        if (!edgeSet.has(key)) {
            edgeSet.add(key);
            edges.push({ from: e.from, to: e.to, patternIdx: e.pattern_idx, subIndex: e.sub_index });
        }
    }

    // Run force simulation
    simulate(nodes, edges, nodeMap, 150);

    // Center the layout
    if (nodes.length > 0) {
        let cx = 0, cz = 0;
        for (const n of nodes) { cx += n.x; cz += n.z; }
        cx /= nodes.length; cz /= nodes.length;
        for (const n of nodes) { n.x -= cx; n.z -= cz; }
    }

    // Sync animation targets to initial positions
    for (const n of nodes) { n.tx = n.x; n.ty = n.y; n.tz = n.z; }

    // Compute center of mass
    let cenX = 0, cenY = 0, cenZ = 0;
    for (const n of nodes) { cenX += n.x; cenY += n.y; cenZ += n.z; }
    if (nodes.length > 0) { cenX /= nodes.length; cenY /= nodes.length; cenZ /= nodes.length; }

    return { nodes, nodeMap, edges, maxWidth, center: [cenX, cenY, cenZ] };
}

// ── Layout constants ──
const SPACING_Y = 0.5;
/** Extra vertical gap between the selected node and the first parent row. */
const PARENT_PAD_Y = 0.9;

/**
 * Camera-relative axes for screen-oriented layout.
 * `right` points rightward on screen, `up` points upward on screen.
 * Each is a unit vector in world space.
 */
export interface CameraAxes {
    right: [number, number, number];
    up: [number, number, number];
}

/**
 * Abstract 2D offset from the anchor node in screen-local coordinates.
 * `dRight` = displacement along screen-right axis.
 * `dUp`    = displacement along screen-up axis.
 */
export interface FocusedOffset {
    dRight: number;
    dUp: number;
}

/**
 * Result of a focused layout computation.
 * Contains the anchor node index and abstract 2D offsets for each affected node.
 * To convert to world positions, project each offset using camera right/up vectors:
 *   worldPos = anchorPos + dRight * cameraRight + dUp * cameraUp
 */
export interface FocusedLayoutOffsets {
    anchorIdx: number;
    offsets: Map<number, FocusedOffset>;
}

/**
 * Compute focused-layout offsets for a selected node.
 * - Parents arranged above the selected node, grouped by width (smallest closest)
 * - Children are rendered inside the parent DOM (not positioned in 3D)
 *
 * Returns abstract 2D offsets (dRight, dUp) from the anchor node.
 * The caller projects these onto camera axes to get world positions:
 *   worldPos = anchorWorldPos + dRight * cameraRight + dUp * cameraUp
 */
export function computeFocusedLayout(
    layout: GraphLayout,
    selectedIdx: number,
): FocusedLayoutOffsets | null {
    const selected = layout.nodeMap.get(selectedIdx);
    if (!selected) return null;

    const offsets = new Map<number, FocusedOffset>();

    // Anchor node has zero offset
    offsets.set(selectedIdx, { dRight: 0, dUp: 0 });

    // ── Parents above in a multi-level fan, ordered by width ──
    const parents = selected.parentIndices
        .map(idx => layout.nodeMap.get(idx))
        .filter((n): n is LayoutNode => n != null);

    // Group parents by width, sort groups ascending (smallest width = closest ring)
    const parentsByWidth = new Map<number, LayoutNode[]>();
    for (const p of parents) {
        if (!parentsByWidth.has(p.width)) parentsByWidth.set(p.width, []);
        parentsByWidth.get(p.width)!.push(p);
    }
    const sortedWidths = [...parentsByWidth.keys()].sort((a, b) => a - b);

    for (let ring = 0; ring < sortedWidths.length; ring++) {
        const group = parentsByWidth.get(sortedWidths[ring]!)!;
        const radius = PARENT_PAD_Y + (ring + 1) * SPACING_Y;
        const count = group.length;

        if (count === 1) {
            // Single node in this ring: directly above
            offsets.set(group[0]!.index, { dRight: 0, dUp: radius });
        } else {
            // Fan: spread nodes in an arc, wider for more nodes
            const fanAngleRange = Math.min(Math.PI * 0.7, (count - 1) * 0.35);
            const startAngle = (Math.PI - fanAngleRange) / 2;
            for (let i = 0; i < count; i++) {
                const t = i / (count - 1);
                const angle = startAngle + t * fanAngleRange;
                offsets.set(group[i]!.index, {
                    dRight: -Math.cos(angle) * radius * 1.2,
                    dUp: Math.sin(angle) * radius,
                });
            }
        }
    }

    // Children are rendered inside the parent's DOM element —
    // no 3D offsets needed for them.

    return { anchorIdx: selectedIdx, offsets };
}

/**
 * Compute search-path-aware layout offsets.
 * Anchors the layout on the search path ROOT (not the currently selected child),
 * then expands children hierarchically along the end_path chain.
 *
 * This keeps the root and start_path nodes stable while the user traverses
 * deeper into the end_path (child comparisons).
 *
 * Returns abstract 2D offsets (same format as computeFocusedLayout).
 */
export function computeSearchPathLayout(
    layout: GraphLayout,
    searchPath: VizPathGraph,
    selectedIdx: number,
): FocusedLayoutOffsets | null {
    const root = searchPath.root;
    if (!root) {
        // No root set yet — fall back to regular focused layout
        return computeFocusedLayout(layout, selectedIdx);
    }

    // Use the root as the anchor for the focused layout
    const result = computeFocusedLayout(layout, root.index);
    if (!result) return null;
    const { offsets } = result;

    // Collapse children for each node along the end_path chain
    // into parent DOM — no 3D offsets needed.

    // If the selected node is not root and not in end_path,
    // its children are also rendered inside its DOM.

    return { anchorIdx: root.index, offsets };
}

function simulate(
    nodes: LayoutNode[],
    edges: LayoutEdge[],
    nodeMap: Map<number, LayoutNode>,
    iterations: number,
) {
    const repulsion = 0.6;
    const springK = 20 / 1000.0;
    const ySpringK = 5 / 1000.0;
    const springLen = 0.1;
    const damping = 0.85;
    const gravity = 0.04;
    const dt = 0.4;
    // Minimum repulsion distance (prevents near-zero denominators from
    // producing enormous forces that fling nodes to extreme positions).
    const minRepulsionDist = 1.0;
    // Maximum velocity per axis per iteration — prevents a single unlucky
    // close encounter from launching a node far away from the cluster.
    const maxVelocity = 3.0;

    for (let iter = 0; iter < iterations; iter++) {
        const temp = 1.0 - iter / iterations;

        // Repulsion (all pairs)
        for (let i = 0; i < nodes.length; i++) {
            for (let j = i + 1; j < nodes.length; j++) {
                const a = nodes[i]!;
                const b = nodes[j]!;
                let dx = a.x - b.x;
                let dz = a.z - b.z;
                let dist = Math.sqrt(dx * dx + dz * dz);
                if (dist < 0.01) { dx = Math.random() - 0.5; dz = Math.random() - 0.5; dist = 0.5; }
                // Clamp distance to prevent explosion when nodes are very close
                if (dist < minRepulsionDist) dist = minRepulsionDist;
                const force = repulsion / (dist * dist) * temp;
                const fx = (dx / dist) * force;
                const fz = (dz / dist) * force;
                a.vx += fx; a.vz += fz;
                b.vx -= fx; b.vz -= fz;
            }
        }

        // Spring attraction (edges)
        for (const e of edges) {
            const a = nodeMap.get(e.from);
            const b = nodeMap.get(e.to);
            if (!a || !b) continue;
            let dx = b.x - a.x;
            let dz = b.z - a.z;
            let dist = Math.sqrt(dx * dx + dz * dz);
            if (dist < 0.01) dist = 0.01;
            const force = springK * (dist - springLen) * temp;
            const fx = (dx / dist) * force;
            const fz = (dz / dist) * force;
            a.vx += fx; a.vz += fz;
            b.vx -= fx; b.vz -= fz;
        }

        // Y-axis spring to target level
        for (const n of nodes) {
            const targetY = (n.width - 1) * 0.27;
            n.vy += (targetY - n.y) * ySpringK;
        }

        // Center gravity (prevents disconnected components from drifting apart)
        if (nodes.length > 1) {
            let cx = 0, cz = 0;
            for (const n of nodes) { cx += n.x; cz += n.z; }
            cx /= nodes.length; cz /= nodes.length;
            for (const n of nodes) {
                n.vx -= (n.x - cx) * gravity * temp;
                n.vz -= (n.z - cz) * gravity * temp;
            }
        }

        // Integrate with velocity capping
        for (const n of nodes) {
            n.vx *= damping; n.vy *= damping; n.vz *= damping;
            // Clamp velocity to prevent outlier nodes
            if (n.vx > maxVelocity) n.vx = maxVelocity;
            else if (n.vx < -maxVelocity) n.vx = -maxVelocity;
            if (n.vy > maxVelocity) n.vy = maxVelocity;
            else if (n.vy < -maxVelocity) n.vy = -maxVelocity;
            if (n.vz > maxVelocity) n.vz = maxVelocity;
            else if (n.vz < -maxVelocity) n.vz = -maxVelocity;
            n.x += n.vx * dt;
            n.y += n.vy * dt;
            n.z += n.vz * dt;
        }
    }

    // Zero out velocities
    for (const n of nodes) { n.vx = 0; n.vy = 0; n.vz = 0; }
}
