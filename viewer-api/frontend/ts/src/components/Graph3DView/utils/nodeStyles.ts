/**
 * Node styling utilities for Graph3DView.
 */

/** Map a node's size metric to a CSS level class for color coding. */
export function nodeSizeClass(size: number, maxSize: number): string {
    if (size <= 1) return 'level-info';
    const t = (size - 1) / Math.max(maxSize - 1, 1);
    if (t > 0.7) return 'level-error';
    if (t > 0.4) return 'level-warn';
    return 'level-debug';
}

/** Parse a CSS hex color string (#RRGGBB) into [r, g, b, a] normalized floats. */
export function hexToRgba(hex: string): [number, number, number, number] {
    const r = parseInt(hex.slice(1, 3), 16) / 255;
    const g = parseInt(hex.slice(3, 5), 16) / 255;
    const b = parseInt(hex.slice(5, 7), 16) / 255;
    return [r, g, b, 1];
}

/** Compute a default evenly-distributed hue-based color for a node. */
export function defaultNodeRgba(index: number, total: number): [number, number, number, number] {
    const hue = (index / Math.max(total, 1)) * 360;
    const h = hue / 60;
    const s = 0.6;
    const l = 0.6;
    const c = (1 - Math.abs(2 * l - 1)) * s;
    const x = c * (1 - Math.abs(h % 2 - 1));
    const m = l - c / 2;
    let r = m, g = m, b = m;
    if (h < 1) { r += c; g += x; }
    else if (h < 2) { r += x; g += c; }
    else if (h < 3) { g += c; b += x; }
    else if (h < 4) { g += x; b += c; }
    else if (h < 5) { r += x; b += c; }
    else { r += c; b += x; }
    return [r, g, b, 1];
}
