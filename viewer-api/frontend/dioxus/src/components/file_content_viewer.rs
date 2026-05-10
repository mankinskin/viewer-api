//! FileContentViewer — detects file type and renders appropriately.
//!
//! - `.md` files → pulldown-cmark Markdown → HTML with `.markdown-body` styling.
//! - All other files → `CodeViewer` with syntect syntax highlighting.
//! - Optional `custom_renderer` callback: if provided and returns `Some(Element)`,
//!   that element is used instead of the default rendering.
use std::cell::RefCell;

use dioxus::prelude::*;

use crate::{
    components::CodeViewer,
    store::Prefetcher,
};

const MARKDOWN_HTML_CACHE_CAPACITY: usize = 256;

thread_local! {
    static MARKDOWN_HTML_CACHE: RefCell<Prefetcher<String, String>> =
        RefCell::new(Prefetcher::with_capacity(MARKDOWN_HTML_CACHE_CAPACITY));
}

fn markdown_class(class: &str) -> String {
    if class.is_empty() {
        "markdown-body".to_string()
    } else {
        format!("markdown-body {class}")
    }
}

fn is_markdown(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    lower.ends_with(".md")
        || lower.ends_with(".mdx")
        || lower.ends_with(".markdown")
}

fn render_markdown(content: &str) -> String {
    use pulldown_cmark::{
        html,
        Options,
        Parser,
    };

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

fn render_markdown_cached(content: &str) -> String {
    let key = content.to_string();
    MARKDOWN_HTML_CACHE.with(|cache| {
        let cache = cache.borrow();
        if let Some(html) = cache.get(&key) {
            return html;
        }

        let html = render_markdown(content);
        cache.insert(key, html.clone());
        html
    })
}

#[component]
pub fn MarkdownContent(
    content: String,
    #[props(default)] class: String,
) -> Element {
    let html = render_markdown_cached(&content);
    let outer_css = markdown_class(&class);

    rsx! {
        div {
            class: "{outer_css}",
            dangerous_inner_html: "{html}",
        }
    }
}

/// Displays file content with automatic type detection.
///
/// If `custom_renderer` is provided it receives `(filename, content)` and can
/// return `Some(Element)` to override the default rendering.
#[component]
pub fn FileContentViewer(
    content: String,
    #[props(default)] filename: String,
    #[props(default)] language: Option<String>,
    /// 1-based line to highlight (forwarded to CodeViewer).
    #[props(default)]
    highlighted_line: Option<usize>,
    #[props(default = true)] show_line_numbers: bool,
    /// Optional override renderer: receives `(filename, content)`, returns
    /// `Some(Element)` to take over rendering, or `None` for default.
    #[props(default)]
    custom_renderer: Option<Callback<(String, String), Option<Element>>>,
    #[props(default)] class: String,
) -> Element {
    // Check custom renderer first.
    if let Some(renderer) = &custom_renderer {
        if let Some(el) = renderer.call((filename.clone(), content.clone())) {
            return el;
        }
    }

    if is_markdown(&filename) {
        return rsx! {
            MarkdownContent {
                content,
                class,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn reset_markdown_cache() {
        MARKDOWN_HTML_CACHE.with(|cache| {
            *cache.borrow_mut() =
                Prefetcher::with_capacity(MARKDOWN_HTML_CACHE_CAPACITY);
        });
    }

    fn markdown_cache_len() -> usize {
        MARKDOWN_HTML_CACHE.with(|cache| cache.borrow().len())
    }

    #[test]
    fn cached_markdown_reuses_baked_html_for_same_content() {
        reset_markdown_cache();

        let first = render_markdown_cached("**hello**");
        let second = render_markdown_cached("**hello**");

        assert_eq!(first, "<p><strong>hello</strong></p>\n");
        assert_eq!(second, first);
        assert_eq!(markdown_cache_len(), 1);
    }

    #[test]
    fn cached_markdown_tracks_distinct_content_entries() {
        reset_markdown_cache();

        let _ = render_markdown_cached("first");
        let _ = render_markdown_cached("second");

        assert_eq!(markdown_cache_len(), 2);
    }
}
