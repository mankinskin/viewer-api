//! ThemeStore — reactive theme colours with CSS custom-property injection.
//!
//! Provides [`ThemeColors`] (all `--theme-*` tokens), four built-in presets,
//! and a [`ThemeStore`] that writes to `:root` CSS variables via `web_sys`
//! and persists the active preset name to `localStorage`.
use dioxus::prelude::*;

use crate::effects::wgpu_overlay::EffectSettings;

// ── Data types ────────────────────────────────────────────────────────────────

/// Full set of design-token colours mirroring the TypeScript theme.ts interface.
#[derive(Clone, PartialEq, Debug)]
pub struct ThemeColors {
    // Backgrounds
    pub bg_primary: &'static str,
    pub bg_secondary: &'static str,
    pub bg_tertiary: &'static str,
    pub bg_elevated: &'static str,
    // Text
    pub text_primary: &'static str,
    pub text_secondary: &'static str,
    pub text_muted: &'static str,
    // Borders
    pub border_primary: &'static str,
    pub border_secondary: &'static str,
    // Accents
    pub accent_blue: &'static str,
    pub accent_purple: &'static str,
    pub accent_green: &'static str,
    pub accent_yellow: &'static str,
    pub accent_red: &'static str,
    pub accent_orange: &'static str,
    pub accent_cyan: &'static str,
    // Syntax tokens (mapped in CSS to syntect `.highlight.*` class colours).
    pub syntax_keyword: &'static str,
    pub syntax_string: &'static str,
    pub syntax_comment: &'static str,
    pub syntax_number: &'static str,
    pub syntax_function: &'static str,
    pub syntax_type: &'static str,
    pub syntax_variable: &'static str,
}

/// Named preset identifier.
#[derive(Clone, PartialEq, Debug, Default)]
pub enum ThemePreset {
    #[default]
    Arcadia,
    Dark,
    Paper,
    Scratchboard,
}

impl ThemePreset {
    pub fn key(&self) -> &'static str {
        match self {
            ThemePreset::Arcadia => "arcadia",
            ThemePreset::Dark => "dark",
            ThemePreset::Paper => "paper",
            ThemePreset::Scratchboard => "scratchboard",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "arcadia" => Some(ThemePreset::Arcadia),
            "dark" => Some(ThemePreset::Dark),
            "paper" => Some(ThemePreset::Paper),
            "scratchboard" => Some(ThemePreset::Scratchboard),
            _ => None,
        }
    }

    pub fn colors(&self) -> &'static ThemeColors {
        match self {
            ThemePreset::Arcadia => &ARCADIA,
            ThemePreset::Dark => &DARK,
            ThemePreset::Paper => &PAPER,
            ThemePreset::Scratchboard => &SCRATCHBOARD,
        }
    }
}

// ── Built-in presets ──────────────────────────────────────────────────────────

/// Arcadia — warm marble light theme (default).
pub static ARCADIA: ThemeColors = ThemeColors {
    bg_primary: "#eae6df",
    bg_secondary: "#f2efe8",
    bg_tertiary: "#f8f6f1",
    bg_elevated: "#ffffff",
    text_primary: "#2c2a26",
    text_secondary: "#5a5650",
    text_muted: "#8a8680",
    border_primary: "#d4cfc7",
    border_secondary: "#e8e4dc",
    accent_blue: "#4a7fa5",
    accent_purple: "#7c5c9e",
    accent_green: "#4a8c5c",
    accent_yellow: "#b8860b",
    accent_red: "#c0392b",
    accent_orange: "#d35400",
    accent_cyan: "#1a8c8c",
    syntax_keyword: "#4a7fa5",
    syntax_string: "#4a8c5c",
    syntax_comment: "#8a8680",
    syntax_number: "#b8860b",
    syntax_function: "#7c5c9e",
    syntax_type: "#1a8c8c",
    syntax_variable: "#2c2a26",
};

