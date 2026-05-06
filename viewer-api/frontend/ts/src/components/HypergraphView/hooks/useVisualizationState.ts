/**
 * Hook for deriving visualization state from graph operation events.
 * Extracts node roles and styling information from search/insert/read states.
 * Integrates VizPathGraph for precise search path edge highlighting.
 */
import { useMemo } from "preact/hooks";
import type {
    GraphOpEvent,
    LocationInfo,
    SnapshotEdge,
    Transition,
    VizPathGraph,
} from "@context-engine/types";
import { allNodeIndices } from "../search-path/reconstruction";
import {
    computeSearchEdgeKeys,
    edgePairKey,
} from "../search-path/edge-highlighting";

export interface VisualizationState {
    /** Primary node being operated on */
    selectedNode: number | null;
    /** Root of current exploration */
    rootNode: number | null;
    /** Nodes in the trace path from root to current */
    tracePath: Set<number>;
    /** Completed/matched nodes */
    completedNodes: Set<number>;
    /** Pending parent candidates */
    pendingParents: Set<number>;
    /** Pending child candidates */
    pendingChildren: Set<number>;
    /** Start node (for start_node transition) */
    startNode: number | null;
    /** Current candidate parent being explored */
    candidateParent: number | null;
    /** Current candidate child being explored */
    candidateChild: number | null;
    /** Node that matched */
    matchedNode: number | null;
    /** Node that mismatched */
    mismatchedNode: number | null;
    /** All nodes involved in current visualization (for dimming others) */
    involvedNodes: Set<number>;
    /** Whether any visualization state is active */
    hasVizState: boolean;
    /** The raw transition for additional context */
    transition: Transition | null;
    /** The raw location info */
    location: LocationInfo | null;
    /** Active search path graph (null when no search path data) */
    searchPath: VizPathGraph | null;
    /** Whether the current root is tentative (set by VisitParent, awaiting confirmation) */
    rootTentative: boolean;
    /** Edge pair keys for start_path edges (upward exploration) */
    searchStartEdgeKeys: Set<number>;
    /** Edge pair keys for root entry edge (start→root, arrow toward parent/A) */
    searchRootEntryEdgeKeys: Set<number>;
    /** Edge pair keys for root exit edge (root→end, arrow toward child/B) */
    searchRootExitEdgeKeys: Set<number>;
    /** Edge pair keys for end_path edges (downward comparison) */
    searchEndEdgeKeys: Set<number>;
    /** Query token indices (nodes in the input pattern) */
    queryTokens: Set<number>;
    /** The token currently being compared against the query (gold ring) */
    activeQueryToken: number | null;
    /** Edge pair keys for insert-specific edges (create_pattern, update_pattern, join) */
    insertEdgeKeys: Set<number>;
    /** The join result node index (for specialized edge coloring), null when not in a join */
    joinResult: number | null;

    // ── Insert-specific node roles ──

    /** Source node being split (split_start / split_complete) */
    splitSource: number | null;
    /** Left fragment after split */
    splitLeft: number | null;
    /** Right fragment after split */
    splitRight: number | null;
    /** Left node in a join step */
    joinLeft: number | null;
    /** Right node in a join step */
    joinRight: number | null;
    /** Parent node of a create_pattern / update_pattern */
    newPatternParent: number | null;
    /** Children of a create_pattern */
    newPatternChildren: Set<number>;
    /** Newly created root node */
    newRoot: number | null;
    /** Whether the current event is an insert operation */
    isInsertOp: boolean;
}

/**
 * Derive the primary node to focus on from a transition.
 */
