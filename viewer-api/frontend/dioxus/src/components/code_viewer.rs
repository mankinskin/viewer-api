//! CodeViewer component — syntax-highlighted code display via syntect.
use dioxus::prelude::*;
use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    html::{styled_line_to_highlighted_html, IncludeBackground},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

/// Detect a syntect syntax name from a file extension or explicit language hint.
fn detect_syntax<'a>(ps: &'a SyntaxSet, filename: &str, language: Option<&str>) -> &'a syntect::parsing::SyntaxReference {
    // Try explicit language hint first.
    if let Some(lang) = language {
        if let Some(syntax) = ps.find_syntax_by_token(lang) {
            return syntax;
        }
        if let Some(syntax) = ps.find_syntax_by_name(lang) {
            return syntax;
        }
    }
    // Fall back to extension from filename.
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    if !ext.is_empty() {
        if let Some(syntax) = ps.find_syntax_by_extension(ext) {
            return syntax;
        }
    }
    ps.find_syntax_plain_text()
}

/// Build highlighted HTML from source code.
///
/// Each line is wrapped in a `<span class="code-line [highlight]">` with an
/// optional `data-line` attribute so CSS line-number counters work.
fn highlight_code(source: &str, filename: &str, language: Option<&str>, highlighted_line: Option<usize>) -> String {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    // We always use the syntect `base16-ocean.dark` theme as a base but
    // override token colours in CSS via the `.highlight .k` etc. selectors.
    // We use `IncludeBackground::No` so the theme background is ignored and
    // our CSS `--bg-*` variables take effect.
    let theme = &ts.themes["base16-ocean.dark"];
    let syntax = detect_syntax(&ps, filename, language);
    let mut h = HighlightLines::new(syntax, theme);

    let mut out = String::new();
    for (idx, line) in LinesWithEndings::from(source).enumerate() {
        let line_no = idx + 1;
        let is_highlighted = highlighted_line.map_or(false, |hl| hl == line_no);
        let regions = h.highlight_line(line, &ps).unwrap_or_default();
        let html = styled_line_to_highlighted_html(&regions, IncludeBackground::No)
            .unwrap_or_else(|_| html_escape(line));

        let cls = if is_highlighted {
            "code-line highlight"
        } else {
            "code-line"
        };
        out.push_str(&format!(
            r#"<span class="{cls}" data-line="{line_no}">{html}</span>"#
        ));
    }
    out
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Syntax-highlighted read-only code viewer.
///
/// Accepts `content` as a raw source string. If `language` is omitted the
/// syntax is inferred from `filename`.
///
/// Set `highlighted_line` to visually mark a specific 1-based line number.
#[component]
pub fn CodeViewer(
    content: String,
    #[props(default)]
    filename: String,
    #[props(default)]
    language: Option<String>,
    /// 1-based line number to highlight (scrolled to on mount).
    #[props(default)]
    highlighted_line: Option<usize>,
    #[props(default = true)]
    show_line_numbers: bool,
    #[props(default)]
    class: String,
) -> Element {
    // Compute lang_label from the originals before they are moved into use_memo.
    let lang_label = language.clone().unwrap_or_else(|| {
        std::path::Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("text")
            .to_string()
    });

    let html = use_memo({
        let filename = filename.clone();
        let language = language.clone();
        move || highlight_code(&content, &filename, language.as_deref(), highlighted_line)
    });

    let outer_css = if class.is_empty() {
        "code-viewer".to_string()
    } else {
        format!("code-viewer {class}")
    };

    let line_nums_class = if show_line_numbers {
        "code-lines code-lines--numbered"
    } else {
        "code-lines"
    };

    rsx! {
        div {
            class: "{outer_css}",
            if !filename.is_empty() {
                div {
                    class: "code-header",
                    span { class: "code-filename", "{filename}" }
                    span { class: "code-language", "{lang_label}" }
                }
            }
            div {
                class: "{line_nums_class}",
                pre {
                    class: "code-pre",
                    code {
                        class: "code-content",
                        dangerous_inner_html: "{html.read()}",
                    }
                }
            }
        }
    }
}
