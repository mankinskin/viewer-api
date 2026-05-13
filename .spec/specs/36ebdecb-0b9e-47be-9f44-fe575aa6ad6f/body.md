# viewer-api: theme settings

Canonical UX and behavior specification for the **shared theme-settings panel** used by every Dioxus viewer (`doc-viewer`, `log-viewer`, `ticket-viewer`, `spec-viewer`). The reference implementation lives in
`tools/viewer/viewer-api/frontend/ts/src/components/ThemeSettings/ThemeSettings.tsx` and
`…/store/theme.ts`. The Dioxus port lives in `tools/viewer/viewer-api/frontend/dioxus/src/components/theme_settings.rs`. CSS (`viewer-api.css`) is the single source of truth and is mirrored verbatim into each viewer's `frontend/dioxus/public/`.

All viewers MUST present the same panel, the same controls, in the same order, with the same defaults, and with the same persistence semantics. Per-viewer customization is limited to:

- The **preset list** (each viewer ships its own named ColorTheme presets).
- The **GPU effect default** (this is a deliberate cross-viewer constant: every viewer defaults the master GPU toggle to **ON**. The viewer is intended to be fully GPU-accelerated by default — 3D graph rendering, glass panels, particle effects, smoke. Users can opt out via the master toggle in ThemeSettings).

---

## 1. Layout

The panel is split into two columns inside `.theme-settings-layout`:

| Column | Element |
|---|---|
| Left (main) | `.theme-settings` — header + collapsible color/effect sections |
| Right | `.saved-themes-panel` — user-saved snapshots (always visible) |

### Header (`.theme-settings-header`)

- `<h2>` "Color Theme Settings"
- Subtitle: "Customize every color in the palette. Changes are applied instantly and saved to your browser."
- Action row (`.theme-settings-actions`) with these buttons in this order:
  1. **Reset to Default** (`btn btn-primary`)
  2. **🎲 Randomize** (`btn btn-primary`) — only rendered when `store.randomizeTheme` exists
  3. **💾 Save Theme** (`btn btn-primary`) — opens an inline name input + "Save"/"✕" buttons
  4. **📤 Export** (`btn btn-secondary`) — downloads the current theme as `<name>.json`
  5. **📂 Import** (`btn btn-secondary`) — opens a hidden `<input type="file" accept=".json">`; errors render inline as `.theme-import-error`

---

## 2. Sections (collapsible)

Each section is a `<button class="theme-section-header">` showing icon + title + chevron (`▾`/`▸`). Body is `.theme-section-body`. Sections appear in this exact order:

| # | Title | Icon | Default Open | Notes |
|---|---|---|---|---|
| 1 | Theme Presets | ◆ | **yes** | Grid of `.theme-preset-card`s; each shows 5 swatches + name + description |
| 2 | Backgrounds | ▧ | no | 5 ColorRows: Primary, Secondary, Tertiary, Hover, Active |
| 3 | Text & Fonts | A | no | 3 ColorRows: Primary, Secondary, Muted |
| 4 | Borders | □ | no | 2 ColorRows: Border, Subtle Border |
| 5 | Accent Colors | ◈ | no | 5 ColorRows: Blue, Green, Orange, Purple, Yellow |
| 6 | Log Level Colors | ▤ | no | 5 ColorRows: TRACE…ERROR |
| 7 | Log Level Text Colors | T | no | 5 ColorRows for badge text colors |
| 8 | Span Badge Colors | → | no | 2 ColorRows: Enter, Exit |
| 9 | GPU Rendering | ⬢ | no | Master GPU toggle (see §4) |
| 10 | Particles: Metal Sparks | ✦ | no | Toggle + 3 ColorRows + 3 sliders (Speed/Count/Size) |
| 11 | Particles: Embers / Ash | 🔥 | no | Toggle + 2 ColorRows + 3 sliders |
| 12 | Particles: Angelic Beams | ✧ | no | Toggle + 2 ColorRows + 4 sliders (Speed, Height, Count, Drift) |
| 13 | Particles: Glitter | ✨ | no | Toggle + 2 ColorRows + 3 sliders |
| 14 | Cinder Palette | ◎ | no | Toggle + 4 ColorRows + 1 slider (Size) |
| 15 | Background Smoke | ☁ | no | Toggle + 3 ColorRows + 10 sliders |
| 16 | Glass Panels | ◻ | **yes** | 2 sliders: Opacity, Blur |
| 17 | CRT Effect | ▤ | **yes** | Toggle + 5 sliders + 1 ColorRow (Scanline Color) |

