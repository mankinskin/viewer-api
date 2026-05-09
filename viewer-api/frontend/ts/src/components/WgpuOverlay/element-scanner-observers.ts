import type { TrackedElement } from './element-scanner-tracking';

export function createElementScannerObservers(options: {
    elementMap: Map<Element, TrackedElement>;
    onRectsStale: () => void;
    onProcessMutations: (records: MutationRecord[]) => void;
}): {
    io: IntersectionObserver;
    ro: ResizeObserver;
    mo: MutationObserver;
} {
    const io = new IntersectionObserver(
        (entries) => {
            for (const entry of entries) {
                const tracked = options.elementMap.get(entry.target);
                if (tracked) {
                    tracked.visible = entry.isIntersecting;
                    if (entry.isIntersecting) {
                        tracked.rectStale = true;
                        options.onRectsStale();
                    }
                }
            }
        },
        { threshold: 0 },
    );

    const ro = new ResizeObserver((entries) => {
        for (const entry of entries) {
            const tracked = options.elementMap.get(entry.target);
            if (tracked) {
                tracked.rectStale = true;
                options.onRectsStale();
            }
        }
    });

    const mo = new MutationObserver((records) => {
        options.onProcessMutations(records);
    });

    return { io, ro, mo };
}