export function getPrimaryNode(
    trans: Transition | null,
    loc: LocationInfo | null,
): number | null {
    if (trans) {
        switch (trans.kind) {
            case "start_node":
                return trans.node.index;
            case "visit_parent":
                return trans.to.index;
            case "visit_child":
                return trans.to.index;
            case "child_match":
                return trans.node.index;
            case "child_mismatch":
                return trans.node.index;
            case "done":
                return trans.final_node;
            case "candidate_mismatch":
                return trans.node.index;
            case "candidate_match":
                return trans.root.index;
            case "parent_explore":
                return trans.current_root;
            case "split_start":
                return trans.node.index;
            case "split_complete":
                return trans.original_node;
            case "join_start":
                return trans.nodes[0] ?? null;
            case "join_step":
                return trans.result;
            case "join_complete":
                return trans.result_node;
            case "create_pattern":
                return trans.parent;
            case "create_root":
                return trans.node.index;
            case "update_pattern":
                return trans.parent;
        }
    }

    // Fall back to location info
    if (loc?.root_node != null) return loc.root_node;
    if (loc?.selected_node != null) return loc.selected_node;
    return null;
}

/**
 * Hook to derive visualization state from an active graph operation event
 * and optional search path graph.
 *
 * @param snapshotEdges - Snapshot edges for computing intermediate
 *   graph edges when the VizPathGraph end_edges skip intermediate nodes.
 */
