# T3: Feature — Complete Theme System

## Problem

The Leptos frontend has a minimal theme system: 5 hardcoded presets, a simple button grid in a Settings tab, GPU-only uniforms with no CSS variable injection, and no color editing. The TS version has 17 presets (served as files), 49-property ThemeColors with CSS variable injection, a full ThemeSettings overlay with collapsible sections, and per-color editing.

## Current State (Leptos)

### theme.rs
- **L10–58**: `EffectSettings` struct — 46 numeric/bool fields (0–100 UI percentages)
- **L60–105**: `Default` impl with sensible defaults
- **L126**: `PaletteData = [f32; 96]` — 24 × vec4f GPU color slots
- **L128–161**: `default_palette()` — 24 color slots for particle/smoke/kind colors
- **L166–169**: `ThemePreset { name, effects, palette }`
- **L171–256**: 5 presets: Cinder, Frost, Elysium, Void, Blood Moon
- **L258–278**: Thread-local accessors for GPU frame reads

### store.rs
- **L30–32**: `active_theme: RwSignal<String>`, `effect_settings: RwSignal<EffectSettings>`, `palette_data: RwSignal<PaletteData>`
- **L54–62**: `apply_theme()` — looks up preset by name, updates all signals + thread-locals

### Missing entirely:
- `ThemeColors` struct (49 CSS color properties)
- CSS variable injection on `:root`
- GPU-active CSS overrides
- Color picker UI
- ThemeSettings overlay
- Backend preset loading

## Reference: TS Implementation

### ThemeColors (store/theme.ts L3–53)
49 hex string properties in 9 groups: backgrounds (5), text (3), borders (2), accents (5), log levels bg (5), log levels text (5), span badges (2), particle colors (9), cinder cycle (4), smoke tones (3).

### EffectSettings (log-viewer store/theme.ts L736–838)
38 fields matching the Leptos `EffectSettings` (minus the extra 8 fields Leptos added for fine-tuning).

### 17 Presets (log-viewer store/theme.ts L301–925)
Cinder, Frost, Blood Moon, Verdant, Void, Amber Terminal, Ocean Abyss, Elysium, Sakura, Solarized Dark, Solarized Light, High Contrast, Copper Dusk, Arctic, Neon Noir, Parchment, Emerald Night. Each has `colors: ThemeColors` + `effects: EffectSettings`.

### CSS Injection (viewer-api-frontend store/theme.ts)
Creates a `<style id="theme-vars">` element with `:root { --bg-primary: #xxx; ... }` for all 49 properties. When `gpu-active`, adds transparent/brightened overrides.

### ThemeSettings (ThemeSettings.tsx)
Full overlay with: preset grid, 8 collapsible color sections (backgrounds, text, borders, accents, log levels, span badges), GPU toggle, 6 particle/effect sections (sparks, embers, beams, glitter, cinder, smoke, glass, CRT). Each color row has color picker + hex input + reset button.

## Design

### Step 1: Add ThemeColors struct

```rust
// theme.rs — new struct
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeColors {
    // Backgrounds
    pub bg_primary: String,      // hex "#rrggbb"
    pub bg_secondary: String,
    pub bg_tertiary: String,
    pub bg_hover: String,
    pub bg_active: String,
    // Text
    pub text_primary: String,
    pub text_secondary: String,
    pub text_muted: String,
    // Borders
    pub border_color: String,
    pub border_subtle: String,
    // Accents
    pub accent_blue: String,
    pub accent_green: String,
    pub accent_orange: String,
    pub accent_purple: String,
    pub accent_yellow: String,
    // Log levels (background)
    pub level_trace: String,
    pub level_debug: String,
    pub level_info: String,
    pub level_warn: String,
    pub level_error: String,
    // Log levels (text)
    pub level_trace_text: String,
    pub level_debug_text: String,
    pub level_info_text: String,
    pub level_warn_text: String,
    pub level_error_text: String,
    // Span badges
    pub span_enter_text: String,
    pub span_exit_text: String,
    // Particle: Sparks
    pub particle_spark_core: String,
    pub particle_spark_ember: String,
    pub particle_spark_steel: String,
    // Particle: Embers
    pub particle_ember_hot: String,
    pub particle_ember_base: String,
    // Particle: Beams
    pub particle_beam_center: String,
    pub particle_beam_edge: String,
    // Particle: Glitter
    pub particle_glitter_warm: String,
    pub particle_glitter_cool: String,
    // Cinder cycle
    pub cinder_ember: String,
    pub cinder_gold: String,
    pub cinder_ash: String,
    pub cinder_vine: String,
    // Smoke tones
    pub smoke_cool: String,
    pub smoke_warm: String,
    pub smoke_moss: String,
}
```

