/**
 * DecompositionManager — imperatively reparents child DOM nodes into
 * decomposition rows under their expanded parent element.
 *
 * Manages expand/collapse state and provides information about which
 * nodes are currently reparented so that the node positioner can skip
 * 3D transforms for them and the edge builder can hide internal edges.
 */
import type { GraphLayout } from '../layout';
import { getDecompositionPatterns } from '../layout';
import { ROW_COLORS } from '../gpu/constants';
import { edgePairKey } from '../utils/math';

// ── Types ──

export interface ExpandedNodeState {
    parentEl: HTMLDivElement;
    container: HTMLDivElement;
    children: { el: HTMLDivElement }[];
    /** Clone elements created for repeated children or clone mode. */
    clones: HTMLDivElement[];
}

// ── Manager ──

export class DecompositionManager {
    private expandedNodes = new Map<number, ExpandedNodeState>();
    private nodeElMap = new Map<number, HTMLDivElement>();
    private lastExpandedKeyStr = '';
    /** Pending collapse animations (parentIdx → cleanup timer). */
    private collapseTimers = new Map<number, number>();
    /** Nodes mid-collapse — empty container still animating CSS shrink. */
    private collapsingContainers = new Map<number, { parentEl: HTMLDivElement; container: HTMLDivElement }>;
    /** Duration for expand/collapse CSS transitions (ms). */
    private static readonly TRANSITION_MS = 300;

    constructor(
        private layout: GraphLayout,
        private nodeLayer: HTMLDivElement,
        private onSelectNode?: (idx: number) => void,
    ) {
        this.refreshNodeElMap();
    }

    // ── Public API ──

    /** Get the map of node index → DOM element. */
    getNodeElMap(): Map<number, HTMLDivElement> {
        return this.nodeElMap;
    }

    /** Get the set of currently-expanded node indices. */
    getExpandedNodes(): Map<number, ExpandedNodeState> {
        return this.expandedNodes;
    }

    /** Get indices of child nodes cloned inside a decomposition row. */
    getClonedChildIndices(): Set<number> {
        const set = new Set<number>();
        for (const [, state] of this.expandedNodes) {
            for (const clone of state.clones) {
                const idx = clone.getAttribute('data-node-idx');
                if (idx != null) set.add(Number(idx));
            }
        }
        return set;
    }

    /** Get mapping: cloned child index → expanded parent index */
    getChildParentMap(): Map<number, number> {
        const map = new Map<number, number>();
        for (const [expIdx, state] of this.expandedNodes) {
            for (const clone of state.clones) {
                const idx = clone.getAttribute('data-node-idx');
                if (idx != null) map.set(Number(idx), expIdx);
            }
        }
        return map;
    }

    /** Edge pair keys that should be hidden (parent↔child inside decomp). */
    getHiddenDecompEdgeKeys(): Set<number> {
        const keys = new Set<number>();
        for (const [expIdx, state] of this.expandedNodes) {
            for (const clone of state.clones) {
                const idx = clone.getAttribute('data-node-idx');
                if (idx != null) {
                    const ci = Number(idx);
                    keys.add(edgePairKey(expIdx, ci));
                    keys.add(edgePairKey(ci, expIdx));
                }
            }
        }
        return keys;
    }

    /**
     * Synchronise the set of expanded nodes with the desired set.
     * Collapses removed, expands added, reorders if changed.
     *
     */
    update(desiredExpanded: Set<number>): void {
        const desiredKeyStr = [...desiredExpanded].sort((a, b) => a - b).join(',');
        if (desiredKeyStr === this.lastExpandedKeyStr) return;

        // Collapse nodes no longer desired
        for (const idx of [...this.expandedNodes.keys()]) {
            if (!desiredExpanded.has(idx)) this.collapseNode(idx);
        }
        // Expand desired nodes that aren't already expanded
        for (const idx of desiredExpanded) {
            if (!this.expandedNodes.has(idx)) this.expandNode(idx);
        }
        this.reorderNodeLayer();
        this.lastExpandedKeyStr = desiredKeyStr;
    }

