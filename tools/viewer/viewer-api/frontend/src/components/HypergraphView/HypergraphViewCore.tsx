/**
 * HypergraphViewCore — signal-free rendering engine.
 *
 * Accepts data via props instead of reading from the log-viewer store.
 * This makes the component extractable to viewer-api in the future.
 *
 * UI chrome panels (SearchStatePanel, InsertStatePanel, etc.) are rendered
 * by the parent wrapper, not by this component.
 */
import {
    useRef,
    useEffect,
    useState,
    useCallback,
    useMemo,
} from "preact/hooks";
import type { ComponentChildren } from "preact";
import "./styles/base.css";
import "./styles/panels.css";
import "./styles/search-panel.css";
import "./styles/viz-states.css";
import "./styles/operation-panels.css";
import "./styles/decomposition.css";
import "./styles/nesting.css";
import {
    buildLayout,
    computeFocusedLayout,
    computeSearchPathLayout,
    type GraphLayout,
    type FocusedLayoutOffsets,
} from "./layout";
import type {
    HypergraphViewProps,
    NestingSettings,
    DuplicateNode,
} from "./types";
import type { VizPathGraph } from "@context-engine/types";

// Hooks
import {
    useCamera,
    useVisualizationState,
    useMouseInteraction,
    useTouchInteraction,
    useOverlayRenderer,
    useNestingState,
    getPrimaryNode,
} from "./hooks";

// Nesting modules
import { computeShellLayout } from "./nesting/shellLayout";

// Components
import {
    NodeInfoPanel,
    GraphInfoOverlay,
    NodeLayer,
} from "./components";

export interface HypergraphViewCoreProps extends HypergraphViewProps {
    /**
     * Render-prop for additional children.
     * Receives `handleFocusNode` callback plus nesting state for panels.
     */
    renderChildren?: (ctx: {
        handleFocusNode: (nodeIndex: number) => void;
        nestingSettings: NestingSettings;
        setNestingSettings: (update: Partial<NestingSettings>) => void;
    }) => ComponentChildren;
}

/**
 * In dup=off nesting mode, child nodes are shown as clones inside their
 * expanded parent.  When the user selects such a child, the camera should
 * target the parent's 3-D position (where the child is visually displayed)
 * instead of the hidden original.
 *
 * Returns the node index whose layout position the camera should focus on.
 */
function getCameraFocusIdx(
    layout: GraphLayout,
    selectedIdx: number,
    nesting: NestingSettings,
    searchPath: VizPathGraph | null,
): number {
    if (selectedIdx < 0) return selectedIdx;
    if (!nesting.enabled || nesting.duplicateMode) return selectedIdx;

    // Replicate the desiredExpanded logic from useOverlayRenderer so we know
    // which nodes are expanded and can detect when selectedIdx lives inside
    // one of them.
    const desiredExpanded = new Set<number>();
    desiredExpanded.add(selectedIdx);
    const spRootIdx = searchPath?.root?.index;
    if (spRootIdx != null) desiredExpanded.add(spRootIdx);

    // Prune: remove any node that is a child of another expanded node
    // (but never prune the search-path root).
    for (const idx of [...desiredExpanded]) {
        if (idx === spRootIdx) continue;
        const node = layout.nodeMap.get(idx);
        if (!node) continue;
        for (const otherIdx of desiredExpanded) {
            if (otherIdx === idx) continue;
            const other = layout.nodeMap.get(otherIdx);
            if (other && other.childIndices.includes(idx)) {
                desiredExpanded.delete(idx);
                break;
            }
        }
    }

    // If selectedIdx survived pruning it is expanded itself — focus on it.
    if (desiredExpanded.has(selectedIdx)) return selectedIdx;

    // selectedIdx was pruned → it lives inside an expanded parent.
    // Find which expanded node owns it and redirect focus there.
    for (const expIdx of desiredExpanded) {
        const expNode = layout.nodeMap.get(expIdx);
        if (expNode && expNode.childIndices.includes(selectedIdx)) {
            return expIdx;
        }
    }

    return selectedIdx;
}

/**
 * Core hypergraph visualization — no signal dependencies.
 */