export function useVisualizationState(
    event: GraphOpEvent | null,
    searchPath?: VizPathGraph | null,
    snapshotEdges?: SnapshotEdge[] | null,
): VisualizationState {
    return useMemo(() => {
        const loc = event?.location ?? null;
        const trans = event?.transition ?? null;
        const sp = searchPath ?? null;

        const selectedNode = loc?.selected_node ?? null;
        const rootNode = loc?.root_node ?? null;
        const tracePath = new Set(loc?.trace_path ?? []);
        const completedNodes = new Set(loc?.completed_nodes ?? []);
        const pendingParents = new Set(loc?.pending_parents ?? []);
        const pendingChildren = new Set(loc?.pending_children ?? []);

        // Derive transition-specific node roles
        const startNode: number | null =
            trans?.kind === "start_node" ? trans.node.index : null;
        const candidateParent: number | null =
            trans?.kind === "visit_parent" ? trans.to.index : null;
        const candidateChild: number | null =
            trans?.kind === "visit_child" ? trans.to.index : null;
        const matchedNode: number | null =
            trans?.kind === "child_match" ? trans.node.index : null;
        const mismatchedNode: number | null =
            trans?.kind === "child_mismatch" ? trans.node.index : null;

        // Include parent_candidates from parent_explore transitions in pendingParents.
        // LocationInfo.pending_parents comes from the queue, but the queue may be empty
        // by the time the event is emitted; the transition itself carries the canonical list.
        if (trans?.kind === "parent_explore") {
            for (const n of trans.parent_candidates) pendingParents.add(n);
        }

        // ── Search path edge key sets (pair keys — pattern_idx independent) ──
        const edgeKeys = sp ? computeSearchEdgeKeys(sp) : null;
        const searchStartEdgeKeys =
            edgeKeys?.startEdgeKeys ?? new Set<number>();
        const searchRootEntryEdgeKeys =
            edgeKeys?.rootEntryEdgeKeys ?? new Set<number>();
        const searchRootExitEdgeKeys =
            edgeKeys?.rootExitEdgeKeys ?? new Set<number>();
        const searchEndEdgeKeys = edgeKeys?.endEdgeKeys ?? new Set<number>();

        // Build the set of all "involved" nodes for dimming non-involved ones
        const involvedNodes = new Set<number>();
        if (loc) {
            if (selectedNode != null) involvedNodes.add(selectedNode);
            if (rootNode != null) involvedNodes.add(rootNode);
            for (const n of loc.trace_path) involvedNodes.add(n);
            for (const n of loc.completed_nodes) involvedNodes.add(n);
            for (const n of loc.pending_parents) involvedNodes.add(n);
            for (const n of loc.pending_children) involvedNodes.add(n);
        }
        if (startNode != null) involvedNodes.add(startNode);
        if (candidateParent != null) involvedNodes.add(candidateParent);
        if (candidateChild != null) involvedNodes.add(candidateChild);
        if (matchedNode != null) involvedNodes.add(matchedNode);
        if (mismatchedNode != null) involvedNodes.add(mismatchedNode);
        // Also include transition 'from' nodes
        if (trans?.kind === "visit_parent" || trans?.kind === "visit_child") {
            involvedNodes.add(trans.from.index);
        }

        // Include search path nodes in the involved set
        if (sp) {
            for (const idx of allNodeIndices(sp)) {
                involvedNodes.add(idx);
            }
        }

        // Include intermediate nodes discovered via edge key computation.
        // The edgePairKey encodes (from << 16 | to), so decode both indices.
        for (const key of searchStartEdgeKeys) {
            involvedNodes.add(key >>> 16);
            involvedNodes.add(key & 0xffff);
        }
        for (const key of searchEndEdgeKeys) {
            involvedNodes.add(key >>> 16);
            involvedNodes.add(key & 0xffff);
        }
        for (const key of searchRootEntryEdgeKeys) {
            involvedNodes.add(key >>> 16);
            involvedNodes.add(key & 0xffff);
        }
        for (const key of searchRootExitEdgeKeys) {
            involvedNodes.add(key >>> 16);
            involvedNodes.add(key & 0xffff);
        }

        const hasVizState = involvedNodes.size > 0;

        // Query token tracking from QueryInfo
        const queryTokens = new Set<number>(event?.query?.query_tokens ?? []);
        const activeQueryToken = event?.query?.active_token ?? null;

        // ── Insert-specific node roles + edge keys ──
        const isInsertOp = event?.op_type === "insert";
        const insertEdgeKeys = new Set<number>();
        let joinResult: number | null = null;
        let splitSource: number | null = null;
        let splitLeft: number | null = null;
        let splitRight: number | null = null;
        let joinLeft: number | null = null;
        let joinRight: number | null = null;
        let newPatternParent: number | null = null;
        const newPatternChildren = new Set<number>();
        let newRoot: number | null = null;

        if (trans) {
            switch (trans.kind) {
                case "split_start":
                    splitSource = trans.node.index;
                    break;
                case "split_complete":
                    splitSource = trans.original_node;
                    splitLeft = trans.left_fragment;
                    splitRight = trans.right_fragment ?? null;
                    if (splitLeft != null) {
                        insertEdgeKeys.add(
                            edgePairKey(trans.original_node, splitLeft),
                        );
                    }
                    if (splitRight != null) {
                        insertEdgeKeys.add(
                            edgePairKey(trans.original_node, splitRight),
                        );
                    }
                    break;
                case "join_step":
                    joinLeft = trans.left;
                    joinRight = trans.right;
                    joinResult = trans.result;
                    insertEdgeKeys.add(edgePairKey(trans.result, trans.left));
                    insertEdgeKeys.add(edgePairKey(trans.result, trans.right));
                    break;
                case "join_complete":
                    joinResult = trans.result_node;
                    break;
                case "create_pattern":
                    newPatternParent = trans.parent;
                    for (const child of trans.children) {
                        newPatternChildren.add(child);
                        insertEdgeKeys.add(edgePairKey(trans.parent, child));
                    }
                    break;
                case "create_root":
                    newRoot = trans.node.index;
                    break;
                case "update_pattern":
                    newPatternParent = trans.parent;
                    for (const child of trans.new_children) {
                        insertEdgeKeys.add(edgePairKey(trans.parent, child));
                    }
                    break;
            }
        }

        // Add insert nodes to involved set so they aren't dimmed
        if (splitSource != null) involvedNodes.add(splitSource);
        if (splitLeft != null) involvedNodes.add(splitLeft);
        if (splitRight != null) involvedNodes.add(splitRight);
        if (joinLeft != null) involvedNodes.add(joinLeft);
        if (joinRight != null) involvedNodes.add(joinRight);
        if (joinResult != null) involvedNodes.add(joinResult);
        if (newPatternParent != null) involvedNodes.add(newPatternParent);
        for (const c of newPatternChildren) involvedNodes.add(c);
        if (newRoot != null) involvedNodes.add(newRoot);

        return {
            selectedNode,
            rootNode,
            tracePath,
            completedNodes,
            pendingParents,
            pendingChildren,
            startNode,
            candidateParent,
            candidateChild,
            matchedNode,
            mismatchedNode,
            involvedNodes,
            hasVizState,
            transition: trans,
            location: loc,
            searchPath: sp,
            rootTentative: sp?.root_tentative ?? false,
            searchStartEdgeKeys,
            searchRootEntryEdgeKeys,
            searchRootExitEdgeKeys,
            searchEndEdgeKeys,
            queryTokens,
            activeQueryToken,
            insertEdgeKeys,
            joinResult,
            splitSource,
            splitLeft,
            splitRight,
            joinLeft,
            joinRight,
            newPatternParent,
            newPatternChildren,
            newRoot,
            isInsertOp,
        };
    }, [event, searchPath, snapshotEdges]);
}

