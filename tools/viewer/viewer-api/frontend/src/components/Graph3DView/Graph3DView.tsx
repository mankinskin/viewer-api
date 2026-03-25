/**
 * Graph3DView — self-contained 3D graph visualization component.
 *
 * Renders an interactive force-directed or hierarchical graph using WebGPU
 * for edges and a DOM node layer for labels.
 */
import { useRef, useMemo, useEffect, useCallback } from 'preact/hooks';
import type { Graph3DViewProps, Graph3DNode, InternalLayout } from './types';
import { buildForceLayout } from './layout/forceDirected';
import { buildHierarchicalLayout } from './layout/hierarchical';
import { useCamera } from './hooks/useCamera';
import { useMouseInteraction } from './hooks/useMouseInteraction';
import { useTouchInteraction } from './hooks/useTouchInteraction';
import { useOverlayRenderer } from './hooks/useOverlayRenderer';
import { NodeLayer } from './components/NodeLayer';

export function Graph3DView({
    nodes,
    edges,
    onNodeClick,
    selectedNodeId,
    layoutMode = 'force',
}: Graph3DViewProps) {
    const containerRef = useRef<HTMLDivElement | null>(null);
    const nodeLayerRef = useRef<HTMLDivElement | null>(null);

    // Build layout when inputs change
    const layout = useMemo<InternalLayout>(() => {
        return layoutMode === 'hierarchical'
            ? buildHierarchicalLayout(nodes, edges)
            : buildForceLayout(nodes, edges);
    }, [nodes, edges, layoutMode]);

    // Keep a stable ref so render callbacks always see the latest layout
    const layoutRef = useRef<InternalLayout | null>(null);
    layoutRef.current = layout;

    const camera = useCamera();

    // Reset camera orbit whenever the layout changes
    useEffect(() => {
        camera.resetForLayout(layout.nodes.length, layout.center);
    }, [layout]);

    const { setSelectedIdx, interRef } = useMouseInteraction(
        containerRef,
        layoutRef,
        camera,
    );

    useTouchInteraction(containerRef, layoutRef, camera, interRef, setSelectedIdx);

    // Sync the `selectedNodeId` prop into internal selectedIdx
    useEffect(() => {
        if (selectedNodeId == null) {
            setSelectedIdx(-1);
            return;
        }
        const idx = layout.nodeIdMap.get(selectedNodeId) ?? -1;
        setSelectedIdx(idx);
    }, [selectedNodeId, layout]);

    const handleNodeClick = useCallback(
        (idx: number) => {
            if (!onNodeClick || idx < 0) return;
            const n = layout.nodes[idx];
            if (!n) return;
            const pub: Graph3DNode = {
                id: n.id,
                label: n.label,
                color: n.color,
                size: n.size,
                data: n.data,
            };
            onNodeClick(pub);
        },
        [layout, onNodeClick],
    );

    useOverlayRenderer(containerRef, nodeLayerRef, layoutRef, camera, interRef, handleNodeClick);

    const maxSize = useMemo(
        () => Math.max(1, ...layout.nodes.map((n) => n.size ?? 1)),
        [layout],
    );

    return (
        <div
            ref={containerRef}
            style={{ position: 'relative', width: '100%', height: '100%', overflow: 'hidden' }}
            onContextMenu={(e) => e.preventDefault()}
        >
            <div
                ref={nodeLayerRef}
                style={{ position: 'absolute', inset: 0, pointerEvents: 'none' }}
            >
                <NodeLayer nodes={layout.nodes} maxSize={maxSize} />
            </div>
        </div>
    );
}