export function HypergraphViewCore(props: HypergraphViewCoreProps) {
    const {
        snapshot,
        currentEvent,
        searchPath: currentSearchPath,
        autoLayout,
        snapshotEdges,
        stepKey,
        renderChildren,
    } = props;

    const containerRef = useRef<HTMLDivElement>(null);
    const nodeLayerRef = useRef<HTMLDivElement>(null);

    // Layout state
    const [layout, setLayout] = useState<GraphLayout | null>(null);
    const layoutRef = useRef<GraphLayout | null>(null);
    const basePositionsRef = useRef<Map<
        number,
        { x: number; y: number; z: number }
    > | null>(null);

    // Camera controller
    const camera = useCamera();

    // Visualization state from search events + search path
    const vizState = useVisualizationState(
        currentEvent,
        currentSearchPath,
        snapshotEdges,
    );

    // Mouse interaction (autoLayout passed via ref so it's always current)
    const autoLayoutRef = useRef(autoLayout);
    autoLayoutRef.current = autoLayout;
    const { selectedIdx, setSelectedIdx, interRef } =
        useMouseInteraction(containerRef, layoutRef, camera, autoLayoutRef);

    // Touch interaction (shares selection state via interRef)
    useTouchInteraction(
        containerRef,
        layoutRef,
        camera,
        interRef,
        setSelectedIdx,
    );

    // Nesting state (persisted to localStorage)
    const { nestingSettings, setNestingSettings } = useNestingState();
    const nestingSettingsRef = useRef(nestingSettings);
    nestingSettingsRef.current = nestingSettings;

    // Compute nesting data for NodeLayer DOM elements
    // Nesting requires autoLayout to be on (layout=on → nesting → duplication cascade)
    const { nestShells, nestDuplicates, nestDuplicatedOriginals } =
        useMemo(() => {
            if (
                !layout ||
                selectedIdx < 0 ||
                !autoLayout ||
                !nestingSettings.enabled
            ) {
                return {
                    nestShells: [],
                    nestDuplicates: [] as DuplicateNode[],
                    nestDuplicatedOriginals: new Set<number>(),
                };
            }
            const shells = computeShellLayout(
                layout,
                selectedIdx,
                nestingSettings.parentDepth,
                80,
                30,
            );
            // Duplicates are now handled by decomposition clones inside patterns;
            // NodeLayer no longer renders separate duplicate elements.
            return {
                nestShells: shells,
                nestDuplicates: [] as DuplicateNode[],
                nestDuplicatedOriginals: new Set<number>(),
            };
        }, [
            layout,
            selectedIdx,
            autoLayout,
            nestingSettings.enabled,
            nestingSettings.parentDepth,
        ]);

    // Build layout when snapshot changes
    useEffect(() => {
        if (!snapshot) {
            layoutRef.current = null;
            setLayout(null);
            return;
        }
        const newLayout = buildLayout(snapshot);
        layoutRef.current = newLayout;
        setLayout(newLayout);
        // Eagerly capture force-directed equilibrium as immutable base positions.
        // These serve as the ground truth that active transforms (focused layout)
        // are layered on top of each frame.
        const base = new Map<number, { x: number; y: number; z: number }>();
        for (const n of newLayout.nodes) {
            base.set(n.index, { x: n.tx, y: n.ty, z: n.tz });
        }
        basePositionsRef.current = base;
        camera.resetForLayout(newLayout.nodes.length, newLayout.center);
        interRef.current.selectedIdx = -1;
        interRef.current.hoverIdx = -1;
        setSelectedIdx(-1);
    }, [snapshot, camera, setSelectedIdx]);

    // ── Focused layout ──
    const focusedOffsetsRef = useRef<FocusedLayoutOffsets | null>(null);
    useEffect(() => {
        const curLayout = layoutRef.current;
        if (!curLayout) return;

        if (selectedIdx >= 0 && autoLayout) {
            let layoutResult: FocusedLayoutOffsets | null;
            if (currentSearchPath?.root) {
                layoutResult = computeSearchPathLayout(
                    curLayout,
                    currentSearchPath,
                    selectedIdx,
                );
            } else {
                layoutResult = computeFocusedLayout(curLayout, selectedIdx);
            }
            focusedOffsetsRef.current = layoutResult;

            // In dup=off nesting, children are shown inside their expanded parent,
            // so the camera should target the parent's position instead.
            const focusIdx = getCameraFocusIdx(
                curLayout,
                selectedIdx,
                nestingSettings,
                currentSearchPath,
            );
            const focusNode = curLayout.nodeMap.get(focusIdx);
            if (focusNode) {
                camera.focusOn([focusNode.tx, focusNode.ty, focusNode.tz]);
            }
        } else if (selectedIdx >= 0) {
            focusedOffsetsRef.current = null;
            const focusIdx = getCameraFocusIdx(
                curLayout,
                selectedIdx,
                nestingSettings,
                currentSearchPath,
            );
            const focusNode = curLayout.nodeMap.get(focusIdx);
            if (focusNode) {
                camera.focusOn([focusNode.tx, focusNode.ty, focusNode.tz]);
            }
        } else {
            focusedOffsetsRef.current = null;
        }
    }, [
        selectedIdx,
        camera,
        currentSearchPath,
        autoLayout,
        nestingSettings.enabled,
        nestingSettings.duplicateMode,
    ]);

    // Focus camera on primary node when search step changes
    useEffect(() => {
        const curLayout = layoutRef.current;
        if (!curLayout || !currentEvent) return;

        const primaryNode = getPrimaryNode(
            currentEvent.transition,
            currentEvent.location,
        );
        if (primaryNode != null) {
            const node = curLayout.nodeMap.get(primaryNode);
            if (node) {
                interRef.current.selectedIdx = primaryNode;
                setSelectedIdx(primaryNode);
            }
        }
    }, [stepKey, camera, setSelectedIdx]);

    // Register WebGPU overlay renderer
    useOverlayRenderer(
        containerRef,
        nodeLayerRef,
        layoutRef,
        camera,
        interRef,
        vizState,
        setSelectedIdx,
        focusedOffsetsRef,
        basePositionsRef,
        nestingSettingsRef,
        autoLayoutRef,
    );

    // Handle focus from NodeInfoPanel links
    const handleFocusNode = useCallback(
        (nodeIndex: number) => {
            const curLayout = layoutRef.current;
            if (!curLayout) return;
            const target = curLayout.nodeMap.get(nodeIndex);
            if (target) {
                setSelectedIdx(nodeIndex);
            }
        },
        [setSelectedIdx],
    );

    // Empty state
    if (!snapshot) {
        return (
            <div class="hypergraph-container hg-dom-mode">
                <div class="hypergraph-empty">
                    <span>No hypergraph data found in current log</span>
                    <div class="hg-hint">
                        To visualize the graph, call{" "}
                        <code>graph.emit_graph_snapshot()</code> in your Rust
                        test after building the graph. This emits a structured
                        tracing event that the log viewer can render.
                    </div>
                </div>
            </div>
        );
    }

    const maxWidth = layout?.maxWidth ?? 1;
    const selectedNode =
        selectedIdx >= 0 ? layout?.nodeMap.get(selectedIdx) : null;

    return (
        <div ref={containerRef} class="hypergraph-container hg-dom-mode">
            {/* DOM node layer */}
            <div ref={nodeLayerRef} class="hg-node-layer">
                {layout && (
                    <NodeLayer
                        nodes={layout.nodes}
                        maxWidth={maxWidth}
                        vizState={vizState}
                        shells={nestShells}
                        duplicates={nestDuplicates}
                        duplicatedOriginals={nestDuplicatedOriginals}
                    />
                )}
            </div>
            {/* Info overlay */}
            <GraphInfoOverlay snapshot={snapshot} />
            {/* Selected Node Info Panel */}
            {selectedNode && layout && (
                <NodeInfoPanel
                    node={selectedNode}
                    layout={layout}
                    vizState={vizState}
                    onFocusNode={handleFocusNode}
                />
            )}
            {/* Tooltip */}
            {/*<NodeTooltip tooltip={tooltip} />*/}
            {/* Log-viewer-specific panels injected via render-prop */}
            {renderChildren?.({
                handleFocusNode,
                nestingSettings,
                setNestingSettings,
            })}
        </div>
    );
}
