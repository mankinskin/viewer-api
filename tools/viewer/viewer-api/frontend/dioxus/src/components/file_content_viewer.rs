//! FileContentViewer — detects file type and renders appropriately.
//!
//! - `.md` files → pulldown-cmark Markdown → HTML with `.markdown-body` styling.
//! - All other files → `CodeViewer` with syntect syntax highlighting.
//! - Optional `custom_renderer` callback: if provided and returns `Some(Element)`,
//!   that element is used instead of the default rendering.
use dioxus::prelude::*;

use crate::components::CodeViewer;

fn is_markdown(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    lower.ends_with(".md") || lower.ends_with(".mdx") || lower.ends_with(".markdown")
}

fn render_markdown(content: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};

    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(content, opts);
    let mut html_buf = String::with_capacity(content.len() * 2);
    html::push_html(&mut html_buf, parser);
    html_buf
}

/// Displays file content with automatic type detection.
///
/// If `custom_renderer` is provided it receives `(filename, content)` and can
/// return `Some(Element)` to override the default rendering.
#[component]
pub fn FileContentViewer(
    content: String,
    #[props(default)]
    filename: String,
    #[props(default)]
    language: Option<String>,
    /// 1-based line to highlight (forwarded to CodeViewer).
    #[props(default)]
    highlighted_line: Option<usize>,
    #[props(default = true)]
    show_line_numbers: bool,
    /// Optional override renderer: receives `(filename, content)`, returns
    /// `Some(Element)` to take over rendering, or `None` for default.
    #[props(default)]
    custom_renderer: Option<Callback<(String, String), Option<Element>>>,
    #[props(default)]
    class: String,
) -> Element {
    // Check custom renderer first.
    if let Some(renderer) = &custom_renderer {
        if let Some(el) = renderer.call((filename.clone(), content.clone())) {
            return el;
        }
    }

    if is_markdown(&filename) {
        let html = render_markdown(&content);
        let outer_css = if class.is_empty() {
            "markdown-body".to_string()
        } else {
            format!("markdown-body {class}")
        };
        return rsx! {
            div {
                class: "{outer_css}",
                dangerous_inner_html: "{html}",
            }
        };
    }

    rsx! {
        CodeViewer {
            content,
            filename,
            language,
            highlighted_line,
            show_line_numbers,
            class,
        }
    }
}
