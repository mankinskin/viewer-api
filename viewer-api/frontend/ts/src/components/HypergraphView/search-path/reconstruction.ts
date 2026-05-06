/**
 * Search path reconstruction from a sequence of PathTransition events.
 *
 * This module mirrors the Rust `VizPathGraph::apply()` / `from_transitions()`
 * logic, allowing the frontend to reconstruct search paths purely from
 * incremental transitions read from log files.
 *
 * The reconstruction is deterministic: given the same transition sequence,
 * both Rust and TypeScript produce identical VizPathGraph objects.
 */

import type {
    EdgeRef,
    PathTransition,
    VizPathGraph,
} from "@context-engine/types";

// ---------------------------------------------------------------------------
// VizPathGraph construction helpers
// ---------------------------------------------------------------------------

/** Create an empty VizPathGraph. */
export function emptyPathGraph(): VizPathGraph {
    return {
        start_node: null,
        start_path: [],
        start_edges: [],
        root: null,
        root_entry_edge: null,
        root_exit_edge: null,
        end_path: [],
        end_edges: [],
        cursor_pos: 0,
        done: false,
        success: false,
        root_tentative: false,
    };
}

/**
 * Apply a single PathTransition to a VizPathGraph, mutating it in place.
 *
 * Throws if the transition is invalid for the current state (same rules
 * as the Rust implementation).
 */
export function applyTransition(
    graph: VizPathGraph,
    transition: PathTransition,
): void {
    switch (transition.kind) {
        case "set_start_node":
            if (graph.start_node !== null) {
                throw new Error("SetStartNode called twice");
            }
            graph.start_node = transition.node;
            break;

        case "push_parent":
            if (graph.start_node === null) {
                throw new Error("PushParent before SetStartNode");
            }
            if (graph.root !== null) {
                // Root already set — extend start path upward past the old root.
                // Demote old root + its edge into start_path, then push the new parent.
                graph.start_path.push(graph.root);
                graph.start_edges.push(graph.root_entry_edge!);
                graph.root = null;
                graph.root_entry_edge = null;
                // Clear end_path — previous root's children are no longer relevant
                graph.end_path.length = 0;
                graph.end_edges.length = 0;
                graph.root_exit_edge = null;
            }
            graph.start_path.push(transition.parent);
            graph.start_edges.push(transition.edge);
            break;

        case "set_root":
            if (graph.start_node === null) {
                throw new Error("SetRoot before SetStartNode");
            }
            if (graph.root !== null) {
                throw new Error("SetRoot called twice");
            }
            // If root matches the last start_path entry, pop it
            // (the node is "graduating" from candidate to confirmed root).
            // Use the popped edge as root_entry_edge since it correctly connects
            // the prior start_path node to this root.
            if (
                graph.start_path.length > 0 &&
                graph.start_path[graph.start_path.length - 1]!.index ===
                    transition.root.index
            ) {
                graph.start_path.pop();
                const poppedEdge = graph.start_edges.pop();
                graph.root = transition.root;
                graph.root_entry_edge = poppedEdge ?? transition.edge;
            } else {
                graph.root = transition.root;
                graph.root_entry_edge = transition.edge;
            }
            break;

        case "push_child":
            if (graph.root === null) {
                throw new Error("PushChild before SetRoot");
            }
            graph.end_path.push(transition.child);
            graph.end_edges.push(transition.edge);
            // Set root_exit_edge on the first child pushed
            if (graph.end_path.length === 1) {
                graph.root_exit_edge = transition.edge;
            }
            break;

        case "pop_child":
            if (graph.end_path.length === 0) {
                throw new Error("PopChild on empty end_path");
            }
            graph.end_path.pop();
            graph.end_edges.pop();
            // Clear root_exit_edge if end_path is now empty
            if (graph.end_path.length === 0) {
                graph.root_exit_edge = null;
            }
            break;

        case "replace_child":
            if (graph.end_path.length === 0) {
                throw new Error("ReplaceChild on empty end_path");
            }
            graph.end_path[graph.end_path.length - 1] = transition.child;
            graph.end_edges[graph.end_edges.length - 1] = transition.edge;
            // Update root_exit_edge if replacing the first child
            if (graph.end_path.length === 1) {
                graph.root_exit_edge = transition.edge;
            }
            break;

        case "child_match":
            graph.cursor_pos = transition.cursor_pos;
            break;

        case "child_mismatch":
            graph.cursor_pos = transition.cursor_pos;
            break;

        case "done":
            graph.done = true;
            graph.success = transition.success;
            break;

        default:
            throw new Error(
                `Unknown transition kind: ${(transition as any).kind}`,
            );
    }
}

/**
 * Reconstruct a VizPathGraph from a sequence of transitions.
 *
 * Throws if any transition is invalid for the state at that point.
 */
export function fromTransitions(transitions: PathTransition[]): VizPathGraph {
    const graph = emptyPathGraph();
    for (let i = 0; i < transitions.length; i++) {
        const transition = transitions[i]!;
        try {
            applyTransition(graph, transition);
        } catch (e) {
            throw new Error(
                `step ${i}: ${e instanceof Error ? e.message : String(e)}`,
            );
        }
    }
    return graph;
}

/**
 * Get all node indices referenced in a VizPathGraph (for highlighting).
 */
export function allNodeIndices(graph: VizPathGraph): number[] {
    const indices: number[] = [];
    if (graph.start_node) indices.push(graph.start_node.index);
    for (const n of graph.start_path) indices.push(n.index);
    if (graph.root) indices.push(graph.root.index);
    for (const n of graph.end_path) indices.push(n.index);
    return indices;
}

/**
 * Get all edge refs in a VizPathGraph (for highlighting).
 */
export function allEdges(graph: VizPathGraph): EdgeRef[] {
    const edges: EdgeRef[] = [];
    edges.push(...graph.start_edges);
    if (graph.root_entry_edge) edges.push(graph.root_entry_edge);
    if (graph.root_exit_edge) edges.push(graph.root_exit_edge);
    edges.push(...graph.end_edges);
    return edges;
}