When a section toggle is **off**, only the toggle row is shown; the colors and sliders for that effect are hidden.

---

## 3. Color rows (`.theme-color-row`)

Each row shows:

| Element | CSS class |
|---|---|
| Label | `.theme-color-label` |
| Description (optional) | `.theme-color-desc` |
| Native color picker | `.theme-color-picker` (`<input type="color">`) |
| Hex text input | `.theme-color-hex` (max 7 chars, validated against `/^#[0-9a-fA-F]{6}$/`) |
| Reset button "↺" | `.theme-color-reset` (resets to `defaultTheme[colorKey]`) |

Color updates are **applied instantly** (every keystroke) and persisted to localStorage on change.

---

## 4. GPU master toggle

A single iOS-style toggle in the GPU Rendering section. Wired to `gpuOverlayEnabled` (TS) / `set_gpu_overlay_enabled(bool)` (Rust). Persisted to localStorage key `viewer-api-gpu-enabled`.

**Default for all viewers: ON.** The viewer is fully GPU-accelerated by default (3D graph rendering, glass panels, particle effects, background animation). Users can opt out via the master toggle.

When OFF, the WebGPU RAF loop does not draw — the canvas stays clear, even though all CSS theming still applies.

---

## 5. Effect controls

Every per-effect section follows this shape:

1. **Toggle row** (`.theme-toggle-row`): label + `.toggle-switch` (the iOS slider styled checkbox).
2. **Conditional body** (only when toggle is checked):
   - 1–4 **ColorRow**s for the effect's particle colors.
   - 0–10 **slider rows** (`.theme-slider-row`) showing label + `<input type="range">` + percentage value.

### Slider ranges (canonical)

| Section | Sliders | Min | Max | Step | Suffix |
|---|---|---|---|---|---|
| Sparks | Speed | 0 | 300 | 1 | % |
| Sparks | Count | 0 | 200 | 1 | % |
| Sparks | Size | 0 | 300 | 1 | % |
| Embers | Speed/Count/Size | same as Sparks |
| Beams | Speed | 0 | 300 | 1 | % |
| Beams | Height | **10** | 100 | 1 | % |
| Beams | Count | 0 | 1024 | 1 | (raw, "All" if 0) |
| Beams | Drift | 0 | 300 | 1 | % |
| Glitter | Speed/Count/Size | same as Sparks |
| Cinder | Size | 0 | 300 | 1 | % |
| Smoke | Intensity | 0 | 100 | 1 | % |
| Smoke | Speed | 0 | 500 | 1 | (raw) |
| Smoke | Warm/Cool/Moss Scale | 0 | 200 | 1 | (raw) |
| Smoke | Grain Intensity | 0 | 100 | 1 | % |
| Smoke | Grain Coarseness | 0 | 100 | 1 | (raw) |
| Smoke | Grain Size | 0 | 100 | 1 | (raw) |
| Smoke | Vignette | 0 | 100 | 1 | % |
| Smoke | Underglow | 0 | 100 | 1 | % |
| Glass | Opacity | 0 | 100 | 1 | % |
| Glass | Blur | 0 | 100 | 1 | % |
| CRT | H/V Scanlines, Edge Shadow, Flicker, Line Width | 0 | 100 | 1 | % |
| CRT | Scanline Color | — | — | — | `<input type="color">` (encodes `[r,g,b]` triplet) |

---

## 6. Saved themes panel (right column)

`.saved-themes-panel` always visible:

- `<h3>` "Saved Themes"
- Subtitle: "Your custom themes, stored in the browser."
- Empty state: `.saved-themes-empty` with hint about the 💾 button.
- Populated: `.saved-themes-list` of `.saved-theme-card`s.

Each card shows:

