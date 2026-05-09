/**
 * Reactive DOM → GPU element bridge.
 *
 * Replaces the old full-re-scan-on-dirty-flag approach with:
 *   - Incremental MutationObserver processing (reclassify single elements)
 *   - IntersectionObserver for cheap visibility tracking
 *   - Dynamic Float32Array growth (no MAX_ELEMENTS limit)
 *   - Rect measurement batching (only measure stale + visible elements)
 *   - Full re-scan only on large DOM changes or explicit invalidation
 *
 * Selector metadata is injected at construction time from AppSchema so the
 * scanner is not coupled to any specific application's element registry.
 */

import { ELEM_FLOATS, type SelectorEntry } from './element-types';
import {
    compactDead,
    markAllRectsStale,
    measureStaleRects,
    rebuildData,
} from './element-scanner-buffer';
import {
    addTrackedTree,
    fullRescan,
    reclassifyElement,
    removeTrackedTree,
    type TrackedElement,
} from './element-scanner-tracking';
import { createElementScannerObservers } from './element-scanner-observers';

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/** Initial Float32Array capacity (in number of elements). */
const INITIAL_CAPACITY = 128;

/**
 * If a single MutationObserver callback delivers more than this many added
 * nodes, fall back to a full re-scan instead of per-node diffing.
 */
const BATCH_THRESHOLD = 50;

// ---------------------------------------------------------------------------
// ElementScanner
// ---------------------------------------------------------------------------

export class ElementScanner {
    /** Current element data ready for GPU upload. */
    private _data: Float32Array;
    /** Number of elements currently packed. */
    private _count = 0;
    /** Capacity of the current Float32Array (in elements). */
    private _capacity: number;

    /** Identity-tracked elements. */
    private _tracked: TrackedElement[] = [];
    /** Quick lookup: DOM element → tracked entry. */
    private _elementMap = new Map<Element, TrackedElement>();

    /** IntersectionObserver for viewport visibility. */
    private _io: IntersectionObserver | null = null;
    /** ResizeObserver for per-element size change detection. */
    private _ro: ResizeObserver | null = null;
    /** MutationObserver for incremental DOM tracking. */
    private _mo: MutationObserver | null = null;

    /** Dirty flags. */
    private _fullRescanPending = true;
    private _rectsStale = false;
    /**
     * Set when element classification (kind/hue) changes without a rect
     * change.  Forces `_rebuildData()` even if no rects moved.
     */
    private _dataStale = false;

    /** Accumulated scroll deltas since last frame (pixels, screen-space). */
    private _scrollDx = 0;
    private _scrollDy = 0;
    /** True when a full re-scan was just performed (view change). */
    private _justDidFullRescan = false;

    /** Event listeners (stored for cleanup). */
    private readonly _onScroll = (e: Event) => {
        this._markAllRectsStale();
        const tgt = e.target as Element | Document;
        const el = (tgt === document || tgt === document.documentElement)
            ? document.documentElement : tgt as Element;
        const key = el;
        const prev = this._scrollPositions.get(key);
        const sx = el.scrollLeft;
        const sy = el.scrollTop;
        if (prev) {
            this._scrollDx -= sx - prev.x;
            this._scrollDy -= sy - prev.y;
            prev.x = sx;
            prev.y = sy;
        } else {
            this._scrollPositions.set(key, { x: sx, y: sy });
        }
    };
    private readonly _onResize = () => { this._markAllRectsStale(); };

    /** Tracked scroll positions per scrollable container. */
    private _scrollPositions = new Map<Element, { x: number; y: number }>();

    // Schema-injected selector data
    private readonly _selectorMeta: ReadonlyArray<SelectorEntry>;
    private readonly _priorityIndices: ReadonlySet<number>;
    private readonly _scanOrder: readonly number[];

