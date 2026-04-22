//! ThemeSettings panel — color pickers for all theme tokens, preset selector,
//! save/rename/delete custom themes, JSON export/import, live preview, and undo.
//!
//! Uses Dioxus signals for local draft state.  Custom themes are stored in
//! `localStorage` under the key `"viewer-api-custom-themes"`.
use dioxus::prelude::*;

use crate::store::{ThemeColors, ThemePreset, ThemeStore, ARCADIA, DARK, PAPER, SCRATCHBOARD};

// ── Mutable colour snapshot ───────────────────────────────────────────────────

/// An owned, heap-allocated copy of all theme colour tokens.
///
/// Mirrors [`ThemeColors`] but with `String` instead of `&'static str` so that
/// it can be produced by the settings UI without requiring `'static` lifetimes.
#[derive(Clone, PartialEq, Debug)]
pub struct ThemeSnapshot {
    pub bg_primary: String,
    pub bg_secondary: String,
    pub bg_tertiary: String,
    pub bg_elevated: String,
    pub text_primary: String,
    pub text_secondary: String,
    pub text_muted: String,
    pub border_primary: String,
    pub border_secondary: String,
    pub accent_blue: String,
    pub accent_purple: String,
    pub accent_green: String,
    pub accent_yellow: String,
    pub accent_red: String,
    pub accent_orange: String,
    pub accent_cyan: String,
    pub syntax_keyword: String,
    pub syntax_string: String,
    pub syntax_comment: String,
    pub syntax_number: String,
    pub syntax_function: String,
    pub syntax_type: String,
    pub syntax_variable: String,
}

impl ThemeSnapshot {
    /// Borrow from a static [`ThemeColors`].
    pub fn from_colors(c: &ThemeColors) -> Self {
        Self {
            bg_primary: c.bg_primary.to_string(),
            bg_secondary: c.bg_secondary.to_string(),
            bg_tertiary: c.bg_tertiary.to_string(),
            bg_elevated: c.bg_elevated.to_string(),
            text_primary: c.text_primary.to_string(),
            text_secondary: c.text_secondary.to_string(),
            text_muted: c.text_muted.to_string(),
            border_primary: c.border_primary.to_string(),
            border_secondary: c.border_secondary.to_string(),
            accent_blue: c.accent_blue.to_string(),
            accent_purple: c.accent_purple.to_string(),
            accent_green: c.accent_green.to_string(),
            accent_yellow: c.accent_yellow.to_string(),
            accent_red: c.accent_red.to_string(),
            accent_orange: c.accent_orange.to_string(),
            accent_cyan: c.accent_cyan.to_string(),
            syntax_keyword: c.syntax_keyword.to_string(),
            syntax_string: c.syntax_string.to_string(),
            syntax_comment: c.syntax_comment.to_string(),
            syntax_number: c.syntax_number.to_string(),
            syntax_function: c.syntax_function.to_string(),
            syntax_type: c.syntax_type.to_string(),
            syntax_variable: c.syntax_variable.to_string(),
        }
    }

