import type { SelectorEntry } from './element-types';

export interface TrackedElement {
    ref: WeakRef<Element>;
    selectorIdx: number;
    kind: number;
    hue: number;
    rect: DOMRect | null;
    visible: boolean;
    rectStale: boolean;
}

interface TrackingOptions {
    tracked: TrackedElement[];
    elementMap: Map<Element, TrackedElement>;
    io: IntersectionObserver | null;
    ro: ResizeObserver | null;
    selectorMeta: ReadonlyArray<SelectorEntry>;
    scanOrder: readonly number[];
}

export function tryTrackElement(options: TrackingOptions, el: Element): boolean {
    if (options.elementMap.has(el)) return false;

    for (const selectorIdx of options.scanOrder) {
        const meta = options.selectorMeta[selectorIdx]!;
        try {
            if (el.matches(meta.sel)) {
                const tracked = createTrackedElement(el, selectorIdx, meta);
                options.tracked.push(tracked);
                options.elementMap.set(el, tracked);
                options.io?.observe(el);
                options.ro?.observe(el);
                return true;
            }
        } catch {
            // Invalid selector - skip.
        }
    }

    return false;
}

export function addTrackedTree(options: TrackingOptions, root: Element): boolean {
    let added = tryTrackElement(options, root);
    const children = root.querySelectorAll('*');
    for (let i = 0; i < children.length; i++) {
        added = tryTrackElement(options, children[i]!) || added;
    }
    return added;
}

export function untrackElement(
    elementMap: Map<Element, TrackedElement>,
    io: IntersectionObserver | null,
    ro: ResizeObserver | null,
    el: Element,
): void {
    const tracked = elementMap.get(el);
    if (!tracked) return;

    elementMap.delete(el);
    io?.unobserve(el);
    ro?.unobserve(el);
}

export function removeTrackedTree(
    elementMap: Map<Element, TrackedElement>,
    io: IntersectionObserver | null,
    ro: ResizeObserver | null,
    root: Element,
): void {
    untrackElement(elementMap, io, ro, root);
    const children = root.querySelectorAll('*');
    for (let i = 0; i < children.length; i++) {
        untrackElement(elementMap, io, ro, children[i]!);
    }
}

export function reclassifyElement(options: TrackingOptions, el: Element): {
    rectsStale: boolean;
    dataStale: boolean;
} {
    const existing = options.elementMap.get(el);
    let dataStale = false;

    for (const selectorIdx of options.scanOrder) {
        const meta = options.selectorMeta[selectorIdx]!;
        try {
            if (el.matches(meta.sel)) {
                if (existing) {
                    if (existing.kind !== meta.kind || existing.hue !== meta.hue) {
                        dataStale = true;
                    }
                    existing.selectorIdx = selectorIdx;
                    existing.kind = meta.kind;
                    existing.hue = meta.hue;
                    existing.rectStale = true;
                } else {
                    const tracked = createTrackedElement(el, selectorIdx, meta);
                    options.tracked.push(tracked);
                    options.elementMap.set(el, tracked);
                    options.io?.observe(el);
                    options.ro?.observe(el);
                }
                return { rectsStale: true, dataStale };
            }
        } catch {
            // Invalid selector - skip.
        }
    }

    if (existing) {
        options.elementMap.delete(el);
        options.io?.unobserve(el);
        options.ro?.unobserve(el);
        return { rectsStale: false, dataStale };
    }

    return { rectsStale: false, dataStale };
}

export function fullRescan(options: {
    tracked: TrackedElement[];
    io: IntersectionObserver | null;
    ro: ResizeObserver | null;
    selectorMeta: ReadonlyArray<SelectorEntry>;
    priorityIndices: ReadonlySet<number>;
}): {
    tracked: TrackedElement[];
    elementMap: Map<Element, TrackedElement>;
} {
    for (const tracked of options.tracked) {
        const el = tracked.ref.deref();
        if (el) {
            options.io?.unobserve(el);
            options.ro?.unobserve(el);
        }
    }

    const tracked: TrackedElement[] = [];
    const elementMap = new Map<Element, TrackedElement>();

    for (const selectorIdx of options.priorityIndices) {
        const meta = options.selectorMeta[selectorIdx];
        if (meta) {
            queryAndTrack({ tracked, elementMap, io: options.io, ro: options.ro }, selectorIdx, meta);
        }
    }

    for (let selectorIdx = 0; selectorIdx < options.selectorMeta.length; selectorIdx++) {
        if (options.priorityIndices.has(selectorIdx)) continue;
        const meta = options.selectorMeta[selectorIdx]!;
        queryAndTrack({ tracked, elementMap, io: options.io, ro: options.ro }, selectorIdx, meta);
    }

    return { tracked, elementMap };
}

function createTrackedElement(
    el: Element,
    selectorIdx: number,
    meta: SelectorEntry,
): TrackedElement {
    return {
        ref: new WeakRef(el),
        selectorIdx,
        kind: meta.kind,
        hue: meta.hue,
        rect: null,
        visible: true,
        rectStale: true,
    };
}

function queryAndTrack(
    options: {
        tracked: TrackedElement[];
        elementMap: Map<Element, TrackedElement>;
        io: IntersectionObserver | null;
        ro: ResizeObserver | null;
    },
    selectorIdx: number,
    meta: SelectorEntry,
): void {
    const elements = document.querySelectorAll(meta.sel);
    for (let i = 0; i < elements.length; i++) {
        const el = elements[i]!;
        if (options.elementMap.has(el)) continue;

        const tracked = createTrackedElement(el, selectorIdx, meta);
        options.tracked.push(tracked);
        options.elementMap.set(el, tracked);
        options.io?.observe(el);
        options.ro?.observe(el);
    }
}