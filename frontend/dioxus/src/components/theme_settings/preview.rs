#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

use super::model::ThemeSnapshot;

pub(super) const PREVIEW_STYLE_ID: &str = "viewer-api-theme-preview";

pub(super) fn inject_preview_css(snapshot: &ThemeSnapshot) {
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
            bg_primary = snapshot.bg_primary,
            bg_secondary = snapshot.bg_secondary,
            bg_tertiary = snapshot.bg_tertiary,
            bg_elevated = snapshot.bg_elevated,
            text_primary = snapshot.text_primary,
            text_secondary = snapshot.text_secondary,
            text_muted = snapshot.text_muted,
            border_primary = snapshot.border_primary,
            border_secondary = snapshot.border_secondary,
            accent_blue = snapshot.accent_blue,
            accent_purple = snapshot.accent_purple,
            accent_green = snapshot.accent_green,
            accent_yellow = snapshot.accent_yellow,
            accent_red = snapshot.accent_red,
            accent_orange = snapshot.accent_orange,
            accent_cyan = snapshot.accent_cyan,
            syntax_keyword = snapshot.syntax_keyword,
            syntax_string = snapshot.syntax_string,
            syntax_comment = snapshot.syntax_comment,
            syntax_number = snapshot.syntax_number,
            syntax_function = snapshot.syntax_function,
            syntax_type = snapshot.syntax_type,
            syntax_variable = snapshot.syntax_variable,
        );

        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                let element = if let Some(existing) =
                    document.get_element_by_id(PREVIEW_STYLE_ID)
                {
                    existing
                } else {
                    let new_element =
                        document.create_element("style").expect("create style");
                    new_element.set_id(PREVIEW_STYLE_ID);
                    if let Some(head) = document.head() {
                        let _ = head.append_child(&new_element);
                    }
                    new_element
                };
                element.set_text_content(Some(&css));
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = snapshot;
}

pub(super) fn remove_preview_css() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(element) =
                    document.get_element_by_id(PREVIEW_STYLE_ID)
                {
                    if let Some(parent) = element.parent_node() {
                        let _ = parent.remove_child(&element);
                    }
                }
            }
        }
    }
}