    /// Serialize to a minimal JSON string for export / localStorage.
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"bg_primary":{q},"bg_secondary":{qs},"bg_tertiary":{qt},"bg_elevated":{qe},"text_primary":{tp},"text_secondary":{ts},"text_muted":{tm},"border_primary":{bp},"border_secondary":{bs},"accent_blue":{ab},"accent_purple":{ap},"accent_green":{ag},"accent_yellow":{ay},"accent_red":{ar},"accent_orange":{ao},"accent_cyan":{ac},"syntax_keyword":{sk},"syntax_string":{ss},"syntax_comment":{sc},"syntax_number":{sn},"syntax_function":{sf},"syntax_type":{sty},"syntax_variable":{sv}}}"#,
            q   = json_str(&self.bg_primary),
            qs  = json_str(&self.bg_secondary),
            qt  = json_str(&self.bg_tertiary),
            qe  = json_str(&self.bg_elevated),
            tp  = json_str(&self.text_primary),
            ts  = json_str(&self.text_secondary),
            tm  = json_str(&self.text_muted),
            bp  = json_str(&self.border_primary),
            bs  = json_str(&self.border_secondary),
            ab  = json_str(&self.accent_blue),
            ap  = json_str(&self.accent_purple),
            ag  = json_str(&self.accent_green),
            ay  = json_str(&self.accent_yellow),
            ar  = json_str(&self.accent_red),
            ao  = json_str(&self.accent_orange),
            ac  = json_str(&self.accent_cyan),
            sk  = json_str(&self.syntax_keyword),
            ss  = json_str(&self.syntax_string),
            sc  = json_str(&self.syntax_comment),
            sn  = json_str(&self.syntax_number),
            sf  = json_str(&self.syntax_function),
            sty = json_str(&self.syntax_type),
            sv  = json_str(&self.syntax_variable),
        )
    }

    /// Deserialize from the simple JSON format produced by [`to_json`].
    pub fn from_json(json: &str) -> Option<Self> {
        fn extract(json: &str, key: &str) -> Option<String> {
            let needle = format!("\"{}\":\"", key);
            let start = json.find(&needle)? + needle.len();
            let rest = &json[start..];
            let end = rest.find('"')?;
            Some(rest[..end].to_string())
        }

        Some(Self {
            bg_primary: extract(json, "bg_primary")?,
            bg_secondary: extract(json, "bg_secondary")?,
            bg_tertiary: extract(json, "bg_tertiary")?,
            bg_elevated: extract(json, "bg_elevated")?,
            text_primary: extract(json, "text_primary")?,
            text_secondary: extract(json, "text_secondary")?,
            text_muted: extract(json, "text_muted")?,
            border_primary: extract(json, "border_primary")?,
            border_secondary: extract(json, "border_secondary")?,
            accent_blue: extract(json, "accent_blue")?,
            accent_purple: extract(json, "accent_purple")?,
            accent_green: extract(json, "accent_green")?,
            accent_yellow: extract(json, "accent_yellow")?,
            accent_red: extract(json, "accent_red")?,
            accent_orange: extract(json, "accent_orange")?,
            accent_cyan: extract(json, "accent_cyan")?,
            syntax_keyword: extract(json, "syntax_keyword")?,
            syntax_string: extract(json, "syntax_string")?,
            syntax_comment: extract(json, "syntax_comment")?,
            syntax_number: extract(json, "syntax_number")?,
            syntax_function: extract(json, "syntax_function")?,
            syntax_type: extract(json, "syntax_type")?,
            syntax_variable: extract(json, "syntax_variable")?,
        })
    }
}

fn json_str(s: &str) -> String {
    // Escape special characters for JSON string values.
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r");
    format!("\"{}\"", escaped)
}

// ── Custom theme record ───────────────────────────────────────────────────────

/// A named, user-saved custom theme.
#[derive(Clone, PartialEq, Debug)]
pub struct CustomTheme {
    pub name: String,
    pub colors: ThemeSnapshot,
}

impl CustomTheme {
    fn to_json(&self) -> String {
        format!(
            r#"{{"name":{},"colors":{}}}"#,
            json_str(&self.name),
            self.colors.to_json()
        )
    }

    fn from_json(json: &str) -> Option<Self> {
        let needle = "\"name\":\"";
        let start = json.find(needle)? + needle.len();
        let rest = &json[start..];
        let end = rest.find('"')?;
        let name = rest[..end].to_string();

        let colors_needle = "\"colors\":";
        let colors_start = json.find(colors_needle)? + colors_needle.len();
        let colors_json = &json[colors_start..];
        let colors = ThemeSnapshot::from_json(colors_json)?;

        Some(Self { name, colors })
    }
}

const CUSTOM_THEMES_KEY: &str = "viewer-api-custom-themes";

