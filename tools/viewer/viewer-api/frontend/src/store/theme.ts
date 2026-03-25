// Shared theme types, defaults, color utilities, and theme store factory.
//
// Used by both log-viewer and doc-viewer for consistent theming.
// Effect-specific settings (WebGPU particles, CRT, etc.) remain in log-viewer.

import { signal, effect } from '@preact/signals';
import type { Signal } from '@preact/signals';

// ── ThemeColors interface ────────────────────────────────────────────────────

export interface ThemeColors {
  // Backgrounds
  bgPrimary: string;
  bgSecondary: string;
  bgTertiary: string;
  bgHover: string;
  bgActive: string;

  // Text
  textPrimary: string;
  textSecondary: string;
  textMuted: string;

  // Borders
  borderColor: string;
  borderSubtle: string;

  // Accents
  accentBlue: string;
  accentGreen: string;
  accentOrange: string;
  accentPurple: string;
  accentYellow: string;

  // Log levels (background colors)
  levelTrace: string;
  levelDebug: string;
  levelInfo: string;
  levelWarn: string;
  levelError: string;

  // Log level badge text colors (readable on level backgrounds)
  levelTraceText: string;
  levelDebugText: string;
  levelInfoText: string;
  levelWarnText: string;
  levelErrorText: string;

  // Span type badge colors
  spanEnterText: string;
  spanExitText: string;

  // Particle: Metal Spark
  particleSparkCore: string;
  particleSparkEmber: string;
  particleSparkSteel: string;

  // Particle: Ember / Ash
  particleEmberHot: string;
  particleEmberBase: string;

  // Particle: Angelic Beam
  particleBeamCenter: string;
  particleBeamEdge: string;

  // Particle: Glitter
  particleGlitterWarm: string;
  particleGlitterCool: string;

  // Cinder palette cycle (used in borders/glows)
  cinderEmber: string;
  cinderGold: string;
  cinderAsh: string;
  cinderVine: string;

  // Background smoke tones
  smokeCool: string;
  smokeWarm: string;
  smokeMoss: string;
}

// ── Default theme (warm marble / "Arcadia" light theme) ──────────────────────

export const DEFAULT_THEME: ThemeColors = {
  bgPrimary: '#eae6df',
  bgSecondary: '#f2efe8',
  bgTertiary: '#f8f6f1',
  bgHover: '#dfd9cf',
  bgActive: '#d4cdc0',
  textPrimary: '#1e1c18',
  textSecondary: '#4a4640',
  textMuted: '#74706a',
  borderColor: '#c8c0b4',
  borderSubtle: '#ddd8ce',
  accentBlue: '#5a9ec4',
  accentGreen: '#4a8a52',
  accentOrange: '#c49050',
  accentPurple: '#8a6aaa',
  accentYellow: '#b8a040',
  levelTrace: '#d8d4cc',
  levelDebug: '#b8d4b8',
  levelInfo: '#b0cce0',
  levelWarn: '#e0c888',
  levelError: '#d4948a',
  levelTraceText: '#4a4848',
  levelDebugText: '#2a4a30',
  levelInfoText: '#2a4060',
  levelWarnText: '#5a4020',
  levelErrorText: '#5a2020',
  spanEnterText: '#2a6a40',
  spanExitText: '#6a4020',
  particleSparkCore: '#fff8e0',
  particleSparkEmber: '#d4aa50',
  particleSparkSteel: '#c8c0b8',
  particleEmberHot: '#f0d888',
  particleEmberBase: '#c89840',
  particleBeamCenter: '#ffffff',
  particleBeamEdge: '#ffe8b0',
  particleGlitterWarm: '#fff0c8',
  particleGlitterCool: '#d8e8ff',
  cinderEmber: '#c8a040',
  cinderGold: '#e0c870',
  cinderAsh: '#b8b0a0',
  cinderVine: '#5a9a58',
  smokeCool: '#a8cce8',
  smokeWarm: '#c8ddf0',
  smokeMoss: '#e8f0fa',
};

// ── Preset type ──────────────────────────────────────────────────────────────

export interface ThemePreset {
  name: string;
  description: string;
  colors: ThemeColors;
}

// ── Color utilities ──────────────────────────────────────────────────────────

/** Convert "#rrggbb" to [r, g, b] in 0..1 range (for GPU shaders). */
export function hexToVec3(hex: string): [number, number, number] {
  const h = hex.replace('#', '');
  const r = parseInt(h.slice(0, 2), 16) / 255;
  const g = parseInt(h.slice(2, 4), 16) / 255;
  const b = parseInt(h.slice(4, 6), 16) / 255;
  return [r, g, b];
}

/** Convert [r, g, b] (0..1) to "#rrggbb". */
export function vec3ToHex(r: number, g: number, b: number): string {
  const clamp = (v: number) => Math.max(0, Math.min(255, Math.round(v * 255)));
  return '#' + [r, g, b].map(v => clamp(v).toString(16).padStart(2, '0')).join('');
}