    /** Collapse everything immediately (no animation). Used on cleanup/unmount. */
    collapseAll(): void {
        // Force-cancel pending collapse container animations
        for (const [idx, timer] of this.collapseTimers) {
            clearTimeout(timer);
            const entry = this.collapsingContainers.get(idx);
            if (entry) {
                entry.parentEl.classList.remove('hg-collapsing');
                entry.container.remove();
                const ep = entry.parentEl as any;
                if (ep.__parentDown) entry.parentEl.removeEventListener('mousedown', ep.__parentDown);
                if (ep.__parentUp) entry.parentEl.removeEventListener('mouseup', ep.__parentUp);
            }
        }
        this.collapseTimers.clear();
        this.collapsingContainers.clear();

        const hadExpanded = this.expandedNodes.size > 0;
        for (const idx of [...this.expandedNodes.keys()]) {
            this.forceCollapseNode(idx);
        }
        if (hadExpanded) this.reorderNodeLayer();
    }

    /** Re-scan the DOM for `data-node-idx` attributes. */
    refreshNodeElMap(): void {
        this.nodeElMap.clear();
        const divs = this.nodeLayer.children;
        for (let i = 0; i < divs.length; i++) {
            const el = divs[i] as HTMLDivElement;
            const idx = el.getAttribute('data-node-idx');
            if (idx != null) this.nodeElMap.set(Number(idx), el);
        }
        // Also include elements inside any expanded decomp container
        // (skip clones — they share data-node-idx but must not overwrite originals)
        for (const state of this.expandedNodes.values()) {
            const nested = state.container.querySelectorAll<HTMLDivElement>('[data-node-idx]');
            for (const el of nested) {
                if (el.hasAttribute('data-clone')) continue;
                const idx = el.getAttribute('data-node-idx');
                if (idx != null) this.nodeElMap.set(Number(idx), el);
            }
        }
    }

    // ── Internal ──

    private collapseNode(idx: number): void {
        const state = this.expandedNodes.get(idx);
        if (!state) return;

        // Cancel any pending collapse timer for this node
        const existingTimer = this.collapseTimers.get(idx);
        if (existingTimer != null) {
            clearTimeout(existingTimer);
            this.collapseTimers.delete(idx);
            // Also clean up old collapsing container
            const old = this.collapsingContainers.get(idx);
            if (old) { old.container.remove(); this.collapsingContainers.delete(idx); }
        }

        // Measure actual container height before any DOM changes so we can
        // pin max-height for a precise CSS transition (no invisible 400→real gap).
        const actualHeight = state.container.scrollHeight;

        // Clear container content so only an empty box shrinks (no visible rows/labels)
        state.container.innerHTML = '';

        // Pin max-height to the measured value and swap to collapsing class
        state.container.style.maxHeight = actualHeight + 'px';
        state.parentEl.classList.remove('hg-expanded');
        state.parentEl.classList.add('hg-collapsing');

        // Force layout read so browser registers the starting max-height,
        // then release to let the hg-collapsing CSS rule (max-height:0) take over.
        void state.container.offsetHeight;
        state.container.style.maxHeight = '';

        // After CSS transition, remove empty container + listeners
        const { parentEl, container } = state;
        const cleanup = () => {
            this.collapseTimers.delete(idx);
            parentEl.classList.remove('hg-collapsing');
            container.remove();
            const ep = parentEl as any;
            if (ep.__parentDown) parentEl.removeEventListener('mousedown', ep.__parentDown);
            if (ep.__parentUp) parentEl.removeEventListener('mouseup', ep.__parentUp);
            this.collapsingContainers.delete(idx);
        };

        const timer = window.setTimeout(cleanup, DecompositionManager.TRANSITION_MS);
        this.collapseTimers.set(idx, timer);
        this.collapsingContainers.set(idx, { parentEl, container });

        this.expandedNodes.delete(idx);
    }

    /** Immediately collapse without animation (used by collapseAll). */
    private forceCollapseNode(idx: number): void {
        const state = this.expandedNodes.get(idx);
        if (!state) return;
        state.parentEl.classList.remove('hg-expanded');
        state.container.remove();
        const ep = state.parentEl as any;
        if (ep.__parentDown) state.parentEl.removeEventListener('mousedown', ep.__parentDown);
        if (ep.__parentUp) state.parentEl.removeEventListener('mouseup', ep.__parentUp);
        this.expandedNodes.delete(idx);
    }