/// Dark — dracula-inspired dark theme.
pub static DARK: ThemeColors = ThemeColors {
    bg_primary: "#1a1b26",
    bg_secondary: "#1f2035",
    bg_tertiary: "#24283b",
    bg_elevated: "#2c2f4a",
    text_primary: "#c0caf5",
    text_secondary: "#9aa5ce",
    text_muted: "#565f89",
    border_primary: "#292e42",
    border_secondary: "#1f2335",
    accent_blue: "#7aa2f7",
    accent_purple: "#bb9af7",
    accent_green: "#9ece6a",
    accent_yellow: "#e0af68",
    accent_red: "#f7768e",
    accent_orange: "#ff9e64",
    accent_cyan: "#7dcfff",
    syntax_keyword: "#bb9af7",
    syntax_string: "#9ece6a",
    syntax_comment: "#565f89",
    syntax_number: "#ff9e64",
    syntax_function: "#7aa2f7",
    syntax_type: "#7dcfff",
    syntax_variable: "#c0caf5",
};

/// Paper — soft off-white light theme.
pub static PAPER: ThemeColors = ThemeColors {
    bg_primary: "#f5f0eb",
    bg_secondary: "#faf8f5",
    bg_tertiary: "#ffffff",
    bg_elevated: "#ffffff",
    text_primary: "#1a1a1a",
    text_secondary: "#4a4a4a",
    text_muted: "#888888",
    border_primary: "#ddd8d0",
    border_secondary: "#ece8e2",
    accent_blue: "#2563eb",
    accent_purple: "#7c3aed",
    accent_green: "#16a34a",
    accent_yellow: "#ca8a04",
    accent_red: "#dc2626",
    accent_orange: "#ea580c",
    accent_cyan: "#0891b2",
    syntax_keyword: "#2563eb",
    syntax_string: "#16a34a",
    syntax_comment: "#888888",
    syntax_number: "#ca8a04",
    syntax_function: "#7c3aed",
    syntax_type: "#0891b2",
    syntax_variable: "#1a1a1a",
};

/// Scratchboard — high-contrast near-black theme.
pub static SCRATCHBOARD: ThemeColors = ThemeColors {
    bg_primary: "#0f0f0f",
    bg_secondary: "#1a1a1a",
    bg_tertiary: "#222222",
    bg_elevated: "#2a2a2a",
    text_primary: "#f0f0f0",
    text_secondary: "#b0b0b0",
    text_muted: "#606060",
    border_primary: "#333333",
    border_secondary: "#2a2a2a",
    accent_blue: "#58a6ff",
    accent_purple: "#c499f3",
    accent_green: "#56d364",
    accent_yellow: "#e3b341",
    accent_red: "#f85149",
    accent_orange: "#ffa657",
    accent_cyan: "#39d0d8",
    syntax_keyword: "#58a6ff",
    syntax_string: "#56d364",
    syntax_comment: "#606060",
    syntax_number: "#ffa657",
    syntax_function: "#c499f3",
    syntax_type: "#39d0d8",
    syntax_variable: "#f0f0f0",
};

// ── CSS injection ─────────────────────────────────────────────────────────────