// ── localStorage helpers ──────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn load_custom_themes() -> Vec<CustomTheme> {
    let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
    else {
        return vec![];
    };
    let Ok(Some(raw)) = storage.get_item(CUSTOM_THEMES_KEY) else {
        return vec![];
    };
    parse_custom_themes_json(&raw)
}

#[cfg(not(target_arch = "wasm32"))]
fn load_custom_themes() -> Vec<CustomTheme> {
    vec![]
}

#[cfg(target_arch = "wasm32")]
fn save_custom_themes_storage(themes: &[CustomTheme]) {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
    {
        let json = serialize_custom_themes(themes);
        let _ = storage.set_item(CUSTOM_THEMES_KEY, &json);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn save_custom_themes_storage(_themes: &[CustomTheme]) {}

/// Serialize a list of custom themes as a JSON array.
fn serialize_custom_themes(themes: &[CustomTheme]) -> String {
    let items: Vec<String> = themes.iter().map(|t| t.to_json()).collect();
    format!("[{}]", items.join(","))
}

/// Parse a JSON array of custom themes.
fn parse_custom_themes_json(json: &str) -> Vec<CustomTheme> {
    let mut themes = Vec::new();
    let trimmed = json.trim();
    if !trimmed.starts_with('[') {
        return themes;
    }
    // Simple greedy extraction: find each `{"name":` block.
    let mut remainder = trimmed.trim_start_matches('[').trim_end_matches(']');
    while let Some(start) = remainder.find("{\"name\":") {
        remainder = &remainder[start..];
        // Find the matching closing brace by tracking depth.
        let mut depth = 0;
        let mut end = 0;
        for (i, ch) in remainder.char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        if end == 0 {
            break;
        }
        let block = &remainder[..end];
        if let Some(t) = CustomTheme::from_json(block) {
            themes.push(t);
        }
        remainder = &remainder[end..];
    }
    themes
}

// ── Inject live preview CSS ───────────────────────────────────────────────────

const PREVIEW_STYLE_ID: &str = "viewer-api-theme-preview";

fn inject_preview_css(snap: &ThemeSnapshot) {
    #[cfg(target_arch = "wasm32")]
    {
        let css = format!(
            r#":root {{
  --bg-primary: {bg_primary};
  --bg-secondary: {bg_secondary};
  --bg-tertiary: {bg_tertiary};
  --bg-elevated: {bg_elevated};
  --text-primary: {text_primary};
  --text-secondary: {text_secondary};
  --text-muted: {text_muted};
  --border-primary: {border_primary};
  --border-secondary: {border_secondary};
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
}}"#,
            bg_primary     = snap.bg_primary,
            bg_secondary   = snap.bg_secondary,
            bg_tertiary    = snap.bg_tertiary,
            bg_elevated    = snap.bg_elevated,
            text_primary   = snap.text_primary,
            text_secondary = snap.text_secondary,
            text_muted     = snap.text_muted,
            border_primary  = snap.border_primary,
            border_secondary = snap.border_secondary,
            accent_blue    = snap.accent_blue,
            accent_purple  = snap.accent_purple,
            accent_green   = snap.accent_green,
            accent_yellow  = snap.accent_yellow,
            accent_red     = snap.accent_red,
            accent_orange  = snap.accent_orange,
            accent_cyan    = snap.accent_cyan,
            syntax_keyword  = snap.syntax_keyword,
            syntax_string   = snap.syntax_string,
            syntax_comment  = snap.syntax_comment,
            syntax_number   = snap.syntax_number,
            syntax_function = snap.syntax_function,
            syntax_type     = snap.syntax_type,
            syntax_variable = snap.syntax_variable,
        );

        if let Some(window) = web_sys::window() {
            if let Some(doc) = window.document() {
                let el = if let Some(existing) = doc.get_element_by_id(PREVIEW_STYLE_ID) {
                    existing
                } else {
                    let new_el = doc.create_element("style").expect("create style");
                    new_el.set_id(PREVIEW_STYLE_ID);
                    if let Some(head) = doc.head() {
                        let _ = head.append_child(&new_el);
                    }
                    new_el
                };
                el.set_text_content(Some(&css));
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = snap;
}

fn remove_preview_css() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Some(doc) = window.document() {
                if let Some(el) = doc.get_element_by_id(PREVIEW_STYLE_ID) {
                    if let Some(parent) = el.parent_node() {
                        let _ = parent.remove_child(&el);
                    }
                }
            }
        }
    }
}