### Step 2: Full ThemePreset with colors + effects + palette

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemePreset {
    pub name: String,
    pub description: String,
    pub colors: ThemeColors,
    pub effects: EffectSettings,
    pub palette: PaletteData,
}
```

### Step 3: Backend preset delivery

Add a new endpoint to the log-viewer backend:
```
GET /api/themes              → Vec<ThemePresetSummary> (name, description)
GET /api/themes/{name}       → ThemePreset (full colors + effects + palette)
```

Presets stored as JSON/TOML files in a `themes/` directory served by the backend. The WASM binary does NOT compile preset data — it fetches on demand.

Frontend loads preset list on mount via `gloo_net::http::Request::get("/api/themes")`. When user selects a preset, fetches the full preset via `GET /api/themes/{name}`.

### Step 4: CSS variable injection

Create a `<style>` element via `web_sys` and update it when theme colors change:

```rust
// theme.rs — new function
pub fn inject_css_variables(colors: &ThemeColors) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    
    let style_id = "theme-vars";
    let style_el = document.get_element_by_id(style_id)
        .unwrap_or_else(|| {
            let el = document.create_element("style").unwrap();
            el.set_id(style_id);
            document.head().unwrap().append_child(&el).unwrap();
            el
        });
    
    let css = format!(
        ":root {{\n\
         --bg-primary: {};\n--bg-secondary: {};\n--bg-tertiary: {};\n\
         --bg-hover: {};\n--bg-active: {};\n\
         --text-primary: {};\n--text-secondary: {};\n--text-muted: {};\n\
         --border-color: {};\n--border-subtle: {};\n\
         --accent-blue: {};\n--accent-green: {};\n--accent-orange: {};\n\
         --accent-purple: {};\n--accent-yellow: {};\n\
         /* ... all 49 properties ... */\n\
         }}",
        colors.bg_primary, colors.bg_secondary, colors.bg_tertiary,
        colors.bg_hover, colors.bg_active,
        colors.text_primary, colors.text_secondary, colors.text_muted,
        colors.border_color, colors.border_subtle,
        colors.accent_blue, colors.accent_green, colors.accent_orange,
        colors.accent_purple, colors.accent_yellow,
    );
    
    style_el.set_text_content(Some(&css));
}
```

### Step 5: GPU-active CSS overrides

When `gpu-active` class is present, inject additional overrides for glass transparency. Reference: **style.css L516–543** already has hardcoded values. These should instead be generated from ThemeColors with alpha modifications:

```rust
pub fn inject_gpu_active_overrides(colors: &ThemeColors) {
    // Parse hex → rgba with transparency
    // Brighten text colors for glass readability
    // Saturate accent colors for visibility
    // Generate :root.gpu-active { ... } block
}
```

### Step 6: Theme button in header

Remove ThemeSelector from Settings tab. Add a theme button (palette icon) to `<Header />` that opens the ThemeSettings overlay.

```rust
// header.rs
<button class="header-theme-btn" on:click=move |_| show_theme_settings.set(true)>
    <PaletteIcon />
</button>

// Overlay rendered at App root level:
<Show when=move || show_theme_settings.get()>
    <ThemeSettings on_close=move || show_theme_settings.set(false) />
</Show>
```

### Step 7: ThemeSettings overlay component

Full-screen overlay with scrollable content, organized in collapsible sections:

```
ThemeSettings overlay
├── Header: "Theme Settings" + Close button (×)
├── Actions: Reset to Default | Randomize
├── Section: Theme Presets (open by default)
│   └── Grid of preset cards (name + 6-color swatch)
├── Section: Backgrounds (collapsed)
│   └── 5 ColorRow components
├── Section: Text & Fonts (collapsed)
│   └── 3 ColorRow components
├── Section: Borders (collapsed)
│   └── 2 ColorRow components
├── Section: Accent Colors (collapsed)
│   └── 5 ColorRow components
├── Section: Log Level Colors (collapsed)
│   └── 5 ColorRow (bg) + 5 ColorRow (text)
├── Section: Span Badge Colors (collapsed)
│   └── 2 ColorRow components
├── Section: GPU Rendering
│   └── Toggle switch
├── Section: Particles — Metal Sparks (collapsed)
│   └── Toggle + 3 ColorRow + 3 Sliders (speed, count, size)
├── Section: Particles — Embers (collapsed)
│   └── Toggle + 2 ColorRow + 3 Sliders
├── Section: Particles — Beams (collapsed)
│   └── Toggle + 2 ColorRow + 4 Sliders (speed, height, count, drift)
├── Section: Particles — Glitter (collapsed)
│   └── Toggle + 2 ColorRow + 3 Sliders
├── Section: Cinder Palette (collapsed)
│   └── Toggle + 4 ColorRow + 1 Slider (size)
├── Section: Background Smoke (collapsed)
│   └── Toggle + 3 ColorRow + sliders (intensity, speed, warm/cool/moss)
├── Section: Glass Panels (open by default)
│   └── 2 Sliders (opacity, blur)
└── Section: CRT Effect (open by default)
    └── Toggle + 5 Sliders + ColorRow (tint)
