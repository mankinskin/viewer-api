/**
 * ThemeSettings — shared color/effect theme settings component.
 *
 * Parameterized via a `ThemeSettingsStore` prop so any viewer can use it with
 * its own signals and functions. The store is injected via Preact context so
 * sub-components don't need prop drilling.
 *
 * Sections:
 *   - Presets (one-click theme switching)
 *   - Backgrounds / Text / Borders / Accents
 *   - Log Level Colors + Badge Text
 *   - Span Badge Colors
 *   - GPU toggle
 *   - Particle Effects (Sparks, Embers, Beams, Glitter, Cinder)
 *   - Background Smoke
 *   - Glass Panels
 *   - CRT Effect
 *   - Saved Themes panel
 */
import type { ThemeSettingsStore } from '../../store/theme';

import { ThemeSettingsProvider, useThemeSettingsStore } from './theme-settings-context';
import { ThemeColorSections } from './theme-settings-color-sections';
import { ThemeEffectSections } from './theme-settings-effect-sections';
import {
  ImportThemeButton,
  SaveThemeButton,
  SavedThemesPanel,
} from './theme-settings-ui';
import './theme-settings.css';

// ── ThemeSettingsImpl ─────────────────────────────────────────────────────────

function ThemeSettingsImpl() {
  const store = useThemeSettingsStore();
  const stopPropagation = (event: Event) => event.stopPropagation();

  return (
    <div
      class="theme-settings-layout"
      data-graph-passthrough="false"
      onMouseDown={stopPropagation}
      onClick={stopPropagation}
      onWheel={stopPropagation}
      onTouchStart={stopPropagation}
      onTouchMove={stopPropagation}
    >
      <div class="theme-settings">
        <div class="theme-settings-header">
          <h2 class="theme-settings-title">Color Theme Settings</h2>
          <p class="theme-settings-subtitle">
            Customize every color in the palette. Changes are applied instantly and saved to your browser.
          </p>
          <div class="theme-settings-actions">
            <button class="btn btn-primary" onClick={store.resetTheme}>Reset to Default</button>
            {store.randomizeTheme && (
              <button class="btn btn-primary" onClick={store.randomizeTheme}>🎲 Randomize</button>
            )}
            <SaveThemeButton />
            <button class="btn btn-secondary" onClick={() => store.exportTheme()} title="Export current theme as .json">
              📤 Export
            </button>
            <ImportThemeButton />
          </div>
        </div>

        <ThemeColorSections />
        <ThemeEffectSections />
      </div>

      <SavedThemesPanel />
    </div>
  );
}

// ── Public export ─────────────────────────────────────────────────────────────

export { type ThemeSettingsStore };

/**
 * Shared ColorTheme settings component.
 *
 * Each viewer creates a `ThemeSettingsStore` from its own signals and functions
 * and passes it here as a prop.
 */
export function ThemeSettings({ store }: { store: ThemeSettingsStore }) {
  return (
    <ThemeSettingsProvider store={store}>
      <ThemeSettingsImpl />
    </ThemeSettingsProvider>
  );
}