/// Build a `<style>` block that sets all `:root` CSS custom properties.
fn colors_to_css(colors: &ThemeColors) -> String {
    format!(
        r#":root {{
  --bg-primary: {bg_primary};
  --bg-secondary: {bg_secondary};
  --bg-tertiary: {bg_tertiary};
  --bg-elevated: {bg_elevated};
    --bg-hover: color-mix(in srgb, {bg_secondary} 86%, {text_primary} 14%);
    --bg-active: color-mix(in srgb, {bg_secondary} 76%, {accent_blue} 24%);
  --text-primary: {text_primary};
  --text-secondary: {text_secondary};
  --text-muted: {text_muted};
  --border-primary: {border_primary};
  --border-secondary: {border_secondary};
    --border-color: {border_primary};
    --border-subtle: {border_secondary};
  --accent-blue: {accent_blue};
  --accent-purple: {accent_purple};
  --accent-green: {accent_green};
  --accent-yellow: {accent_yellow};
  --accent-red: {accent_red};
  --accent-orange: {accent_orange};
  --accent-cyan: {accent_cyan};
  --syntax-keyword: {syntax_keyword};
  --syntax-string: {syntax_string};
  --syntax-comment: {syntax_comment};
  --syntax-number: {syntax_number};
  --syntax-function: {syntax_function};
  --syntax-type: {syntax_type};
  --syntax-variable: {syntax_variable};

  /* ── Panel surface tokens ──────────────────────────────────────────
     Derived from the active theme palette so light themes get light
     translucent panels and dark themes get dark translucent panels.
     The WebGPU smoke shader still bleeds through. */
  --panel-bg:        color-mix(in srgb, {bg_secondary} 96%, transparent);
  --panel-bg-strong: color-mix(in srgb, {bg_secondary} 99%, transparent);
  --panel-bg-floor:  color-mix(in srgb, {bg_primary}   96%, transparent);
  --panel-blur:      14px;
  --panel-saturate:  150%;

  /* Solid fallbacks (mirror theme bg). */
  --bg-primary-solid:   {bg_primary};
  --bg-secondary-solid: {bg_secondary};
  --bg-tertiary-solid:  {bg_tertiary};
}}"#,
        bg_primary = colors.bg_primary,
        bg_secondary = colors.bg_secondary,
        bg_tertiary = colors.bg_tertiary,
        bg_elevated = colors.bg_elevated,
        text_primary = colors.text_primary,
        text_secondary = colors.text_secondary,
        text_muted = colors.text_muted,
        border_primary = colors.border_primary,
        border_secondary = colors.border_secondary,
        accent_blue = colors.accent_blue,
        accent_purple = colors.accent_purple,
        accent_green = colors.accent_green,
        accent_yellow = colors.accent_yellow,
        accent_red = colors.accent_red,
        accent_orange = colors.accent_orange,
        accent_cyan = colors.accent_cyan,
        syntax_keyword = colors.syntax_keyword,
        syntax_string = colors.syntax_string,
        syntax_comment = colors.syntax_comment,
        syntax_number = colors.syntax_number,
        syntax_function = colors.syntax_function,
        syntax_type = colors.syntax_type,
        syntax_variable = colors.syntax_variable,
    )
}

const STYLE_ELEM_ID: &str = "viewer-api-theme";
const STORAGE_KEY: &str = "viewer-api-theme";
const GPU_STORAGE_KEY: &str = "viewer-api-gpu-enabled";

// ── ThemeStore ────────────────────────────────────────────────────────────────

/// Reactive store for the active theme.
///
/// Call [`ThemeStore::use_store`] inside a component to access it.
/// The store injects a `<style id="viewer-api-theme">` element into
/// `document.head` whenever the preset changes and persists the selection
/// to `localStorage`.
///
/// Also tracks the master GPU-overlay enable flag, which gates rendering of
/// the [`crate::effects::WgpuOverlay`] (smoke / particles / CRT effects).
/// Defaults to **off** so first-load viewers do not render expensive effects
/// without the user opting in via the Theme Settings panel.
#[derive(Clone, Copy)]
pub struct ThemeStore {
    preset: Signal<ThemePreset>,
    gpu_enabled: Signal<bool>,
    /// The **committed** effect settings — the value persisted to
    /// `localStorage` and restored on page load.  Live preview state lives in
    /// the global `EFFECTS_LIVE` thread-local inside `wgpu_overlay`.
    effects_committed: Signal<EffectSettings>,
}

impl ThemeStore {
    /// Initialise the store (call once near the top of the app component).
    ///
    /// Reads the saved preset from `localStorage` on first mount and applies it.
    pub fn use_store() -> Self {
        #[cfg(target_arch = "wasm32")]
        let initial = {
            web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
                .and_then(|s| s.get_item(STORAGE_KEY).ok().flatten())
                .and_then(|k| ThemePreset::from_key(&k))
                .unwrap_or_default()
        };
        #[cfg(not(target_arch = "wasm32"))]
        let initial = ThemePreset::default();

        // GPU enabled flag — default ON; persisted under GPU_STORAGE_KEY.
        // The viewer is intended to be fully GPU-accelerated by default
        // (3D graph rendering, glass panels, particle effects, smoke).
        // Users can opt out via the master toggle in ThemeSettings.
        #[cfg(target_arch = "wasm32")]
        let initial_gpu = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item(GPU_STORAGE_KEY).ok().flatten())
            .map(|v| v == "true")
            .unwrap_or(true);
        #[cfg(not(target_arch = "wasm32"))]
        let initial_gpu = true;

        let preset = use_signal(|| initial);
        let gpu_enabled = use_signal(|| initial_gpu);
        // Load committed shader effects from localStorage.  Push the same
        // snapshot into the WgpuOverlay's live state so the render loop
        // immediately picks up the user's saved tweaks on first paint.
        let initial_effects = EffectSettings::load();
        crate::effects::wgpu_overlay::set_live_effects(initial_effects.clone());
        let effects_committed = use_signal(|| initial_effects);
        let store = ThemeStore { preset, gpu_enabled, effects_committed };

