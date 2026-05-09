use crate::store::ThemeColors;

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
    pub fn from_colors(colors: &ThemeColors) -> Self {
        Self {
            bg_primary: colors.bg_primary.to_string(),
            bg_secondary: colors.bg_secondary.to_string(),
            bg_tertiary: colors.bg_tertiary.to_string(),
            bg_elevated: colors.bg_elevated.to_string(),
            text_primary: colors.text_primary.to_string(),
            text_secondary: colors.text_secondary.to_string(),
            text_muted: colors.text_muted.to_string(),
            border_primary: colors.border_primary.to_string(),
            border_secondary: colors.border_secondary.to_string(),
            accent_blue: colors.accent_blue.to_string(),
            accent_purple: colors.accent_purple.to_string(),
            accent_green: colors.accent_green.to_string(),
            accent_yellow: colors.accent_yellow.to_string(),
            accent_red: colors.accent_red.to_string(),
            accent_orange: colors.accent_orange.to_string(),
            accent_cyan: colors.accent_cyan.to_string(),
            syntax_keyword: colors.syntax_keyword.to_string(),
            syntax_string: colors.syntax_string.to_string(),
            syntax_comment: colors.syntax_comment.to_string(),
            syntax_number: colors.syntax_number.to_string(),
            syntax_function: colors.syntax_function.to_string(),
            syntax_type: colors.syntax_type.to_string(),
            syntax_variable: colors.syntax_variable.to_string(),
        }
    }

    /// Serialize to a minimal JSON string for export / localStorage.
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"bg_primary":{bg_primary},"bg_secondary":{bg_secondary},"bg_tertiary":{bg_tertiary},"bg_elevated":{bg_elevated},"text_primary":{text_primary},"text_secondary":{text_secondary},"text_muted":{text_muted},"border_primary":{border_primary},"border_secondary":{border_secondary},"accent_blue":{accent_blue},"accent_purple":{accent_purple},"accent_green":{accent_green},"accent_yellow":{accent_yellow},"accent_red":{accent_red},"accent_orange":{accent_orange},"accent_cyan":{accent_cyan},"syntax_keyword":{syntax_keyword},"syntax_string":{syntax_string},"syntax_comment":{syntax_comment},"syntax_number":{syntax_number},"syntax_function":{syntax_function},"syntax_type":{syntax_type},"syntax_variable":{syntax_variable}}}"#,
            bg_primary = json_str(&self.bg_primary),
            bg_secondary = json_str(&self.bg_secondary),
            bg_tertiary = json_str(&self.bg_tertiary),
            bg_elevated = json_str(&self.bg_elevated),
            text_primary = json_str(&self.text_primary),
            text_secondary = json_str(&self.text_secondary),
            text_muted = json_str(&self.text_muted),
            border_primary = json_str(&self.border_primary),
            border_secondary = json_str(&self.border_secondary),
            accent_blue = json_str(&self.accent_blue),
            accent_purple = json_str(&self.accent_purple),
            accent_green = json_str(&self.accent_green),
            accent_yellow = json_str(&self.accent_yellow),
            accent_red = json_str(&self.accent_red),
            accent_orange = json_str(&self.accent_orange),
            accent_cyan = json_str(&self.accent_cyan),
            syntax_keyword = json_str(&self.syntax_keyword),
            syntax_string = json_str(&self.syntax_string),
            syntax_comment = json_str(&self.syntax_comment),
            syntax_number = json_str(&self.syntax_number),
            syntax_function = json_str(&self.syntax_function),
            syntax_type = json_str(&self.syntax_type),
            syntax_variable = json_str(&self.syntax_variable),
        )
    }

    /// Deserialize from the simple JSON format produced by [`to_json`].
    pub fn from_json(json: &str) -> Option<Self> {
        Some(Self {
            bg_primary: extract_json_value(json, "bg_primary")?,
            bg_secondary: extract_json_value(json, "bg_secondary")?,
            bg_tertiary: extract_json_value(json, "bg_tertiary")?,
            bg_elevated: extract_json_value(json, "bg_elevated")?,
            text_primary: extract_json_value(json, "text_primary")?,
            text_secondary: extract_json_value(json, "text_secondary")?,
            text_muted: extract_json_value(json, "text_muted")?,
            border_primary: extract_json_value(json, "border_primary")?,
            border_secondary: extract_json_value(json, "border_secondary")?,
            accent_blue: extract_json_value(json, "accent_blue")?,
            accent_purple: extract_json_value(json, "accent_purple")?,
            accent_green: extract_json_value(json, "accent_green")?,
            accent_yellow: extract_json_value(json, "accent_yellow")?,
            accent_red: extract_json_value(json, "accent_red")?,
            accent_orange: extract_json_value(json, "accent_orange")?,
            accent_cyan: extract_json_value(json, "accent_cyan")?,
            syntax_keyword: extract_json_value(json, "syntax_keyword")?,
            syntax_string: extract_json_value(json, "syntax_string")?,
            syntax_comment: extract_json_value(json, "syntax_comment")?,
            syntax_number: extract_json_value(json, "syntax_number")?,
            syntax_function: extract_json_value(json, "syntax_function")?,
            syntax_type: extract_json_value(json, "syntax_type")?,
            syntax_variable: extract_json_value(json, "syntax_variable")?,
        })
    }
}

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
        let name = extract_json_value(json, "name")?;
        let colors_needle = "\"colors\":";
        let colors_start = json.find(colors_needle)? + colors_needle.len();
        let colors = ThemeSnapshot::from_json(&json[colors_start..])?;
        Some(Self { name, colors })
    }
}