/** Calculate relative luminance of a hex color (0 = black, 1 = white). */
export function hexLuminance(hex: string): number {
  const r = parseInt(hex.slice(1, 3), 16) / 255;
  const g = parseInt(hex.slice(3, 5), 16) / 255;
  const b = parseInt(hex.slice(5, 7), 16) / 255;
  const toLinear = (c: number) => c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
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
  // Lerp towards white
  const brighten = (c: number) => Math.min(255, Math.round(c + (255 - c) * factor));
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
  const saturate = (c: number) => Math.min(1, Math.max(0, gray + (c - gray) * (1 + factor)));
  return vec3ToHex(saturate(r), saturate(g), saturate(b));
}

// ── Theme store factory ──────────────────────────────────────────────────────

export interface ThemeStore {
  /** Reactive signal holding current theme colors. */
  colors: Signal<ThemeColors>;
  /** Update a single color key. */
  updateColor: <K extends keyof ThemeColors>(key: K, value: string) => void;
  /** Apply a complete preset. */
  applyPreset: (preset: ThemeColors) => void;
  /** Reset to default theme. */
  reset: () => void;
}

/**
 * Create a theme store with localStorage persistence and CSS variable
 * injection. Each viewer tool calls this with its own storage key.
 *
 * @param storageKey - localStorage key for persistence (e.g. 'log-viewer-theme')
 * @param defaults - default theme colors (defaults to DEFAULT_THEME)
 * @param enableGpuOverrides - whether to generate :root.gpu-active CSS rules
 */
export function createThemeStore(
  storageKey: string,
  defaults: ThemeColors = DEFAULT_THEME,
  enableGpuOverrides = false,
): ThemeStore {
  // Load from localStorage
  function loadTheme(): ThemeColors {
    try {
      const saved = localStorage.getItem(storageKey);
      if (saved) {
        return { ...defaults, ...JSON.parse(saved) };
      }
    } catch { /* ignore */ }
    return { ...defaults };
  }

  const colors = signal<ThemeColors>(loadTheme());

  // Persist on change
  effect(() => {
    try {
      localStorage.setItem(storageKey, JSON.stringify(colors.value));
    } catch { /* storage full */ }
  });

  // Apply CSS custom properties
  let styleEl: HTMLStyleElement | null = null;
  effect(() => {
    const c = colors.value;
    if (!styleEl) {
      styleEl = document.createElement('style');
      styleEl.id = `${storageKey}-theme`;
      document.head.appendChild(styleEl);
    }

    const bgLum = hexLuminance(c.bgPrimary);
    const colorScheme = bgLum < 0.2 ? 'dark' : 'light';

    let css = `:root {
  color-scheme: ${colorScheme};
  --bg-primary: ${c.bgPrimary};
  --bg-secondary: ${c.bgSecondary};
  --bg-tertiary: ${c.bgTertiary};
  --bg-hover: ${c.bgHover};
  --bg-active: ${c.bgActive};
  --text-primary: ${c.textPrimary};
  --text-secondary: ${c.textSecondary};
  --text-muted: ${c.textMuted};
  --border-color: ${c.borderColor};
  --border-subtle: ${c.borderSubtle};
  --accent-blue: ${c.accentBlue};
  --accent-green: ${c.accentGreen};
  --accent-orange: ${c.accentOrange};
  --accent-purple: ${c.accentPurple};
  --accent-yellow: ${c.accentYellow};
  --level-trace: ${c.levelTrace};
  --level-debug: ${c.levelDebug};
  --level-info: ${c.levelInfo};
  --level-warn: ${c.levelWarn};
  --level-error: ${c.levelError};
  --level-trace-text: ${c.levelTraceText};
  --level-debug-text: ${c.levelDebugText};
  --level-info-text: ${c.levelInfoText};
  --level-warn-text: ${c.levelWarnText};
  --level-error-text: ${c.levelErrorText};
  --span-enter-text: ${c.spanEnterText};
  --span-exit-text: ${c.spanExitText};
}`;

    if (enableGpuOverrides) {
      const gpuBgSecondary = hexToRgba(c.bgSecondary, 0.25);
      const gpuBgTertiary = hexToRgba(c.bgTertiary, 0.25);
      const gpuBgHover = hexToRgba(c.bgHover, 0.35);
      const gpuBgActive = hexToRgba(c.bgActive, 0.35);
      const gpuBorderColor = hexToRgba(c.borderColor, 0.35);
      const gpuBorderSubtle = hexToRgba(c.borderSubtle, 0.25);
      // Brighten text colors for glass/transparent background readability
      const gpuTextPrimary = brightenHex(c.textPrimary, 0.12);
      const gpuTextSecondary = brightenHex(c.textSecondary, 0.15);
      const gpuTextMuted = brightenHex(c.textMuted, 0.20);
      // Boost accent colors for visibility on transparent backgrounds
      const gpuAccentBlue = saturateHex(brightenHex(c.accentBlue, 0.15), 0.1);
      const gpuAccentGreen = saturateHex(brightenHex(c.accentGreen, 0.15), 0.1);
      const gpuAccentPurple = saturateHex(brightenHex(c.accentPurple, 0.15), 0.1);
      const gpuAccentYellow = saturateHex(brightenHex(c.accentYellow, 0.12), 0.1);
      css += `\n:root.gpu-active {
  --bg-primary: transparent;
  --bg-secondary: ${gpuBgSecondary};
  --bg-tertiary: ${gpuBgTertiary};
  --bg-hover: ${gpuBgHover};
  --bg-active: ${gpuBgActive};
  --border-color: ${gpuBorderColor};
  --border-subtle: ${gpuBorderSubtle};
  --text-primary: ${gpuTextPrimary};
  --text-secondary: ${gpuTextSecondary};
  --text-muted: ${gpuTextMuted};
  --accent-blue: ${gpuAccentBlue};
  --accent-green: ${gpuAccentGreen};
  --accent-purple: ${gpuAccentPurple};
  --accent-yellow: ${gpuAccentYellow};
  /* Mobile sidebar needs opaque background even in glass/GPU mode */
  --sidebar-mobile-bg: ${c.bgSecondary};
}`;
    }

    styleEl.textContent = css;
  });

  return {
    colors,
    updateColor: <K extends keyof ThemeColors>(key: K, value: string) => {
      colors.value = { ...colors.value, [key]: value };
    },
    applyPreset: (preset: ThemeColors) => {
      colors.value = { ...preset };
    },
    reset: () => {
      colors.value = { ...defaults };
    },
  };
}

