use dioxus::prelude::*;

use super::model::ThemeSnapshot;

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

pub(super) fn get_token<'a>(
    snapshot: &'a ThemeSnapshot,
    key: &str,
) -> &'a str {
    background_token(snapshot, key)
        .or_else(|| text_token(snapshot, key))
        .or_else(|| border_token(snapshot, key))
        .or_else(|| accent_token(snapshot, key))
        .or_else(|| syntax_token(snapshot, key))
        .unwrap_or("")
}

pub(super) fn set_token(
    snapshot: &mut ThemeSnapshot,
    key: &str,
    value: String,
) {
    if set_background_token(snapshot, key, &value)
        || set_text_token(snapshot, key, &value)
        || set_border_token(snapshot, key, &value)
        || set_accent_token(snapshot, key, &value)
    {
        return;
    }
    let _ = set_syntax_token(snapshot, key, &value);
}

#[component]
pub(super) fn TokenSections(mut draft: Signal<ThemeSnapshot>) -> Element {
    rsx! {
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
                                    span {
                                        class: "theme-settings__token-swatch",
                                        style: "background: {current_val};",
                                    }
                                    input {
                                        r#type: "color",
                                        class: "theme-settings__color-input",
                                        value: "{current_val}",
                                        aria_label: "{token_label} color",
                                        oninput: move |event| {
                                            let value = event.value();
                                            let mut snapshot = draft.write();
                                            set_token(&mut snapshot, tk, value);
                                        },
                                    }
                                    input {
                                        r#type: "text",
                                        class: "theme-settings__hex-input",
                                        value: "{current_val}",
                                        aria_label: "{token_label} hex value",
                                        maxlength: "7",
                                        oninput: move |event| {
                                            let value = event.value();
                                            if value.starts_with('#') && (value.len() == 4 || value.len() == 7) {
                                                let mut snapshot = draft.write();
                                                set_token(&mut snapshot, tk, value);
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
    }
}

fn background_token<'a>(
    snapshot: &'a ThemeSnapshot,
    key: &str,
) -> Option<&'a str> {
    match key {
        "bg_primary" => Some(&snapshot.bg_primary),
        "bg_secondary" => Some(&snapshot.bg_secondary),
        "bg_tertiary" => Some(&snapshot.bg_tertiary),
        "bg_elevated" => Some(&snapshot.bg_elevated),
        _ => None,
    }
}

fn text_token<'a>(
    snapshot: &'a ThemeSnapshot,
    key: &str,
) -> Option<&'a str> {
    match key {
        "text_primary" => Some(&snapshot.text_primary),
        "text_secondary" => Some(&snapshot.text_secondary),
        "text_muted" => Some(&snapshot.text_muted),
        _ => None,
    }
}

fn border_token<'a>(
    snapshot: &'a ThemeSnapshot,
    key: &str,
) -> Option<&'a str> {
    match key {
        "border_primary" => Some(&snapshot.border_primary),
        "border_secondary" => Some(&snapshot.border_secondary),
        _ => None,
    }
}

fn accent_token<'a>(
    snapshot: &'a ThemeSnapshot,
    key: &str,
) -> Option<&'a str> {
    match key {
        "accent_blue" => Some(&snapshot.accent_blue),
        "accent_purple" => Some(&snapshot.accent_purple),
        "accent_green" => Some(&snapshot.accent_green),
        "accent_yellow" => Some(&snapshot.accent_yellow),
        "accent_red" => Some(&snapshot.accent_red),
        "accent_orange" => Some(&snapshot.accent_orange),
        "accent_cyan" => Some(&snapshot.accent_cyan),
        _ => None,
    }
}

fn syntax_token<'a>(
    snapshot: &'a ThemeSnapshot,
    key: &str,
) -> Option<&'a str> {
    match key {
        "syntax_keyword" => Some(&snapshot.syntax_keyword),
        "syntax_string" => Some(&snapshot.syntax_string),
        "syntax_comment" => Some(&snapshot.syntax_comment),
        "syntax_number" => Some(&snapshot.syntax_number),
        "syntax_function" => Some(&snapshot.syntax_function),
        "syntax_type" => Some(&snapshot.syntax_type),
        "syntax_variable" => Some(&snapshot.syntax_variable),
        _ => None,
    }
}

fn set_background_token(
    snapshot: &mut ThemeSnapshot,
    key: &str,
    value: &str,
) -> bool {
    match key {
        "bg_primary" => snapshot.bg_primary = value.to_string(),
        "bg_secondary" => snapshot.bg_secondary = value.to_string(),
        "bg_tertiary" => snapshot.bg_tertiary = value.to_string(),
        "bg_elevated" => snapshot.bg_elevated = value.to_string(),
        _ => return false,
    }
    true
}

fn set_text_token(
    snapshot: &mut ThemeSnapshot,
    key: &str,
    value: &str,
) -> bool {
    match key {
        "text_primary" => snapshot.text_primary = value.to_string(),
        "text_secondary" => snapshot.text_secondary = value.to_string(),
        "text_muted" => snapshot.text_muted = value.to_string(),
        _ => return false,
    }
    true
}

fn set_border_token(
    snapshot: &mut ThemeSnapshot,
    key: &str,
    value: &str,
) -> bool {
    match key {
        "border_primary" => snapshot.border_primary = value.to_string(),
        "border_secondary" => snapshot.border_secondary = value.to_string(),
        _ => return false,
    }
    true
}

fn set_accent_token(
    snapshot: &mut ThemeSnapshot,
    key: &str,
    value: &str,
) -> bool {
    match key {
        "accent_blue" => snapshot.accent_blue = value.to_string(),
        "accent_purple" => snapshot.accent_purple = value.to_string(),
        "accent_green" => snapshot.accent_green = value.to_string(),
        "accent_yellow" => snapshot.accent_yellow = value.to_string(),
        "accent_red" => snapshot.accent_red = value.to_string(),
        "accent_orange" => snapshot.accent_orange = value.to_string(),
        "accent_cyan" => snapshot.accent_cyan = value.to_string(),
        _ => return false,
    }
    true
}

fn set_syntax_token(
    snapshot: &mut ThemeSnapshot,
    key: &str,
    value: &str,
) -> bool {
    match key {
        "syntax_keyword" => snapshot.syntax_keyword = value.to_string(),
        "syntax_string" => snapshot.syntax_string = value.to_string(),
        "syntax_comment" => snapshot.syntax_comment = value.to_string(),
        "syntax_number" => snapshot.syntax_number = value.to_string(),
        "syntax_function" => snapshot.syntax_function = value.to_string(),
        "syntax_type" => snapshot.syntax_type = value.to_string(),
        "syntax_variable" => snapshot.syntax_variable = value.to_string(),
        _ => return false,
    }
    true
}
