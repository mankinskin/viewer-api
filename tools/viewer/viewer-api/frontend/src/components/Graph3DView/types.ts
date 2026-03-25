/**
 * Public and internal types for the Graph3DView component.
 */

// ── Public API types ──

export interface Graph3DNode {
    id: string;
    label: string;
    color?: string;
    size?: number;
    data?: unknown;
}

export interface Graph3DEdge {
    source: string;
    target: string;
    color?: string;
    style?: 'solid' | 'dashed';
}

export interface Graph3DViewProps {
    nodes: Graph3DNode[];
    edges: Graph3DEdge[];
    onNodeClick?: (node: Graph3DNode) => void;
    selectedNodeId?: string;
    layoutMode?: 'force' | 'hierarchical';
}

// ── Internal runtime types ──

/** Internal augmented node with 3D position and physics state. */
export interface InternalNode {
    /** Stable integer index derived from position in nodes array. */
    index: number;
    id: string;
    label: string;
    color?: string;
    size?: number;
    data?: unknown;
    /** Current animated 3D position. */
    x: number;
    y: number;
    z: number;
    /** Animation target position. */
    tx: number;
    ty: number;
    tz: number;
    /** Physics velocity for force-directed layout. */
    vx: number;
    vy: number;
    vz: number;
    /** Sphere radius for ray-intersection hit detection. */
    radius: number;
    /** RGBA color [0..1] sent to GPU. */
    rgba: [number, number, number, number];
    /** Indices of nodes this node has edges TO (outgoing). */
    sourceIndices: number[];
    /** Indices of nodes that have edges TO this node (incoming). */
    targetIndices: number[];
}

export interface InternalEdge {
    from: number;
    to: number;
    color?: string;
    style?: 'solid' | 'dashed';
}

export interface InternalLayout {
    nodes: InternalNode[];
    nodeMap: Map<number, InternalNode>;
    /** Maps user-facing node id → internal index. */
    nodeIdMap: Map<string, number>;
    edges: InternalEdge[];
    center: [number, number, number];
}