/**
 * Compute the CSS visualization classes for a node based on viz state.
 */
export function getNodeVizClasses(
    nodeIndex: number,
    viz: VisualizationState,
): string {
    const {
        startNode,
        selectedNode,
        rootNode,
        candidateParent,
        candidateChild,
        matchedNode,
        mismatchedNode,
        tracePath,
        completedNodes,
        pendingParents,
        pendingChildren,
        hasVizState,
        involvedNodes,
        searchPath,
    } = viz;

    const isStart = nodeIndex === startNode;
    const isSelected = nodeIndex === selectedNode && !isStart;
    const isRoot = nodeIndex === rootNode;

    // When a child_match transition is active, suppress parent-related
    // highlights — only the matched child node (and its inbound edge)
    // should draw attention.
    const suppressParentHighlight =
        matchedNode != null && nodeIndex !== matchedNode;

    // Search path node roles — all nodes in the search path get the same
    // start-path or end-path highlight; sp-start/sp-root are additive badges.
    const spStartNode = searchPath?.start_node?.index ?? -1;
    const spRootNode = searchPath?.root?.index ?? -1;
    const isSpStart = nodeIndex === spStartNode;
    const isSpRoot = nodeIndex === spRootNode;
    const isInStartPath =
        isSpStart ||
        isSpRoot ||
        (searchPath?.start_path.some((n) => n.index === nodeIndex) ?? false);
    const isSpEndPath =
        searchPath?.end_path.some((n) => n.index === nodeIndex) ?? false;
    const isCandidateParent = nodeIndex === candidateParent;
    const isCandidateChild = nodeIndex === candidateChild;
    const isMatched = nodeIndex === matchedNode;
    const isMismatched = nodeIndex === mismatchedNode;
    const isPath =
        tracePath.has(nodeIndex) &&
        !isStart &&
        !isRoot &&
        !isCandidateParent &&
        !isCandidateChild &&
        !isMatched &&
        !isMismatched;
    const isCompleted =
        completedNodes.has(nodeIndex) &&
        !isStart &&
        !isMatched &&
        !isMismatched;
    const isPendingParent = pendingParents.has(nodeIndex) && !isCandidateParent;
    const isPendingChild = pendingChildren.has(nodeIndex) && !isCandidateChild;
    // Query token roles — nodes that are part of the input pattern
    const isQueryToken = viz.queryTokens.has(nodeIndex);
    const isActiveQueryToken = nodeIndex === viz.activeQueryToken;
    // Search path nodes and query tokens are never dimmed — they are always "involved"
    const isInSearchPath = isInStartPath || isSpEndPath;
    const isDimmed =
        hasVizState &&
        !involvedNodes.has(nodeIndex) &&
        !isInSearchPath &&
        !isQueryToken;

    return [
        isStart && !suppressParentHighlight && "viz-start",
        isSelected && !suppressParentHighlight && "viz-selected",
        isRoot && !viz.rootTentative && !suppressParentHighlight && "viz-root",
        isRoot &&
            viz.rootTentative &&
            !suppressParentHighlight &&
            "viz-root-tentative",
        isCandidateParent && !suppressParentHighlight && "viz-candidate-parent",
        isCandidateChild && "viz-candidate-child",
        isMatched && "viz-matched",
        isMismatched && "viz-mismatched",
        isPath && !suppressParentHighlight && "viz-path",
        isCompleted && "viz-completed",
        isPendingParent && !suppressParentHighlight && "viz-pending-parent",
        isPendingChild && "viz-pending-child",
        isSpStart && "viz-sp-start",
        isSpRoot && "viz-sp-root",
        isInStartPath && "viz-sp-start-path",
        isSpEndPath && "viz-sp-end-path",
        isQueryToken && "viz-query-token",
        isActiveQueryToken && "viz-query-active",
        // Insert-specific classes
        nodeIndex === viz.splitSource && "viz-split-source",
        nodeIndex === viz.splitLeft && "viz-split-left",
        nodeIndex === viz.splitRight && "viz-split-right",
        nodeIndex === viz.joinLeft && "viz-join-left",
        nodeIndex === viz.joinRight && "viz-join-right",
        nodeIndex === viz.joinResult && "viz-join-result",
        nodeIndex === viz.newPatternParent && "viz-new-pattern",
        viz.newPatternChildren.has(nodeIndex) && "viz-new-pattern-child",
        nodeIndex === viz.newRoot && "viz-new-root",
        isDimmed && "viz-dimmed",
    ]
        .filter(Boolean)
        .join(" ");
}

