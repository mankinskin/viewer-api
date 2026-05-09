//! ThemeStore — reactive theme colours with CSS custom-property injection.
//!
//! Provides [`ThemeColors`] (all `--theme-*` tokens), four built-in presets,
//! and a [`ThemeStore`] that writes to `:root` CSS variables via `web_sys`
//! and persists the active preset name to `localStorage`.
use dioxus::prelude::*;

use crate::effects::wgpu_overlay::EffectSettings;

mod presets;

pub use self::presets::{ARCADIA, DARK, PAPER, SCRATCHBOARD, ThemeColors, ThemePreset};

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