1. **Thumbnail**: `.saved-theme-thumbnail` (PNG data-URL captured from the WebGPU canvas via `captureOverlayThumbnail()`), or `.saved-theme-swatches` fallback (6 colored squares: bgPrimary, accentOrange, accentBlue, levelError, cinderEmber, textPrimary).
2. `.saved-theme-info`: name (double-click to rename) + creation date.
3. `.saved-theme-actions`:
   - **Apply** — `applySavedTheme(theme)`
   - **✏️** — opens "Confirm" → overwrites theme's colors & thumbnail with current state via `updateSavedTheme`
   - **🗑** — opens "Confirm" → `deleteTheme(id)`

---

## 7. Persistence

| Key | Stored value |
|---|---|
| `viewer-api-theme` | `JSON.stringify(ThemeColors)` |
| `viewer-api-effects` | `JSON.stringify(EffectSettings)` |
| `viewer-api-gpu-enabled` | `"true" \| "false"` |
| `viewer-api-saved-themes` | `JSON.stringify(SavedTheme[])` |

All writes happen synchronously when the corresponding signal changes. Reads happen on first store construction.

---

## 8. Public store interface

The shared `ThemeSettings` component is parameterized by a `ThemeSettingsStore` so each viewer can plug in its own signals/presets. The interface (mirrored from
`tools/viewer/viewer-api/frontend/ts/src/store/theme.ts`):

```ts
interface ThemeSettingsStore {
  themeColors: Signal<ThemeColors>;
  effectSettings: Signal<EffectSettings>;
  presets: ThemePreset[];
  defaultTheme: ThemeColors;
  updateColor<K extends keyof ThemeColors>(key: K, value: string): void;
  applyPreset(preset: ThemePreset): void;
  resetTheme(): void;
  randomizeTheme?(): void;            // optional: enables 🎲 button
  savedThemes: Signal<SavedTheme[]>;
  saveTheme(name: string, thumbnail?: string): void;
  deleteTheme(id: string): void;
  applySavedTheme(theme: SavedTheme): void;
  updateSavedTheme(id: string, thumbnail?: string): void;
  renameSavedTheme(id: string, newName: string): void;
  updateEffect<K extends keyof EffectSettings>(key: K, value: EffectSettings[K]): void;
  exportTheme(name?: string): void;
  importTheme(file: File): Promise<string | null>;
}
```

For the Dioxus port the same surface MUST be exposed via `ThemeStore` (`Signal<T>` instead of preact signals; `Vec<ThemePreset>` for presets; `set_…` setters for each mutation).

---

## 9. Defaults

Default `ThemeColors` ("Arcadia" warm marble) and `DEFAULT_EFFECT_SETTINGS` are defined in
`tools/viewer/viewer-api/frontend/ts/src/store/theme.ts`. The Dioxus port in
`tools/viewer/viewer-api/frontend/dioxus/src/store/theme.rs` MUST mirror those values exactly; a divergence is a bug.

The opt-out variant `DEFAULT_EFFECT_SETTINGS_OFF` zeros every effect (incl. `glassOpacity = 0`, `glassBlur = 0`, all toggles = `false`) and is the default state for `ticket-viewer` / `spec-viewer`.

---

## 10. Acceptance criteria

A viewer is considered conformant when:

1. The panel renders with the 17 sections in the order listed in §2.
2. Each ColorRow accepts both color picker and hex input, with reset.
3. Every slider matches the range table in §5.
4. The GPU master toggle persists to `viewer-api-gpu-enabled` and gates the WebGPU render loop.
5. Save/Apply/Update/Rename/Delete operations on the saved-themes panel work and survive page reload.
6. Export downloads a `.json` containing `{ name, colors, effects }`; Import validates and applies the same shape.
7. CSS classes match: `.theme-settings`, `.theme-section`, `.theme-color-row`, `.theme-color-picker`, `.theme-color-hex`, `.theme-color-reset`, `.theme-toggle-row`, `.toggle-switch`, `.theme-slider-row`, `.theme-range-slider`, `.theme-slider-value`, `.theme-presets-grid`, `.theme-preset-card`, `.saved-themes-panel`, `.saved-theme-card`. Viewers MAY add additional BEM-style aliases (`.theme-settings__*`) but must keep the canonical names available.