    /**
     * @param selectorMeta     Pre-computed selector entries from AppSchema.
     * @param priorityIndices  Indices of selectors that must always get buffer slots.
     */
    constructor(
        selectorMeta: ReadonlyArray<SelectorEntry>,
        priorityIndices: ReadonlySet<number>,
    ) {
        this._selectorMeta = selectorMeta;
        this._priorityIndices = priorityIndices;

        // Pre-compute scan order: priority selectors first
        const order: number[] = [];
        for (const si of priorityIndices) order.push(si);
        for (let i = 0; i < selectorMeta.length; i++) {
            if (!priorityIndices.has(i)) order.push(i);
        }
        this._scanOrder = order;

        this._capacity = INITIAL_CAPACITY;
        this._data = new Float32Array(INITIAL_CAPACITY * ELEM_FLOATS);
    }

    // --- Public getters ----------------------------------------------------

    /** Float32Array ready for GPU upload. */
    get data(): Float32Array { return this._data; }

    /** Number of elements currently written to `data`. */
    get count(): number { return this._count; }

    /** Capacity of the internal buffer (in elements). */
    get capacity(): number { return this._capacity; }

    /** Reusable scroll delta object (avoids per-frame allocation). */
    private readonly _scrollDeltaResult = { dx: 0, dy: 0 };

    /** Consume accumulated scroll deltas (resets to 0 after reading). */
    consumeScrollDelta(): { dx: number; dy: number } {
        this._scrollDeltaResult.dx = this._scrollDx;
        this._scrollDeltaResult.dy = this._scrollDy;
        this._scrollDx = 0;
        this._scrollDy = 0;
        return this._scrollDeltaResult;
    }

    /** True if a full re-scan just completed (view change / invalidateAll). */
    get didFullRescan(): boolean { return this._justDidFullRescan; }

    // --- Lifecycle ----------------------------------------------------------

    /** Start observing DOM. Call once after DOM is ready. */
    start(): void {
        const observers = createElementScannerObservers({
            elementMap: this._elementMap,
            onRectsStale: () => {
                this._rectsStale = true;
            },
            onProcessMutations: (records) => {
                this._processMutations(records);
            },
        });
        this._io = observers.io;
        this._ro = observers.ro;
        this._mo = observers.mo;
        this._mo.observe(document.body, {
            childList: true,
            subtree: true,
            attributes: true,
            attributeFilter: ['class', 'style'],
        });

        // Scroll + resize → mark rects stale (not a full re-scan)
        window.addEventListener('scroll', this._onScroll, true);
        window.addEventListener('resize', this._onResize);

        // Initial full scan
        this._fullRescanPending = true;
    }

    /** Stop observing and release resources. */
    destroy(): void {
        this._io?.disconnect();
        this._io = null;
        this._ro?.disconnect();
        this._ro = null;
        this._mo?.disconnect();
        this._mo = null;
        window.removeEventListener('scroll', this._onScroll, true);
        window.removeEventListener('resize', this._onResize);
        this._tracked = [];
        this._elementMap.clear();
        this._scrollPositions.clear();
        this._count = 0;
    }

    /** Force a full re-scan (tab change, etc.). */
    invalidateAll(): void {
        this._fullRescanPending = true;
    }

    /**
     * Per-frame update: measure stale rects, compact dead elements,
     * and rebuild the Float32Array.  Returns `true` if data changed.
     */
    updateFrame(): boolean {
        let changed = false;
        this._justDidFullRescan = false;

        // 1. Full re-scan if pending
        if (this._fullRescanPending) {
            this._fullRescan();
            this._fullRescanPending = false;
            this._justDidFullRescan = true;
            this._scrollPositions.clear();
            this._scrollDx = 0;
            this._scrollDy = 0;
            changed = true;
        }

        // 2. Compact: remove GC'd elements
        const compacted = this._compactDead();
        if (compacted) changed = true;

        // 3. Measure stale rects for visible elements
        if (this._rectsStale || changed) {
            const measured = this._measureStaleRects();
            if (measured) changed = true;
            this._rectsStale = false;
        }

        // 4. Rebuild Float32Array if anything changed
        if (this._dataStale) {
            changed = true;
            this._dataStale = false;
        }
        if (changed) {
            this._rebuildData();
        }

        return changed;
    }

