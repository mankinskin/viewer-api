/**
 * Edge instance buffer builder — classifies edges and assigns colors.
 *
 * Outputs a packed Float32Array that maps 1:1 to the layout edge list,
 * ready to be uploaded to the GPU edge instance buffer.
 */
import {
    EDGE_INSTANCE_FLOATS,
    PATTERN_COLORS,
    PATH_EDGE_COLOR,
    SP_PATH_EDGE_COLOR,
    SP_ROOT_EDGE_COLOR,
    CANDIDATE_EDGE_COLOR,
    PARENT_EDGE_COLOR,
    CHILD_EDGE_COLOR,
    INSERT_EDGE_COLOR,
    INSERT_JOIN_EDGE_COLOR,
} from './constants';
import type { GraphLayout, LayoutEdge } from '../layout';
import type { VisualizationState } from '../hooks/useVisualizationState';
import type { InteractionState } from '../hooks/useMouseInteraction';
import { edgePairKey, edgeTripleKey } from '../utils/math';

// ── Types ──

export interface EdgeBuildResult {
    edgeData: Float32Array;
    edgeCount: number;
}

export interface EdgeBuildContext {
    layout: GraphLayout;
    vizState: VisualizationState;
    inter: InteractionState;
    connectedEdgeKeys: Set<number>;
    hiddenDecompEdgeKeys: Set<number>;
    /** Edge keys hidden by nesting view (parent↔nested child). */
    hiddenNestingEdgeKeys: Set<number>;
    lastParentCandidates: number[];
}

// ── Edge classification ──

const enum EdgeType {
    Normal = 1,
    SpStart = 2,
    SpEnd = 4,
    TracePath = 5,
    Candidate = 6,
    Insert = 7,
    SpRootEntry = 8,
    SpRootExit = 9,
}

/**
 * Fill the provided `edgeDataBuf` with per-edge instance data.
 *
 * Returns the number of edges written (always === layout.edges.length).
 * Mutates `lastParentCandidates` in the context when transition changes.
 */
