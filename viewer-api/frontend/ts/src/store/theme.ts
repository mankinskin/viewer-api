// Shared theme types, defaults, color utilities, and theme store factory.
//
// Used by both log-viewer and doc-viewer for consistent theming.
// Effect-specific settings (WebGPU particles, CRT, etc.) remain in log-viewer.

import { signal, effect } from '@preact/signals';
import type { Signal } from '@preact/signals';
import {
  brightenHex,
  hexLuminance,
  hexToRgba,
  saturateHex,
} from './theme-colors';
import type { EffectSettings } from './theme-effects';

export {
  brightenHex,
  hexLuminance,
  hexToRgba,
  hexToVec3,
  saturateHex,
  vec3ToHex,
} from './theme-colors';
export {
  DEFAULT_EFFECT_SETTINGS,
  DEFAULT_EFFECT_SETTINGS_OFF,
} from './theme-effects';
export type { EffectSettings } from './theme-effects';

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

// ── Global active theme colors ───────────────────────────────────────────────

/**
 * Global signal holding the currently active theme colors.
 * Updated whenever a theme store's colors change.
 * Used by GPU rendering components (e.g. HypergraphViewCore) to apply
 * the current theme without requiring prop drilling.
 */
export const themeColors = signal<ThemeColors>(DEFAULT_THEME);

// ── Preset types ─────────────────────────────────────────────────────────────
export interface ColorTheme {
  name: string;
  description: string;
  colors: ThemeColors;
}

/** A named set of GPU particle / background effect settings. */
export interface EffectTheme {
  name: string;
  description: string;
  effects: EffectSettings;
}

/**
 * A full theme preset composed of a ColorTheme plus an optional EffectTheme.
 * Viewers may extend this with viewer-specific fields (e.g. LogViewerPreset).
 */
export interface ThemePreset extends ColorTheme {
  /** Optional effect overrides to apply alongside the color theme. */
  effectTheme?: EffectTheme;
}

/** @deprecated Use `ColorTheme` instead. Kept for backward compatibility. */
export type LegacyThemePreset = ThemePreset;

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

  // Persist on change and sync global themeColors signal.
  effect(() => {
    const c = colors.value;
    themeColors.value = c;
    try {
      localStorage.setItem(storageKey, JSON.stringify(c));
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

// ── SavedTheme / ThemeSettingsStore ──────────────────────────────────────────

/** A user-saved theme snapshot (colors + effects + optional thumbnail). */
export interface SavedTheme {
  id: string;
  name: string;
  colors: ThemeColors;
  effects?: EffectSettings;
  thumbnail?: string;
  createdAt: number;
}

/**
 * Interface for the store object accepted by the shared `ThemeSettings` component.
 * Each viewer creates its own implementation and passes it as a prop.
 */
export interface ThemeSettingsStore {
  themeColors: Signal<ThemeColors>;
  effectSettings: Signal<EffectSettings>;
  /** Ordered preset list shown in the preset grid. */
  presets: ThemePreset[];
  defaultTheme: ThemeColors;
  updateColor: <K extends keyof ThemeColors>(key: K, value: string) => void;
  applyPreset: (preset: ThemePreset) => void;
  resetTheme: () => void;
  randomizeTheme?: () => void;
  savedThemes: Signal<SavedTheme[]>;
  saveTheme: (name: string, thumbnail?: string) => void;
  deleteTheme: (id: string) => void;
  applySavedTheme: (theme: SavedTheme) => void;
  updateSavedTheme: (id: string, thumbnail?: string) => void;
  renameSavedTheme: (id: string, newName: string) => void;
  updateEffect: <K extends keyof EffectSettings>(key: K, value: EffectSettings[K]) => void;
  exportTheme: (name?: string) => void;
  importTheme: (file: File) => Promise<string | null>;
}