```

### Step 8: Reusable sub-components

**CollapsibleSection**: Tracks open/closed state, renders chevron + title + optional icon:
```rust
#[component]
fn CollapsibleSection(
    title: &'static str,
    icon: &'static str,
    #[prop(default = false)] default_open: bool,
    children: Children,
) -> impl IntoView { ... }
```

**ColorRow**: Color input + hex text input + reset:
```rust
#[component]
fn ColorRow(
    label: &'static str,
    value: Signal<String>,
    on_change: Callback<String>,
    default: String,
) -> impl IntoView { ... }
```

**Slider**: Range input with label + value display:
```rust
#[component]
fn Slider(
    label: &'static str,
    value: Signal<f32>,
    on_change: Callback<f32>,
    min: f32,
    max: f32,
    step: f32,
) -> impl IntoView { ... }
```

### Step 9: Store updates

**store.rs** — Add `theme_colors: RwSignal<ThemeColors>` signal. Wire `create_effect` to call `inject_css_variables()` whenever `theme_colors` changes.

**Palette sync**: When particle/cinder/smoke colors change in `ThemeColors`, update the corresponding `PaletteData` slots (hex → f32 RGB conversion) and push to GPU thread-local.

### Step 10: localStorage persistence

Save current `ThemeColors` + `EffectSettings` to localStorage on every change:
```rust
fn persist_theme(colors: &ThemeColors, effects: &EffectSettings) {
    let window = web_sys::window().unwrap();
    if let Ok(Some(storage)) = window.local_storage() {
        let data = serde_json::json!({
            "colors": colors,
            "effects": effects,
        });
        let _ = storage.set_item("log-viewer-theme", &data.to_string());
    }
}
```

On app init, load from localStorage before fetching presets:
```rust
fn load_persisted_theme() -> Option<(ThemeColors, EffectSettings)> {
    let storage = web_sys::window()?.local_storage().ok()??;
    let raw = storage.get_item("log-viewer-theme").ok()??;
    serde_json::from_str(&raw).ok()
}
```

## Files to Create

| File | Purpose |
|------|---------|
| `src/components/theme_settings.rs` | Full ThemeSettings overlay component |
| `src/components/color_row.rs` | Reusable color picker row |
| `src/components/slider.rs` | Reusable range slider |
| `src/components/collapsible_section.rs` | Collapsible accordion section |

## Files to Modify

| File | Change |
|------|--------|
| `src/theme.rs` | Add `ThemeColors` struct, preset loading from API, CSS injection, GPU-active overrides |
| `src/store.rs` | Add `theme_colors` signal, CSS injection effect, palette sync, localStorage persist/load |
| `src/app.rs` | Mount ThemeSettings overlay, remove old Settings tab ThemeSelector |
| `src/components/header.rs` | Add theme button with palette icon |
| `src/components/mod.rs` | Register new component modules |
| `style.css` | ThemeSettings overlay styles, color row, slider, section styles |
| Backend: add `/api/themes` endpoint + preset files |

## Acceptance Criteria

1. ThemeColors struct with all 49 color properties, injected as CSS custom properties on `:root`
2. All 17 presets loaded dynamically from backend (not compiled into WASM)
3. GPU-active CSS overrides for glass transparency
4. Theme button in header (not Settings tab) opens ThemeSettings overlay
5. Full ThemeSettings overlay with collapsible sections for all color groups and effect controls
6. EffectSettings with all properties driving GPU uniforms
7. Selecting a preset loads ThemeColors + EffectSettings + PaletteData
8. Individual color changes update CSS vars in real-time
9. localStorage persistence of current theme
