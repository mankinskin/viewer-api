import { ELEM_FLOATS } from './element-types';
import type { TrackedElement } from './element-scanner-tracking';

export function compactDead(
    tracked: TrackedElement[],
    elementMap: Map<Element, TrackedElement>,
): TrackedElement[] {
    return tracked.filter((item) => {
        const el = item.ref.deref();
        return !!el && elementMap.has(el);
    });
}

export function markAllRectsStale(tracked: TrackedElement[]): void {
    for (const item of tracked) {
        item.rectStale = true;
    }
}

export function measureStaleRects(tracked: TrackedElement[]): boolean {
    let changed = false;
    const viewportHeight = window.innerHeight;

    for (const item of tracked) {
        if (!item.rectStale) continue;
        if (!item.visible && item.rect) continue;

        const el = item.ref.deref();
        if (!el) continue;

        const rect = el.getBoundingClientRect();
        item.rectStale = false;

        if (rect.width === 0 || rect.height === 0) {
            if (item.rect) {
                item.rect = null;
                changed = true;
            }
            continue;
        }
        if (rect.bottom < 0 || rect.top > viewportHeight) {
            if (item.rect) {
                item.rect = null;
                changed = true;
            }
            continue;
        }

        const prev = item.rect;
        if (
            !prev ||
            prev.left !== rect.left ||
            prev.top !== rect.top ||
            prev.width !== rect.width ||
            prev.height !== rect.height
        ) {
            item.rect = rect;
            changed = true;
        }
    }

    return changed;
}

export function rebuildData(options: {
    tracked: TrackedElement[];
    priorityIndices: ReadonlySet<number>;
    capacity: number;
    data: Float32Array;
}): {
    data: Float32Array;
    count: number;
    capacity: number;
} {
    let count = 0;
    for (const item of options.tracked) {
        if (item.rect) count++;
    }

    let capacity = options.capacity;
    let data = options.data;
    if (count > capacity) {
        capacity = Math.max(count, capacity * 2);
        data = new Float32Array(capacity * ELEM_FLOATS);
    }

    data.fill(0);
    let idx = 0;

    for (const item of options.tracked) {
        if (!item.rect || !options.priorityIndices.has(item.selectorIdx)) continue;
        writeTrackedRect(data, idx, item);
        idx++;
    }

    for (const item of options.tracked) {
        if (!item.rect || options.priorityIndices.has(item.selectorIdx)) continue;
        writeTrackedRect(data, idx, item);
        idx++;
    }

    return { data, count: idx, capacity };
}

function writeTrackedRect(
    data: Float32Array,
    idx: number,
    item: TrackedElement,
): void {
    const base = idx * ELEM_FLOATS;
    data[base] = item.rect!.left;
    data[base + 1] = item.rect!.top;
    data[base + 2] = item.rect!.width;
    data[base + 3] = item.rect!.height;
    data[base + 4] = item.hue;
    data[base + 5] = item.kind;
    const el = item.ref.deref();
    data[base + 6] = el
        ? parseFloat(el.getAttribute('data-depth') ?? '0') || 0
        : 0;
}