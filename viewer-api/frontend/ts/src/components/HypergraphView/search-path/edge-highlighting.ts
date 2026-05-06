/**
 * Edge highlighting computation for search path visualization.
 *
 * Given a VizPathGraph, computes Sets of edge pair keys for start-path,
 * root, and end-path highlighting.  These pair keys are pattern_idx-independent
 * and match the layout edges used by the overlay renderer.
 *
 * The backend provides explicit root_entry_edge and root_exit_edge, so no
 * BFS/searching in the frontend is required — edges are converted to pair
 * keys directly.
 *
 * Extracted from useVisualizationState so the logic is independently
 * testable without React/Preact hooks.
 */

import type { VizPathGraph } from "@context-engine/types";

// ---------------------------------------------------------------------------
// Edge pair key (duplicated from HypergraphView/utils/math to avoid
// circular dependency on the component tree — identical implementation).
// ---------------------------------------------------------------------------

/** Encode two node indices into a single numeric key (supports up to 65535 nodes). */
export function edgePairKey(from: number, to: number): number {
    return (from << 16) | to;
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

export interface SearchEdgeKeys {
    /** Edge pair keys for start_path edges (upward exploration). */
    startEdgeKeys: Set<number>;
    /** Edge pair key for root entry edge (start→root, arrow toward parent/A). */
    rootEntryEdgeKeys: Set<number>;
    /** Edge pair key for root exit edge (root→end, arrow toward child/B). */
    rootExitEdgeKeys: Set<number>;
    /** Edge pair keys for end_path edges (downward comparison). */
    endEdgeKeys: Set<number>;
}

/**
 * Compute the search-path edge pair key sets for highlighting.
 *
 * Uses the explicit edge refs from VizPathGraph directly — no BFS needed.
 * Start edges are bottom-up (from=child, to=parent) so they are flipped
 * to parent→child for pair keys.  End edges and root_exit_edge are already
 * top-down.  root_entry_edge is bottom-up and also flipped.
 */
export function computeSearchEdgeKeys(sp: VizPathGraph): SearchEdgeKeys {
    const startEdgeKeys = new Set<number>();
    const rootEntryEdgeKeys = new Set<number>();
    const rootExitEdgeKeys = new Set<number>();
    const endEdgeKeys = new Set<number>();

    // ── Start path edges ──
    // Start edges are bottom-up (from=child, to=parent).
    // Layout edges are parent→child, so flip direction.
    for (const e of sp.start_edges) {
        startEdgeKeys.add(edgePairKey(e.to, e.from));
    }

    // ── Root entry edge (bottom-up: from=child, to=root) ──
    if (sp.root_entry_edge) {
        rootEntryEdgeKeys.add(
            edgePairKey(sp.root_entry_edge.to, sp.root_entry_edge.from),
        );
    }

    // ── Root exit edge (top-down: from=root, to=child) ──
    if (sp.root_exit_edge) {
        rootExitEdgeKeys.add(
            edgePairKey(sp.root_exit_edge.from, sp.root_exit_edge.to),
        );
    }

    // ── End path edges (top-down: from=parent, to=child) ──
    for (const e of sp.end_edges) {
        endEdgeKeys.add(edgePairKey(e.from, e.to));
    }

    return { startEdgeKeys, rootEntryEdgeKeys, rootExitEdgeKeys, endEdgeKeys };
}