    private expandNode(idx: number): void {
        if (this.expandedNodes.has(idx)) return;

        // If mid-collapse, cancel the container animation
        const collapseTimer = this.collapseTimers.get(idx);
        if (collapseTimer != null) {
            clearTimeout(collapseTimer);
            this.collapseTimers.delete(idx);
            const entry = this.collapsingContainers.get(idx);
            if (entry) {
                entry.parentEl.classList.remove('hg-collapsing');
                entry.container.remove();
                const ep = entry.parentEl as any;
                if (ep.__parentDown) entry.parentEl.removeEventListener('mousedown', ep.__parentDown);
                if (ep.__parentUp) entry.parentEl.removeEventListener('mouseup', ep.__parentUp);
                this.collapsingContainers.delete(idx);
            }
        }

        const node = this.layout.nodeMap.get(idx);
        if (!node || node.isAtom) return;

        const patterns = getDecompositionPatterns(this.layout, idx);
        if (patterns.length === 0) return;

        // Refresh map in case Preact re-rendered
        this.refreshNodeElMap();

        const parentEl = this.nodeElMap.get(idx);
        if (!parentEl) return;

        // Create decomposition container
        const container = document.createElement('div');
        container.className = 'decomp-patterns';

        const clones: HTMLDivElement[] = [];

        for (let pi = 0; pi < patterns.length; pi++) {
            const pat = patterns[pi]!;
            const row = document.createElement('div');
            row.className = 'decomp-row';
            row.style.background = ROW_COLORS[pi % ROW_COLORS.length]!;

            const label = document.createElement('span');
            label.className = 'decomp-row-label';
            label.textContent = `P${pat.patternIdx}`;
            row.appendChild(label);

            const tokens = document.createElement('div');
            tokens.className = 'decomp-tokens';

            for (const child of pat.children) {
                const realEl = this.nodeElMap.get(child.index);
                if (!realEl) continue;

                // Always clone — never reparent real Preact-managed elements
                const clone = realEl.cloneNode(true) as HTMLDivElement;
                clone.setAttribute('data-clone', 'true');
                clones.push(clone);

                clone.classList.add('hg-decomp-child');
                clone.style.flex = `${child.width} 0 0%`;
                tokens.appendChild(clone);
            }

            row.appendChild(tokens);
            container.appendChild(row);
        }

        // DOM event handlers on parentEl
        // Only intercept clicks on decomp children (to select them).
        // Other clicks (header, row labels) bubble up to the container so
        // the normal ray-sphere drag logic can pick up the parent node.
        const onSelectNode = this.onSelectNode;
        const onParentMouseDown = (e: MouseEvent) => {
            if (e.button !== 0) return;
            const childTarget = (e.target as HTMLElement).closest('.hg-decomp-child');
            if (childTarget) {
                const cIdx = childTarget.getAttribute('data-node-idx');
                if (cIdx != null && onSelectNode) {
                    onSelectNode(Number(cIdx));
                }
                e.stopPropagation();
            }
            // Non-child clicks bubble → enables parent dragging
        };
        const onParentMouseUp = (e: MouseEvent) => {
            const childTarget = (e.target as HTMLElement).closest('.hg-decomp-child');
            if (childTarget) {
                e.stopPropagation();
            }
        };
        parentEl.addEventListener('mousedown', onParentMouseDown);
        parentEl.addEventListener('mouseup', onParentMouseUp);
        (parentEl as any).__parentDown = onParentMouseDown;
        (parentEl as any).__parentUp = onParentMouseUp;

        parentEl.appendChild(container);

        this.expandedNodes.set(idx, { parentEl, container, children: [], clones });

        // Apply expanded class immediately — full content visible right away
        parentEl.classList.add('hg-expanded');
    }

    private reorderNodeLayer(): void {
        const elByIdx = new Map<number, HTMLDivElement>();
        const divs = this.nodeLayer.children;
        for (let i = 0; i < divs.length; i++) {
            const el = divs[i] as HTMLDivElement;
            const idx = el.getAttribute('data-node-idx');
            if (idx != null) elByIdx.set(Number(idx), el);
        }
        for (const n of this.layout.nodes) {
            const el = elByIdx.get(n.index);
            if (el) this.nodeLayer.appendChild(el);
        }
    }
}
