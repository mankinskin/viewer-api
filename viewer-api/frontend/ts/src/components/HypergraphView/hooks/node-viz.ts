import type { VisualizationState } from './useVisualizationState';

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

    const suppressParentHighlight =
        matchedNode != null && nodeIndex !== matchedNode;

    const searchPathStartNode = searchPath?.start_node?.index ?? -1;
    const searchPathRootNode = searchPath?.root?.index ?? -1;
    const isSearchPathStart = nodeIndex === searchPathStartNode;
    const isSearchPathRoot = nodeIndex === searchPathRootNode;
    const isInStartPath =
        isSearchPathStart ||
        isSearchPathRoot ||
        (searchPath?.start_path.some((node) => node.index === nodeIndex) ??
            false);
    const isSearchPathEnd =
        searchPath?.end_path.some((node) => node.index === nodeIndex) ??
        false;
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
    const isQueryToken = viz.queryTokens.has(nodeIndex);
    const isActiveQueryToken = nodeIndex === viz.activeQueryToken;
    const isInSearchPath = isInStartPath || isSearchPathEnd;
    const isDimmed =
        hasVizState &&
        !involvedNodes.has(nodeIndex) &&
        !isInSearchPath &&
        !isQueryToken;

    return [
        isStart && !suppressParentHighlight && 'viz-start',
        isSelected && !suppressParentHighlight && 'viz-selected',
        isRoot && !viz.rootTentative && !suppressParentHighlight && 'viz-root',
        isRoot &&
            viz.rootTentative &&
            !suppressParentHighlight &&
            'viz-root-tentative',
        isCandidateParent && !suppressParentHighlight && 'viz-candidate-parent',
        isCandidateChild && 'viz-candidate-child',
        isMatched && 'viz-matched',
        isMismatched && 'viz-mismatched',
        isPath && !suppressParentHighlight && 'viz-path',
        isCompleted && 'viz-completed',
        isPendingParent && !suppressParentHighlight && 'viz-pending-parent',
        isPendingChild && 'viz-pending-child',
        isSearchPathStart && 'viz-sp-start',
        isSearchPathRoot && 'viz-sp-root',
        isInStartPath && 'viz-sp-start-path',
        isSearchPathEnd && 'viz-sp-end-path',
        isQueryToken && 'viz-query-token',
        isActiveQueryToken && 'viz-query-active',
        nodeIndex === viz.splitSource && 'viz-split-source',
        nodeIndex === viz.splitLeft && 'viz-split-left',
        nodeIndex === viz.splitRight && 'viz-split-right',
        nodeIndex === viz.joinLeft && 'viz-join-left',
        nodeIndex === viz.joinRight && 'viz-join-right',
        nodeIndex === viz.joinResult && 'viz-join-result',
        nodeIndex === viz.newPatternParent && 'viz-new-pattern',
        viz.newPatternChildren.has(nodeIndex) && 'viz-new-pattern-child',
        nodeIndex === viz.newRoot && 'viz-new-root',
        isDimmed && 'viz-dimmed',
    ]
        .filter(Boolean)
        .join(' ');
}

/**
 * Get the active visualization states for a specific node.
 */
export function getNodeVizStates(
    nodeIndex: number,
    viz: VisualizationState,
): string[] {
    const states: string[] = [];
    if (nodeIndex === viz.startNode) states.push('start');
    if (nodeIndex === viz.selectedNode) states.push('selected');
    if (nodeIndex === viz.rootNode) states.push('root');
    if (nodeIndex === viz.candidateParent) states.push('candidate-parent');
    if (nodeIndex === viz.candidateChild) states.push('candidate-child');
    if (nodeIndex === viz.matchedNode) states.push('matched');
    if (nodeIndex === viz.mismatchedNode) states.push('mismatched');
    if (viz.tracePath.has(nodeIndex)) states.push('path');
    if (viz.completedNodes.has(nodeIndex)) states.push('completed');
    if (viz.pendingParents.has(nodeIndex)) states.push('pending-parent');
    if (viz.pendingChildren.has(nodeIndex)) states.push('pending-child');
    if (viz.searchPath) {
        if (viz.searchPath.start_node?.index === nodeIndex) {
            states.push('sp-start');
        }
        if (viz.searchPath.root?.index === nodeIndex) states.push('sp-root');
        if (viz.searchPath.start_path.some((node) => node.index === nodeIndex)) {
            states.push('sp-start-path');
        }
        if (viz.searchPath.end_path.some((node) => node.index === nodeIndex)) {
            states.push('sp-end-path');
        }
    }
    if (viz.queryTokens.has(nodeIndex)) states.push('query-token');
    if (nodeIndex === viz.activeQueryToken) states.push('query-active');
    if (nodeIndex === viz.splitSource) states.push('split-source');
    if (nodeIndex === viz.splitLeft) states.push('split-left');
    if (nodeIndex === viz.splitRight) states.push('split-right');
    if (nodeIndex === viz.joinLeft) states.push('join-left');
    if (nodeIndex === viz.joinRight) states.push('join-right');
    if (nodeIndex === viz.joinResult) states.push('join-result');
    if (nodeIndex === viz.newPatternParent) states.push('new-pattern');
    if (viz.newPatternChildren.has(nodeIndex)) {
        states.push('new-pattern-child');
    }
    if (nodeIndex === viz.newRoot) states.push('new-root');
    return states;
}