        // Inject CSS for the initial preset on first mount.
        use_effect(move || {
            store.apply_css(preset.read().clone());
        });

        // Apply GPU-enabled flag to the overlay on first mount and whenever it changes.
        use_effect(move || {
            let enabled = *gpu_enabled.read();
            crate::effects::wgpu_overlay::set_gpu_overlay_enabled(enabled);
        });

        store
    }

    /// Current active [`ThemePreset`].
    pub fn preset(&self) -> ThemePreset {
        self.preset.read().clone()
    }

    /// Current active [`ThemeColors`].
    pub fn colors(&self) -> &'static ThemeColors {
        self.preset.read().colors()
    }

    /// Whether the WebGPU overlay (smoke / particles / CRT) should render.
    pub fn gpu_enabled(&self) -> bool {
        *self.gpu_enabled.read()
    }

    /// Enable or disable the WebGPU overlay. Persists to `localStorage`.
    pub fn set_gpu_enabled(&mut self, enabled: bool) {
        self.gpu_enabled.set(enabled);
        crate::effects::wgpu_overlay::set_gpu_overlay_enabled(enabled);
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
            {
                let _ = storage.set_item(GPU_STORAGE_KEY, if enabled { "true" } else { "false" });
            }
        }
    }

    /// Switch to a different preset, inject updated CSS, and persist the choice.
    pub fn apply_preset(&mut self, p: ThemePreset) {
        self.preset.set(p.clone());
        self.apply_css(p.clone());
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
            {
                let _ = storage.set_item(STORAGE_KEY, p.key());
            }
        }
    }

    // ── Effect-settings API ──

    /// Snapshot of the **committed** effect settings (the persisted value).
    /// Use this as the starting point for a new edit session.
    pub fn effects_committed(&self) -> EffectSettings {
        self.effects_committed.read().clone()
    }

    /// Push a draft snapshot to the live render loop for immediate preview.
    /// Does **not** persist to `localStorage` and does **not** mutate the
    /// committed snapshot — call [`commit_effects`] for that.
    pub fn preview_effects(&self, draft: EffectSettings) {
        crate::effects::wgpu_overlay::set_live_effects(draft);
    }

    /// Persist a draft snapshot as the new committed value: writes to
    /// `localStorage`, updates the committed Signal, and pushes it live so
    /// the render loop and any subscribers see the same value.
    pub fn commit_effects(&mut self, draft: EffectSettings) {
        draft.save();
        crate::effects::wgpu_overlay::set_live_effects(draft.clone());
        self.effects_committed.set(draft);
    }

    /// Discard any pending preview by re-pushing the committed snapshot to
    /// the live render loop.  The committed Signal is unchanged.
    pub fn revert_effects(&self) {
        let saved = self.effects_committed.read().clone();
        crate::effects::wgpu_overlay::set_live_effects(saved);
    }

    // ── private ──

    fn apply_css(&self, preset: ThemePreset) {
        #[cfg(target_arch = "wasm32")]
        {
            let css = colors_to_css(preset.colors());
            if let Some(window) = web_sys::window() {
                if let Some(doc) = window.document() {
                    // Reuse or create the style element.
                    let style_el = if let Some(el) = doc.get_element_by_id(STYLE_ELEM_ID) {
                        el
                    } else {
                        let el = doc
                            .create_element("style")
                            .expect("create_element style");
                        el.set_id(STYLE_ELEM_ID);
                        if let Some(head) = doc.head() {
                            let _ = head.append_child(&el);
                        }
                        el
                    };
                    style_el.set_text_content(Some(&css));                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (preset, css_inject_noop());
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn css_inject_noop() {}

// ── ThemeProvider component ───────────────────────────────────────────────────

/// Context provider — wraps the application and makes [`ThemeStore`] available
/// via Dioxus context.
///
/// Access it in any child with `use_context::<ThemeStore>()`.  
/// Prefer [`ThemeProvider`] at the app root.
#[component]
pub fn ThemeProvider(children: Element) -> Element {
    let store = ThemeStore::use_store();
    provide_context(store);
    rsx! { { children } }
}