export function buildEdgeInstances(
    edgeDataBuf: Float32Array,
    ctx: EdgeBuildContext,
): number {
    const { layout, vizState, inter, connectedEdgeKeys, hiddenDecompEdgeKeys, hiddenNestingEdgeKeys } = ctx;

    // ── Trace path edge keys (pair-based) ──
    const vizTracePath = vizState.location?.trace_path ?? [];
    const pathEdgeKeys = new Set<number>();
    for (let p = 0; p < vizTracePath.length - 1; p++) {
        const from = vizTracePath[p]!, to = vizTracePath[p + 1]!;
        pathEdgeKeys.add(edgePairKey(from, to));
        pathEdgeKeys.add(edgePairKey(to, from));
    }

    // ── Search path edge keys ──
    const spStartKeys = vizState.searchStartEdgeKeys;
    const spRootEntryKeys = vizState.searchRootEntryEdgeKeys;
    const spRootExitKeys = vizState.searchRootExitEdgeKeys;
    const spEndKeys = vizState.searchEndEdgeKeys;
    const hasSearchPath = spStartKeys.size > 0 || spRootEntryKeys.size > 0 || spRootExitKeys.size > 0 || spEndKeys.size > 0;
    const hasViz = vizTracePath.length > 0 || vizState.selectedNode != null || hasSearchPath;

    // ── Insert edge keys ──
    const insertKeys = vizState.insertEdgeKeys;

    // ── Parent candidate tracking across steps ──
    const trans = vizState.transition;
    if (trans?.kind === 'parent_explore') {
        ctx.lastParentCandidates = trans.parent_candidates;
    } else if (trans?.kind !== 'visit_parent' && trans?.kind !== 'candidate_match') {
        ctx.lastParentCandidates = [];
    }

    // ── Candidate node set ──
    const candidateNodes = new Set<number>();
    if (vizState.candidateParent != null) candidateNodes.add(vizState.candidateParent);
    if (vizState.candidateChild != null) candidateNodes.add(vizState.candidateChild);
    for (const n of vizState.pendingParents) candidateNodes.add(n);
    for (const n of vizState.pendingChildren) candidateNodes.add(n);
    for (const n of ctx.lastParentCandidates) candidateNodes.add(n);

    // ── Fill edge instances ──
    for (let i = 0; i < layout.edges.length; i++) {
        const e = layout.edges[i]!;
        const a = layout.nodeMap.get(e.from);
        const b = layout.nodeMap.get(e.to);
        if (!a || !b) continue;
        const off = i * EDGE_INSTANCE_FLOATS;

        // Hide edges between expanded parents and their inline children
        if (hiddenDecompEdgeKeys.has(edgePairKey(e.from, e.to))
            || hiddenNestingEdgeKeys.has(edgePairKey(e.from, e.to))) {
            for (let j = 0; j < EDGE_INSTANCE_FLOATS; j++) edgeDataBuf[off + j] = 0;
            continue;
        }

        edgeDataBuf[off] = a.x;
        edgeDataBuf[off + 1] = a.y;
        edgeDataBuf[off + 2] = a.z;
        edgeDataBuf[off + 3] = b.x;
        edgeDataBuf[off + 4] = b.y;
        edgeDataBuf[off + 5] = b.z;

        // Classify edge
        const { r, g, b: b2, alpha, hlFlag, edgeType } = classifyAndColorEdge(
            e, pathEdgeKeys, spStartKeys, spRootEntryKeys, spRootExitKeys, spEndKeys,
            hasSearchPath, hasViz, candidateNodes, insertKeys, vizState, inter,
            connectedEdgeKeys,
        );

        edgeDataBuf[off + 6] = r;
        edgeDataBuf[off + 7] = g;
        edgeDataBuf[off + 8] = b2;
        edgeDataBuf[off + 9] = alpha;
        edgeDataBuf[off + 10] = hlFlag;
        edgeDataBuf[off + 11] = edgeType;
    }

    return layout.edges.length;
}

// ── Internal helpers ──

interface EdgeColorResult {
    r: number;
    g: number;
    b: number;
    alpha: number;
    hlFlag: number;
    edgeType: number;
}