pub(super) const CUSTOM_THEMES_KEY: &str = "viewer-api-custom-themes";

#[cfg(target_arch = "wasm32")]
pub(super) fn load_custom_themes() -> Vec<CustomTheme> {
    let Some(storage) = web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
    else {
        return vec![];
    };
    let Ok(Some(raw)) = storage.get_item(CUSTOM_THEMES_KEY) else {
        return vec![];
    };
    parse_custom_themes_json(&raw)
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn load_custom_themes() -> Vec<CustomTheme> {
    vec![]
}

#[cfg(target_arch = "wasm32")]
pub(super) fn save_custom_themes_storage(themes: &[CustomTheme]) {
    if let Some(storage) = web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
    {
        let json = serialize_custom_themes(themes);
        let _ = storage.set_item(CUSTOM_THEMES_KEY, &json);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn save_custom_themes_storage(_themes: &[CustomTheme]) {}

fn serialize_custom_themes(themes: &[CustomTheme]) -> String {
    let items: Vec<String> = themes.iter().map(CustomTheme::to_json).collect();
    format!("[{}]", items.join(","))
}

fn parse_custom_themes_json(json: &str) -> Vec<CustomTheme> {
    let mut themes = Vec::new();
    let trimmed = json.trim();
    if !trimmed.starts_with('[') {
        return themes;
    }

    let mut remainder = trimmed.trim_start_matches('[').trim_end_matches(']');
    while let Some(start) = remainder.find("{\"name\":") {
        remainder = &remainder[start..];
        let Some(end) = find_json_block_end(remainder) else {
            break;
        };
        if let Some(theme) = CustomTheme::from_json(&remainder[..end]) {
            themes.push(theme);
        }
        remainder = &remainder[end..];
    }
    themes
}

fn extract_json_value(
    json: &str,
    key: &str,
) -> Option<String> {
    let needle = format!("\"{}\":\"", key);
    let start = json.find(&needle)? + needle.len();
    let rest = &json[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn find_json_block_end(json: &str) -> Option<usize> {
    let mut depth = 0;
    for (index, ch) in json.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index + 1);
                }
            },
            _ => {},
        }
    }
    None
}

fn json_str(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r");
    format!("\"{}\"", escaped)
}