    // --- Internal: mutation processing -------------------------------------

    private _processMutations(records: MutationRecord[]): void {
        let totalAdded = 0;
        let hasChildListMutation = false;

        for (const rec of records) {
            if (rec.type === 'childList') {
                hasChildListMutation = true;
                for (let i = 0; i < rec.removedNodes.length; i++) {
                    const node = rec.removedNodes[i]!;
                    if (node.nodeType === Node.ELEMENT_NODE) {
                        this._removeTrackedTree(node as Element);
                    }
                }
                totalAdded += rec.addedNodes.length;
            } else if (rec.type === 'attributes') {
                const target = rec.target;
                if (target.nodeType === Node.ELEMENT_NODE) {
                    if (rec.attributeName === 'class') {
                        this._reclassifyElement(target as Element);
                    }
                    const tracked = this._elementMap.get(target as Element);
                    if (tracked) {
                        tracked.rectStale = true;
                        this._rectsStale = true;
                    }
                }
            }
        }

        if (totalAdded > BATCH_THRESHOLD) {
            this._fullRescanPending = true;
        } else if (totalAdded > 0) {
            for (const rec of records) {
                if (rec.type === 'childList') {
                    for (let i = 0; i < rec.addedNodes.length; i++) {
                        const node = rec.addedNodes[i]!;
                        if (node.nodeType === Node.ELEMENT_NODE) {
                            this._addTrackedTree(node as Element);
                        }
                    }
                }
            }
        }

        if (hasChildListMutation) {
            this._markAllRectsStale();
        }
    }

    /** Recursively add an element and its descendants to the tracked set. */
    private _addTrackedTree(root: Element): void {
        if (addTrackedTree({
            tracked: this._tracked,
            elementMap: this._elementMap,
            io: this._io,
            ro: this._ro,
            selectorMeta: this._selectorMeta,
            scanOrder: this._scanOrder,
        }, root)) {
            this._rectsStale = true;
        }
    }

    /** Recursively remove an element and its descendants from tracked set. */
    private _removeTrackedTree(root: Element): void {
        removeTrackedTree(this._elementMap, this._io, this._ro, root);
    }

    /** Reclassify an element after a class change. */
    private _reclassifyElement(el: Element): void {
        const flags = reclassifyElement({
            tracked: this._tracked,
            elementMap: this._elementMap,
            io: this._io,
            ro: this._ro,
            selectorMeta: this._selectorMeta,
            scanOrder: this._scanOrder,
        }, el);
        if (flags.rectsStale) {
            this._rectsStale = true;
        }
        if (flags.dataStale) {
            this._dataStale = true;
        }
    }

    // --- Internal: full re-scan --------------------------------------------

    private _fullRescan(): void {
        const rescan = fullRescan({
            tracked: this._tracked,
            io: this._io,
            ro: this._ro,
            selectorMeta: this._selectorMeta,
            priorityIndices: this._priorityIndices,
        });
        this._tracked = rescan.tracked;
        this._elementMap = rescan.elementMap;
    }

    // --- Internal: maintenance ---------------------------------------------

    /** Remove GC'd or untracked elements. Returns true if any were removed. */
    private _compactDead(): boolean {
        const before = this._tracked.length;
        this._tracked = compactDead(this._tracked, this._elementMap);
        return this._tracked.length !== before;
    }

    private _markAllRectsStale(): void {
        markAllRectsStale(this._tracked);
        this._rectsStale = true;
    }

    /** Measure rects for stale + visible elements. Returns true if any changed. */
    private _measureStaleRects(): boolean {
        return measureStaleRects(this._tracked);
    }

    // --- Internal: rebuild Float32Array ------------------------------------

    private _rebuildData(): void {
        const rebuilt = rebuildData({
            tracked: this._tracked,
            priorityIndices: this._priorityIndices,
            capacity: this._capacity,
            data: this._data,
        });
        this._data = rebuilt.data;
        this._count = rebuilt.count;
        this._capacity = rebuilt.capacity;
    }
}