// ── ThemeSettings component ───────────────────────────────────────────────────

/// Token group for grouping color pickers in the UI.
struct TokenGroup {
    title: &'static str,
    tokens: &'static [(&'static str, &'static str)],
}

static TOKEN_GROUPS: &[TokenGroup] = &[
    TokenGroup {
        title: "Backgrounds",
        tokens: &[
            ("bg_primary", "Primary Background"),
            ("bg_secondary", "Secondary Background"),
            ("bg_tertiary", "Tertiary Background"),
            ("bg_elevated", "Elevated Background"),
        ],
    },
    TokenGroup {
        title: "Text",
        tokens: &[
            ("text_primary", "Primary Text"),
            ("text_secondary", "Secondary Text"),
            ("text_muted", "Muted Text"),
        ],
    },
    TokenGroup {
        title: "Borders",
        tokens: &[
            ("border_primary", "Primary Border"),
            ("border_secondary", "Secondary Border"),
        ],
    },
    TokenGroup {
        title: "Accents",
        tokens: &[
            ("accent_blue", "Blue"),
            ("accent_purple", "Purple"),
            ("accent_green", "Green"),
            ("accent_yellow", "Yellow"),
            ("accent_red", "Red"),
            ("accent_orange", "Orange"),
            ("accent_cyan", "Cyan"),
        ],
    },
    TokenGroup {
        title: "Syntax",
        tokens: &[
            ("syntax_keyword", "Keyword"),
            ("syntax_string", "String"),
            ("syntax_comment", "Comment"),
            ("syntax_number", "Number"),
            ("syntax_function", "Function"),
            ("syntax_type", "Type"),
            ("syntax_variable", "Variable"),
        ],
    },
];

fn get_token<'a>(snap: &'a ThemeSnapshot, key: &str) -> &'a str {
    match key {
        "bg_primary"      => &snap.bg_primary,
        "bg_secondary"    => &snap.bg_secondary,
        "bg_tertiary"     => &snap.bg_tertiary,
        "bg_elevated"     => &snap.bg_elevated,
        "text_primary"    => &snap.text_primary,
        "text_secondary"  => &snap.text_secondary,
        "text_muted"      => &snap.text_muted,
        "border_primary"  => &snap.border_primary,
        "border_secondary"=> &snap.border_secondary,
        "accent_blue"     => &snap.accent_blue,
        "accent_purple"   => &snap.accent_purple,
        "accent_green"    => &snap.accent_green,
        "accent_yellow"   => &snap.accent_yellow,
        "accent_red"      => &snap.accent_red,
        "accent_orange"   => &snap.accent_orange,
        "accent_cyan"     => &snap.accent_cyan,
        "syntax_keyword"  => &snap.syntax_keyword,
        "syntax_string"   => &snap.syntax_string,
        "syntax_comment"  => &snap.syntax_comment,
        "syntax_number"   => &snap.syntax_number,
        "syntax_function" => &snap.syntax_function,
        "syntax_type"     => &snap.syntax_type,
        "syntax_variable" => &snap.syntax_variable,
        _ => "",
    }
}

