/**
 * AppSchema — per-application configuration for the WgpuOverlay system.
 *
 * Each tool (log-viewer, ticket-viewer, etc.) creates its own AppSchema
 * instance that describes which DOM elements to illuminate and provides
 * runtime signals for theme and effect state.
 */
import type { Signal } from '@preact/signals';
import type { ThemeColors, EffectSettings } from '../../store/theme';

// ---------------------------------------------------------------------------
// Selector entry
// ---------------------------------------------------------------------------

/** A single CSS selector entry with GPU shader metadata. */
export interface SchemaSelectorEntry {
    /** CSS selector to match DOM elements. */
    sel: string;
    /** Hue value 0..1 assigned to this element category for the shader. */
    hue: number;
    /** Shader kind discriminant (see KIND_* constants in element-types.ts). */
    kind: number;
    /**
     * When true, this selector is scanned first and always gets a slot in the
     * element buffer. Used for interactive-state selectors (selected,
     * highlighted, panic) that drive particle effects.
     */
    priority?: boolean;
}

// ---------------------------------------------------------------------------
// AppSchema
// ---------------------------------------------------------------------------

export interface AppSchema {
    /** CSS selectors for UI elements to track and illuminate with GPU effects. */
    selectors: ReadonlyArray<SchemaSelectorEntry>;
    /** Reactive signal providing current theme colors for GPU palette upload. */
    themeColors: Signal<ThemeColors>;
    /** Reactive signal providing current effect settings for the render loop. */
    effectSettings: Signal<EffectSettings>;
    /**
     * Returns the numeric view ID for the currently active tab/view.
     * Used by the GPU shader for per-view particle filtering.
     * Return 0 if no per-view filtering is needed.
     */
    getCurrentViewId(): number;
    /**
     * Returns true when the active view uses 3D perspective projection.
     * The render loop uses 2D orthographic matrices when false.
     */
    isActive3DView(): boolean;
}

// ---------------------------------------------------------------------------
// MINIMAL_SCHEMA — base for tools without GPU element decoration
// ---------------------------------------------------------------------------

import { signal } from '@preact/signals';
import { DEFAULT_THEME, DEFAULT_EFFECT_SETTINGS } from '../../store/theme';

/** Minimal schema with no element selectors. Suitable as a base for extending. */
export const MINIMAL_SCHEMA: AppSchema = {
    selectors: [],
    themeColors: signal(DEFAULT_THEME),
    effectSettings: signal(DEFAULT_EFFECT_SETTINGS),
    getCurrentViewId: () => 0,
    isActive3DView: () => false,
};
