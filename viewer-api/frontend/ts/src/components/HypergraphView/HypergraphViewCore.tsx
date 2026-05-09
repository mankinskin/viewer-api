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
    type LayoutNode,
} from "./layout";
import type {
    HypergraphViewProps,
    NestingSettings,
    DuplicateNode,
} from "./types";
import { getCameraFocusIdx } from "./focus";

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
    /**
     * Optional custom renderer for each node's DOM content.
     * When provided, replaces the default atom/compound badge rendering.
     * The outer `.hg-node` div (used by the WebGPU overlay for 3D positioning) is
     * still emitted — only the inner content changes.
     */
    renderNode?: (node: LayoutNode) => ComponentChildren;
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
        renderNode,
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
                        renderNode={renderNode}
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