fn set_token(snap: &mut ThemeSnapshot, key: &str, value: String) {
    match key {
        "bg_primary"      => snap.bg_primary = value,
        "bg_secondary"    => snap.bg_secondary = value,
        "bg_tertiary"     => snap.bg_tertiary = value,
        "bg_elevated"     => snap.bg_elevated = value,
        "text_primary"    => snap.text_primary = value,
        "text_secondary"  => snap.text_secondary = value,
        "text_muted"      => snap.text_muted = value,
        "border_primary"  => snap.border_primary = value,
        "border_secondary"=> snap.border_secondary = value,
        "accent_blue"     => snap.accent_blue = value,
        "accent_purple"   => snap.accent_purple = value,
        "accent_green"    => snap.accent_green = value,
        "accent_yellow"   => snap.accent_yellow = value,
        "accent_red"      => snap.accent_red = value,
        "accent_orange"   => snap.accent_orange = value,
        "accent_cyan"     => snap.accent_cyan = value,
        "syntax_keyword"  => snap.syntax_keyword = value,
        "syntax_string"   => snap.syntax_string = value,
        "syntax_comment"  => snap.syntax_comment = value,
        "syntax_number"   => snap.syntax_number = value,
        "syntax_function" => snap.syntax_function = value,
        "syntax_type"     => snap.syntax_type = value,
        "syntax_variable" => snap.syntax_variable = value,
        _ => {}
    }
}

