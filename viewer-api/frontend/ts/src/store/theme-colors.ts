/** Convert "#rrggbb" to [r, g, b] in 0..1 range (for GPU shaders). */
export function hexToVec3(hex: string): [number, number, number] {
  const normalized = hex.replace('#', '');
  const r = parseInt(normalized.slice(0, 2), 16) / 255;
  const g = parseInt(normalized.slice(2, 4), 16) / 255;
  const b = parseInt(normalized.slice(4, 6), 16) / 255;
  return [r, g, b];
}

/** Convert [r, g, b] (0..1) to "#rrggbb". */
export function vec3ToHex(r: number, g: number, b: number): string {
  const clamp = (value: number) =>
    Math.max(0, Math.min(255, Math.round(value * 255)));
  return (
    '#' +
    [r, g, b]
      .map((value) => clamp(value).toString(16).padStart(2, '0'))
      .join('')
  );
}

/** Calculate relative luminance of a hex color (0 = black, 1 = white). */
export function hexLuminance(hex: string): number {
  const r = parseInt(hex.slice(1, 3), 16) / 255;
  const g = parseInt(hex.slice(3, 5), 16) / 255;
  const b = parseInt(hex.slice(5, 7), 16) / 255;
  const toLinear = (channel: number) =>
    channel <= 0.03928
      ? channel / 12.92
      : Math.pow((channel + 0.055) / 1.055, 2.4);
  return 0.2126 * toLinear(r) + 0.7152 * toLinear(g) + 0.0722 * toLinear(b);
}

/** Convert a hex color to an rgba() string with the given alpha. */
export function hexToRgba(hex: string, alpha: number): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

/**
 * Brighten a hex color by a factor (0..1 = 0%..100% brighter towards white).
 * Used to improve text readability on transparent/glass GPU backgrounds.
 */
export function brightenHex(hex: string, factor: number): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  const brighten = (channel: number) =>
    Math.min(255, Math.round(channel + (255 - channel) * factor));
  return vec3ToHex(brighten(r) / 255, brighten(g) / 255, brighten(b) / 255);
}

/**
 * Saturate/boost a hex color by a factor (0..1).
 * Increases saturation while preserving luminance.
 */
export function saturateHex(hex: string, factor: number): string {
  const r = parseInt(hex.slice(1, 3), 16) / 255;
  const g = parseInt(hex.slice(3, 5), 16) / 255;
  const b = parseInt(hex.slice(5, 7), 16) / 255;
  const gray = 0.2989 * r + 0.587 * g + 0.114 * b;
  const saturate = (channel: number) =>
    Math.min(1, Math.max(0, gray + (channel - gray) * (1 + factor)));
  return vec3ToHex(saturate(r), saturate(g), saturate(b));
}