/**
 * Node styling utilities.
 */

/**
 * Get CSS class for node based on its width relative to max width.
 * Maps node width to log-level color classes for visual hierarchy.
 */
export function nodeWidthClass(width: number, maxWidth: number): string {
    if (width === 1) return 'level-info'; // atoms → calm blue glow
    const t = (width - 1) / Math.max(maxWidth - 1, 1);
    if (t > 0.7) return 'level-error'; // wide nodes → hot red
    if (t > 0.4) return 'level-warn'; // medium → amber
    return 'level-debug'; // small compounds → dim green
}