// ── EffectSettings ───────────────────────────────────────────────────────────
// GPU particle and background effect configuration shared across viewer tools.

export interface EffectSettings {
  /** Glass panel background opacity 0–100 (maps to 0.0–0.4 alpha). */
  glassOpacity: number;
  /** Glass panel backdrop blur 0–100 (maps to 0–16px). */
  glassBlur: number;
  crtEnabled: boolean;
  /** Horizontal scanlines (+ pixel grid) intensity 0–100. */
  crtScanlinesH: number;
  /** Vertical scanlines (+ pixel grid) intensity 0–100. */
  crtScanlinesV: number;
  /** Edge/border shadow intensity 0–100. */
  crtEdgeShadow: number;
  /** Torch flicker intensity 0–100. */
  crtFlicker: number;
  /** Scanline width/thickness 0–100. */
  crtLineWidth: number;
  /** Scanline tint color [R, G, B] 0–255. */
  crtColor: [number, number, number];
  smokeEnabled: boolean;
  smokeIntensity: number;
  smokeSpeed: number;
  smokeWarmScale: number;
  smokeCoolScale: number;
  smokeMossScale: number;
  grainIntensity: number;
  grainCoarseness: number;
  grainSize: number;
  vignetteStrength: number;
  underglowStrength: number;
  sparkSpeed: number;
  sparksEnabled: boolean;
  emberSpeed: number;
  embersEnabled: boolean;
  beamSpeed: number;
  beamsEnabled: boolean;
  glitterSpeed: number;
  glitterEnabled: boolean;
  beamHeight: number;
  beamDrift: number;
  beamCount: number;
  sparkCount: number;
  sparkSize: number;
  emberCount: number;
  emberSize: number;
  glitterCount: number;
  glitterSize: number;
  cinderSize: number;
  cinderEnabled: boolean;
}

export const DEFAULT_EFFECT_SETTINGS: EffectSettings = {
  glassOpacity: 35,
  glassBlur: 25,
  crtEnabled: true,
  crtScanlinesH: 20,
  crtScanlinesV: 12,
  crtEdgeShadow: 35,
  crtFlicker: 12,
  crtLineWidth: 50,
  crtColor: [100, 80, 60] as [number, number, number],
  smokeEnabled: true,
  smokeIntensity: 40,
  smokeSpeed: 50,
  smokeWarmScale: 100,
  smokeCoolScale: 100,
  smokeMossScale: 100,
  grainIntensity: 20,
  grainCoarseness: 40,
  grainSize: 35,
  vignetteStrength: 40,
  underglowStrength: 25,
  sparkSpeed: 70,
  sparksEnabled: true,
  emberSpeed: 70,
  embersEnabled: true,
  beamSpeed: 50,
  beamsEnabled: true,
  glitterSpeed: 60,
  glitterEnabled: true,
  beamHeight: 35,
  beamDrift: 80,
  beamCount: 48,
  sparkCount: 40,
  sparkSize: 70,
  emberCount: 40,
  emberSize: 70,
  glitterCount: 40,
  glitterSize: 60,
  cinderSize: 70,
  cinderEnabled: true,
};