/**
 * Get the active visualization states for a specific node (for info panel display).
 */
export function getNodeVizStates(
    nodeIndex: number,
    viz: VisualizationState,
): string[] {
    const states: string[] = [];
    if (nodeIndex === viz.startNode) states.push("start");
    if (nodeIndex === viz.selectedNode) states.push("selected");
    if (nodeIndex === viz.rootNode) states.push("root");
    if (nodeIndex === viz.candidateParent) states.push("candidate-parent");
    if (nodeIndex === viz.candidateChild) states.push("candidate-child");
    if (nodeIndex === viz.matchedNode) states.push("matched");
    if (nodeIndex === viz.mismatchedNode) states.push("mismatched");
    if (viz.tracePath.has(nodeIndex)) states.push("path");
    if (viz.completedNodes.has(nodeIndex)) states.push("completed");
    if (viz.pendingParents.has(nodeIndex)) states.push("pending-parent");
    if (viz.pendingChildren.has(nodeIndex)) states.push("pending-child");
    if (viz.searchPath) {
        if (viz.searchPath.start_node?.index === nodeIndex)
            states.push("sp-start");
        if (viz.searchPath.root?.index === nodeIndex) states.push("sp-root");
        if (viz.searchPath.start_path.some((n) => n.index === nodeIndex))
            states.push("sp-start-path");
        if (viz.searchPath.end_path.some((n) => n.index === nodeIndex))
            states.push("sp-end-path");
    }
    if (viz.queryTokens.has(nodeIndex)) states.push("query-token");
    if (nodeIndex === viz.activeQueryToken) states.push("query-active");
    // Insert-specific states
    if (nodeIndex === viz.splitSource) states.push("split-source");
    if (nodeIndex === viz.splitLeft) states.push("split-left");
    if (nodeIndex === viz.splitRight) states.push("split-right");
    if (nodeIndex === viz.joinLeft) states.push("join-left");
    if (nodeIndex === viz.joinRight) states.push("join-right");
    if (nodeIndex === viz.joinResult) states.push("join-result");
    if (nodeIndex === viz.newPatternParent) states.push("new-pattern");
    if (viz.newPatternChildren.has(nodeIndex)) states.push("new-pattern-child");
    if (nodeIndex === viz.newRoot) states.push("new-root");
    return states;
}
