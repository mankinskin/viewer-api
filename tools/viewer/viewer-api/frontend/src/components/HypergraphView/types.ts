/**
 * Props interface for the HypergraphView core component.
 *
 * Decouples the rendering engine from log-viewer-specific signal dependencies
 * so that the core can eventually be extracted to viewer-api.
 */
import type {
    GraphOpEvent,
    GraphSnapshot as HypergraphSnapshot,
    VizPathGraph,
    SnapshotEdge,
} from "@context-engine/types";

/**
 * Data props accepted by HypergraphViewCore.
 * A thin wrapper reads signals and passes these as props.
 */
export interface HypergraphViewProps {
    /** The current graph snapshot to render */
    snapshot: HypergraphSnapshot | null;
    /** Current graph operation event (search/insert step) */
    currentEvent: GraphOpEvent | null;
    /** Active search path graph for edge highlighting */
    searchPath: VizPathGraph | null;
    /** Whether auto-layout is active (expand/contract around selected) */
    autoLayout: boolean;
    /** Snapshot edges for edge key computation */
    snapshotEdges: SnapshotEdge[] | null;
    /**
     * Opaque key that changes when the search step changes.
     * Used to trigger auto-focus on the primary node of a new step.
     * Typically `activeSearchStep.value + '/' + activePathStep.value`.
     */
    stepKey: string;
}

// ── Nesting View Types ──

/** User-configurable nesting settings (persisted in localStorage). */
export interface NestingSettings {
    /** Master toggle for nesting view. */
    enabled: boolean;
    /** true = show duplicates inside parent + at original position; false = reparent/move. */
    duplicateMode: boolean;
    /** How many parent shell levels to show (1–5). */
    parentDepth: number;
    /** How many child levels to show inside expanded node (1–3). */
    childDepth: number;
}

/** Describes one parent rendered as a nesting shell container. */
export interface ShellNode {
    /** Graph node index of this parent. */
    nodeIdx: number;
    /** 1 = direct parent, 2 = grandparent, etc. */
    shellLevel: number;
    /** Rendered container width (px, grows with nesting depth). */
    width: number;
    /** Rendered container height (px). */
    height: number;
    /** X offset from the selected node's center (for multi-parent horizontal spread). */
    centerX: number;
    /** Y offset from the selected node's center. */
    centerY: number;
}

/** Describes one child node duplicated inside an expanded parent. */
export interface DuplicateNode {
    /** Graph index of the original node this duplicates. */
    originalIdx: number;
    /** Stable unique ID for DOM keying. */
    duplicateId: string;
    /** Graph index of the expanded parent that contains this duplicate. */
    parentIdx: number;
    /** Position slot within the parent's row layout. */
    slotIndex: number;
}

/** Runtime nesting state (computed each frame, not persisted). */
export interface NestingState {
    settings: NestingSettings;
    /** Current center node for nesting view. */
    selectedIdx: number;
    /** Parent nodes arranged as nesting shells. */
    shells: ShellNode[];
    /** Child duplicates rendered inside expanded parents. */
    duplicates: DuplicateNode[];
}

/** Highlight applied to a node when its parent↔child edge is hidden due to nesting. */
export interface EdgeHighlight {
    nodeIdx: number;
    role: "parent" | "child";
}