function classifyAndColorEdge(
    e: LayoutEdge,
    pathEdgeKeys: Set<number>,
    spStartKeys: Set<number>,
    spRootEntryKeys: Set<number>,
    spRootExitKeys: Set<number>,
    spEndKeys: Set<number>,
    _hasSearchPath: boolean,
    hasViz: boolean,
    candidateNodes: Set<number>,
    insertKeys: Set<number>,
    vizState: VisualizationState,
    inter: InteractionState,
    connectedEdgeKeys: Set<number>,
): EdgeColorResult {
    const pairKey = edgePairKey(e.from, e.to);

    const isSpStartEdge = spStartKeys.has(pairKey);
    const isSpRootEntryEdge = spRootEntryKeys.has(pairKey);
    const isSpRootExitEdge = spRootExitKeys.has(pairKey);
    const isSpRootEdge = isSpRootEntryEdge || isSpRootExitEdge;
    const isSpEndEdge = spEndKeys.has(pairKey);
    const isSearchPathEdge = isSpStartEdge || isSpRootEdge || isSpEndEdge;

    const isPathEdge = !isSearchPathEdge && pathEdgeKeys.has(pairKey);
    const highlighted = connectedEdgeKeys.has(edgeTripleKey(e.from, e.to, e.patternIdx));

    const isCandidateEdge = !isSearchPathEdge && !isPathEdge &&
        candidateNodes.size > 0 &&
        (candidateNodes.has(e.from) || candidateNodes.has(e.to));

    const isInsertEdge = !isSearchPathEdge && !isPathEdge && !isCandidateEdge &&
        insertKeys.size > 0 &&
        insertKeys.has(pairKey);

    const isJoinEdge = isInsertEdge && vizState.joinResult != null &&
        (e.from === vizState.joinResult || e.to === vizState.joinResult);

    // Pick color + type
    let r: number, g: number, b: number, alpha: number, hlFlag: number;

    if (isSpRootEdge) {
        r = SP_ROOT_EDGE_COLOR[0]; g = SP_ROOT_EDGE_COLOR[1]; b = SP_ROOT_EDGE_COLOR[2];
        alpha = 0.95; hlFlag = 1;
    } else if (isSpStartEdge) {
        r = SP_PATH_EDGE_COLOR[0]; g = SP_PATH_EDGE_COLOR[1]; b = SP_PATH_EDGE_COLOR[2];
        alpha = 0.9; hlFlag = 1;
    } else if (isSpEndEdge) {
        r = SP_PATH_EDGE_COLOR[0]; g = SP_PATH_EDGE_COLOR[1]; b = SP_PATH_EDGE_COLOR[2];
        alpha = 0.9; hlFlag = 1;
    } else if (isPathEdge) {
        r = PATH_EDGE_COLOR[0]; g = PATH_EDGE_COLOR[1]; b = PATH_EDGE_COLOR[2];
        alpha = 0.9; hlFlag = 1;
    } else if (isCandidateEdge) {
        r = CANDIDATE_EDGE_COLOR[0]; g = CANDIDATE_EDGE_COLOR[1]; b = CANDIDATE_EDGE_COLOR[2];
        alpha = 0.30; hlFlag = 0;
    } else if (isInsertEdge) {
        if (isJoinEdge) {
            r = INSERT_JOIN_EDGE_COLOR[0]; g = INSERT_JOIN_EDGE_COLOR[1]; b = INSERT_JOIN_EDGE_COLOR[2];
        } else {
            r = INSERT_EDGE_COLOR[0]; g = INSERT_EDGE_COLOR[1]; b = INSERT_EDGE_COLOR[2];
        }
        alpha = 0.85; hlFlag = 1;
    } else if (inter.selectedIdx >= 0) {
        if (highlighted) {
            const isParentEdge = e.to === inter.selectedIdx;
            if (isParentEdge) {
                r = PARENT_EDGE_COLOR[0]; g = PARENT_EDGE_COLOR[1]; b = PARENT_EDGE_COLOR[2];
            } else {
                r = CHILD_EDGE_COLOR[0]; g = CHILD_EDGE_COLOR[1]; b = CHILD_EDGE_COLOR[2];
            }
            alpha = 0.85; hlFlag = 1;
        } else {
            const pc = PATTERN_COLORS[e.patternIdx % PATTERN_COLORS.length]!;
            r = pc[0]; g = pc[1]; b = pc[2];
            alpha = 0.12; hlFlag = 0;
        }
    } else if (hasViz) {
        const pc = PATTERN_COLORS[e.patternIdx % PATTERN_COLORS.length]!;
        r = pc[0]; g = pc[1]; b = pc[2];
        alpha = 0.12; hlFlag = 0;
    } else {
        const pc = PATTERN_COLORS[e.patternIdx % PATTERN_COLORS.length]!;
        r = pc[0]; g = pc[1]; b = pc[2];
        alpha = 0.4; hlFlag = 0;
    }

    // edgeType: 0=grid, 1=normal, 2=SP-start, 3=unused, 4=SP-end,
    //   5=trace-path, 6=candidate, 7=insert, 8=SP-root-entry, 9=SP-root-exit
    const edgeType = isSpStartEdge ? EdgeType.SpStart
        : isSpRootEntryEdge ? EdgeType.SpRootEntry
            : isSpRootExitEdge ? EdgeType.SpRootExit
                : isSpEndEdge ? EdgeType.SpEnd
                    : isPathEdge ? EdgeType.TracePath
                        : isCandidateEdge ? EdgeType.Candidate
                            : isInsertEdge ? EdgeType.Insert
                                : EdgeType.Normal;

    return { r, g, b, alpha, hlFlag, edgeType };
}