/// ThemeSettings panel/modal.
///
/// Provides:
/// - Color pickers for all theme tokens, grouped by category.
/// - Preset selector (built-in and custom).
/// - Save/rename/delete for custom themes.
/// - JSON export and import.
/// - Live preview of pending changes (injected as an overriding `<style>`).
/// - Undo-changes action (reverts draft to last committed state).
///
/// Access [`ThemeStore`] via context (mount [`crate::store::ThemeProvider`] near
/// the app root) or pass the store through a prop if preferred.
///
/// `on_close` is called when the user dismisses the panel.
#[component]
pub fn ThemeSettings(
    /// Called when the user clicks the close / dismiss button.
    #[props(default)]
    on_close: EventHandler<()>,
    /// Extra CSS classes on the root panel element.
    #[props(default)]
    class: String,
) -> Element {
    // ── Store access ──
    let mut store = use_context::<ThemeStore>();

    // ── Local state ──
    // Draft: the colors currently being edited (starts from active preset).
    let mut draft = use_signal(|| ThemeSnapshot::from_colors(store.colors()));
    // Committed snapshot — used to implement "undo changes".
    let mut committed = use_signal(|| ThemeSnapshot::from_colors(store.colors()));
    // Custom themes loaded from / persisted to localStorage.
    let mut custom_themes: Signal<Vec<CustomTheme>> = use_signal(load_custom_themes);
    // Name input for saving a new custom theme.
    let mut save_name = use_signal(String::new);
    // Rename target (custom theme index) and new name.
    let mut rename_idx: Signal<Option<usize>> = use_signal(|| None);
    let mut rename_name = use_signal(String::new);
    // Import textarea content.
    let mut import_json = use_signal(String::new);
    // Error / info message shown in the panel.
    let mut message: Signal<Option<(bool, String)>> = use_signal(|| None); // (is_error, text)

    // Clean up preview style on unmount.
    use_drop(|| remove_preview_css());

    // Inject live preview whenever the draft changes.
    {
        let d = draft.read().clone();
        use_effect(move || {
            inject_preview_css(&d);
        });
    }

    let panel_class = if class.is_empty() {
        "theme-settings glass-panel".to_string()
    } else {
        format!("theme-settings glass-panel {class}")
    };

    rsx! {
        div {
            class: "{panel_class}",
            role: "dialog",
            aria_label: "Theme settings",

            // ── Header ──
            div {
                class: "glass-panel__header theme-settings__header",
                span { class: "glass-panel__title", "Theme Settings" }
                button {
                    class: "tab-close",
                    aria_label: "Close theme settings",
                    onclick: move |_| {
                        // Remove preview and revert to saved preset.
                        remove_preview_css();
                        on_close.call(());
                    },
                    "✕"
                }
            }

            div {
                class: "theme-settings__body",

                // ── Preset selector ──
                section {
                    class: "theme-settings__section",
                    h3 { class: "theme-settings__section-title", "Preset" }
                    div {
                        class: "theme-settings__preset-row",
                        // Built-in presets
                        for (preset_key, preset_label) in [
                            ("arcadia", "Arcadia"),
                            ("dark", "Dark"),
                            ("paper", "Paper"),
                            ("scratchboard", "Scratchboard"),
                        ] {
                            {
                                let active = store.preset().key() == preset_key;
                                let colors: &ThemeColors = match preset_key {
                                    "arcadia" => &ARCADIA,
                                    "dark" => &DARK,
                                    "paper" => &PAPER,
                                    _ => &SCRATCHBOARD,
                                };
                                let snap = ThemeSnapshot::from_colors(colors);
                                rsx! {
                                    button {
                                        key: "{preset_key}",
                                        class: if active { "theme-settings__preset-btn theme-settings__preset-btn--active" }
                                               else { "theme-settings__preset-btn" },
                                        onclick: move |_| {
                                            if let Some(p) = ThemePreset::from_key(preset_key) {
                                                store.apply_preset(p);
                                            }
                                            let new_snap = snap.clone();
                                            draft.set(new_snap.clone());
                                            committed.set(new_snap);
                                            message.set(None);
                                        },
                                        "{preset_label}"
                                    }
                                }
                            }
                        }
                        // Custom theme buttons
                        for (idx, ct) in custom_themes.read().iter().enumerate() {
                            {
                                let ct_clone = ct.clone();
                                let ct_name = ct.name.clone();
                                rsx! {
                                    button {
                                        key: "custom-{idx}",
                                        class: "theme-settings__preset-btn",
                                        onclick: move |_| {
                                            draft.set(ct_clone.colors.clone());
                                            committed.set(ct_clone.colors.clone());
                                            inject_preview_css(&ct_clone.colors);
                                            message.set(Some((false, format!("Loaded \"{ct_name}\""))));
                                        },
                                        "{ct.name}"
                                    }
                                }
                            }
                        }
                    }
                }

                // ── Effects toggle (master switch for WgpuOverlay) ──
                section {
                    class: "theme-settings__section",
                    h3 { class: "theme-settings__section-title", "Effects" }
                    p { class: "theme-settings__section-hint",
                        "Animated background — smoke, sparks/embers/beams (\"angelic beam\" particle simulation), and CRT scanlines. ",
                        "Disabled by default to keep the viewer lightweight; enable to show the full visual treatment."
                    }
                    div {
                        class: "theme-settings__effect-row",
                        div {
                            class: "theme-settings__effect-info",
                            span { class: "theme-settings__effect-label", "Enable GPU overlay" }
                            span { class: "theme-settings__effect-desc",
                                "Master switch — toggles smoke, particles, and CRT effects."
                            }
                        }
                        label {
                            class: "theme-settings__toggle-switch",
                            aria_label: "Toggle GPU overlay effects",
                            input {
                                r#type: "checkbox",
                                checked: store.gpu_enabled(),
                                onchange: move |e: Event<FormData>| {
                                    let on = e.value() == "true";
                                    store.set_gpu_enabled(on);
                                    message.set(Some((false,
                                        if on { "GPU overlay enabled.".into() }
                                        else  { "GPU overlay disabled.".into() })));
                                },
                            }
                            span { class: "theme-settings__toggle-slider" }
                        }
                    }
                }

                // ── Color token groups ──
                for group in TOKEN_GROUPS {
                    section {
                        key: "{group.title}",
                        class: "theme-settings__section",
                        h3 { class: "theme-settings__section-title", "{group.title}" }
                        div {
                            class: "theme-settings__token-grid",
                            for (token_key, token_label) in group.tokens.iter() {
                                {
                                    let tk = *token_key;
                                    let current_val = get_token(&draft.read(), tk).to_string();
                                    rsx! {
                                        label {
                                            key: "{tk}",
                                            class: "theme-settings__token-row",
                                            span { class: "theme-settings__token-label", "{token_label}" }
                                            // Colour swatch preview
                                            span {
                                                class: "theme-settings__token-swatch",
                                                style: "background: {current_val};",
                                            }
                                            input {
                                                r#type: "color",
                                                class: "theme-settings__color-input",
                                                value: "{current_val}",
                                                aria_label: "{token_label} color",
                                                oninput: move |e| {
                                                    let val = e.value();
                                                    let mut d = draft.write();
                                                    set_token(&mut d, tk, val);
                                                },
                                            }
                                            // Hex text input
                                            input {
                                                r#type: "text",
                                                class: "theme-settings__hex-input",
                                                value: "{current_val}",
                                                aria_label: "{token_label} hex value",
                                                maxlength: "7",
                                                oninput: move |e| {
                                                    let val = e.value();
                                                    if val.starts_with('#') && (val.len() == 4 || val.len() == 7) {
                                                        let mut d = draft.write();
                                                        set_token(&mut d, tk, val);
                                                    }
                                                },
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // ── Save custom theme ──
                section {
                    class: "theme-settings__section",
                    h3 { class: "theme-settings__section-title", "Save Custom Theme" }
                    div {
                        class: "theme-settings__save-row",
                        input {
                            r#type: "text",
                            class: "theme-settings__name-input",
                            placeholder: "Theme name…",
                            value: "{save_name}",
                            aria_label: "Custom theme name",
                            oninput: move |e| save_name.set(e.value()),
                        }
                        button {
                            class: "theme-settings__action-btn",
                            disabled: save_name.read().trim().is_empty(),
                            onclick: move |_| {
                                let name = save_name.read().trim().to_string();
                                if name.is_empty() { return; }
                                let new_theme = CustomTheme {
                                    name: name.clone(),
                                    colors: draft.read().clone(),
                                };
                                let mut list = custom_themes.write();
                                // Replace existing by name, else push.
                                if let Some(pos) = list.iter().position(|t| t.name == name) {
                                    list[pos] = new_theme;
                                } else {
                                    list.push(new_theme);
                                }
                                save_custom_themes_storage(&list);
                                drop(list);
                                save_name.set(String::new());
                                message.set(Some((false, format!("Saved \"{name}\""))));
                            },
                            "Save"
                        }
                    }

                    // Rename / delete existing custom themes
                    if !custom_themes.read().is_empty() {
                        div {
                            class: "theme-settings__custom-list",
                            for (idx, ct) in custom_themes.read().iter().enumerate() {
                                {
                                    let ct_name = ct.name.clone();
                                    let ct_name_rename = ct_name.clone();
                                    let ct_name_delete = ct_name.clone();
                                    let is_renaming = rename_idx.read().map_or(false, |i| i == idx);
                                    rsx! {
                                        div {
                                            key: "ct-{idx}",
                                            class: "theme-settings__custom-row",
                                            if is_renaming {
                                                input {
                                                    r#type: "text",
                                                    class: "theme-settings__name-input",
                                                    value: "{rename_name}",
                                                    aria_label: "New name for {ct_name}",
                                                    oninput: move |e| rename_name.set(e.value()),
                                                }
                                                button {
                                                    class: "theme-settings__action-btn",
                                                    onclick: move |_| {
                                                        let new_name = rename_name.read().trim().to_string();
                                                        if !new_name.is_empty() {
                                                            let mut list = custom_themes.write();
                                                            if let Some(i) = *rename_idx.read() {
                                                                if let Some(t) = list.get_mut(i) {
                                                                    t.name = new_name.clone();
                                                                }
                                                            }
                                                            save_custom_themes_storage(&list);
                                                        }
                                                        rename_idx.set(None);
                                                        rename_name.set(String::new());
                                                        message.set(Some((false, "Renamed.".into())));
                                                    },
                                                    "OK"
                                                }
                                                button {
                                                    class: "theme-settings__action-btn",
                                                    onclick: move |_| {
                                                        rename_idx.set(None);
                                                        rename_name.set(String::new());
                                                    },
                                                    "Cancel"
                                                }
                                            } else {
                                                span { class: "theme-settings__custom-name", "{ct_name}" }
                                                button {
                                                    class: "theme-settings__action-btn",
                                                    aria_label: "Rename {ct_name}",
                                                    onclick: move |_| {
                                                        rename_idx.set(Some(idx));
                                                        rename_name.set(ct_name_rename.clone());
                                                    },
                                                    "Rename"
                                                }
                                                button {
                                                    class: "theme-settings__action-btn theme-settings__action-btn--danger",
                                                    aria_label: "Delete {ct_name}",
                                                    onclick: move |_| {
                                                        let del_name = ct_name_delete.clone();
                                                        let mut list = custom_themes.write();
                                                        list.retain(|t| t.name != del_name);
                                                        save_custom_themes_storage(&list);
                                                        drop(list);
                                                        message.set(Some((false, format!("Deleted \"{del_name}\""))));
                                                    },
                                                    "Delete"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // ── Export / Import JSON ──
                section {
                    class: "theme-settings__section",
                    h3 { class: "theme-settings__section-title", "Export / Import JSON" }
                    div {
                        class: "theme-settings__export-row",
                        button {
                            class: "theme-settings__action-btn",
                            aria_label: "Copy theme JSON to clipboard",
                            onclick: move |_| {
                                let json = draft.read().to_json();
                                import_json.set(json.clone());
                                #[cfg(target_arch = "wasm32")]
                                {
                                    if let Some(w) = web_sys::window() {
                                        let _ = w.navigator().clipboard().write_text(&json);
                                    }
                                }
                                message.set(Some((false, "JSON copied to clipboard.".into())));
                            },
                            "Export (copy)"
                        }
                    }
                    textarea {
                        class: "theme-settings__import-textarea",
                        rows: "4",
                        placeholder: "Paste JSON here to import…",
                        aria_label: "Import theme JSON",
                        value: "{import_json}",
                        oninput: move |e| import_json.set(e.value()),
                    }
                    button {
                        class: "theme-settings__action-btn",
                        onclick: move |_| {
                            let raw = import_json.read().clone();
                            if let Some(snap) = ThemeSnapshot::from_json(&raw) {
                                draft.set(snap.clone());
                                inject_preview_css(&snap);
                                message.set(Some((false, "Theme imported — adjust and save.".into())));
                            } else {
                                message.set(Some((true, "Invalid JSON — import failed.".into())));
                            }
                        },
                        "Import"
                    }
                }

                // ── Actions footer ──
                div {
                    class: "theme-settings__footer",

                    // Undo changes
                    button {
                        class: "theme-settings__action-btn",
                        title: "Revert all unsaved edits",
                        onclick: move |_| {
                            let saved = committed.read().clone();
                            draft.set(saved.clone());
                            inject_preview_css(&saved);
                            message.set(Some((false, "Changes reverted.".into())));
                        },
                        "Undo changes"
                    }

                    // Apply to active preset slot (overwrites CSS variables globally)
                    button {
                        class: "theme-settings__action-btn theme-settings__action-btn--primary",
                        title: "Apply current draft as the active theme",
                        onclick: move |_| {
                            let snap = draft.read().clone();
                            committed.set(snap.clone());
                            // Persist via ThemeStore (re-inject the base style).
                            // We keep the preview style since it exactly represents
                            // the draft tokens; the base style will match on next
                            // preset switch.
                            inject_preview_css(&snap);
                            message.set(Some((false, "Theme applied.".into())));
                        },
                        "Apply"
                    }
                }

                // ── Status message ──
                if let Some((is_error, msg)) = message.read().as_ref() {
                    div {
                        class: if *is_error { "theme-settings__message theme-settings__message--error" }
                               else { "theme-settings__message" },
                        "{msg}"
                    }
                }
            }
        }
    }
}
