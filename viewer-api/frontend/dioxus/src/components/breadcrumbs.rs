//! Breadcrumbs — a horizontal path of clickable segments.
//!
//! Render a sequence of [`BreadcrumbItem`]s separated by chevron icons.
//! Items with `on_click` set become clickable; the last item is rendered
//! as the "current" segment (no click handler, distinct styling).
//!
//! ## Usage
//!
//! ```ignore
//! Breadcrumbs {
//!     items: vec![
//!         BreadcrumbItem::link("Home", move |_| go_home()),
//!         BreadcrumbItem::link("Specs", move |_| go_specs()),
//!         BreadcrumbItem::current("my-spec"),
//!     ],
//! }
//! ```
//!
//! CSS lives in `viewer-api/public/css/breadcrumbs.css`.
use dioxus::prelude::*;

use crate::components::ChevronRightIcon;

/// A single segment in a [`Breadcrumbs`] trail.
///
/// Use [`BreadcrumbItem::link`] for clickable parents and
/// [`BreadcrumbItem::current`] for the (non-clickable) leaf.
#[derive(Clone, PartialEq)]
pub struct BreadcrumbItem {
    pub label: String,
    /// Optional click handler.  When `None`, the segment is rendered as
    /// non-interactive text (used for the current/leaf segment).
    pub on_click: Option<EventHandler<()>>,
    /// Optional `href`.  When `Some`, the segment renders as an `<a>` tag
    /// with the given `href` so middle-click / open-in-new-tab works.
    /// `on_click` is still invoked on plain left-click and may
    /// `preventDefault()` to avoid full navigation.
    pub href: Option<String>,
}

impl BreadcrumbItem {
    /// Construct a clickable breadcrumb segment.
    pub fn link(label: impl Into<String>, on_click: EventHandler<()>) -> Self {
        Self {
            label: label.into(),
            on_click: Some(on_click),
            href: None,
        }
    }

    /// Construct the current/leaf breadcrumb (non-clickable).
    pub fn current(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            on_click: None,
            href: None,
        }
    }

    /// Attach an `href` to a clickable segment for proper anchor semantics.
    pub fn with_href(mut self, href: impl Into<String>) -> Self {
        self.href = Some(href.into());
        self
    }
}

/// Horizontal breadcrumb trail.
///
/// Renders `<nav class="breadcrumbs">` with one `<span>` (or `<a>`) per
/// item separated by chevron icons.  Items with `on_click` get the
/// `.breadcrumbs__item--clickable` modifier; the final item gets
/// `.breadcrumbs__item--current`.
#[component]
pub fn Breadcrumbs(
    items: Vec<BreadcrumbItem>,
    /// Extra CSS classes on the root `.breadcrumbs` nav.
    #[props(default)]
    class: String,
) -> Element {
    let combined = if class.is_empty() {
        "breadcrumbs".to_string()
    } else {
        format!("breadcrumbs {class}")
    };

    let last = items.len().saturating_sub(1);

    rsx! {
        nav {
            class: "{combined}",
            aria_label: "Breadcrumb",

            for (i, item) in items.into_iter().enumerate() {
                BreadcrumbSegment {
                    key: "{i}",
                    item: item,
                    is_last: i == last,
                }
                if i != last {
                    span {
                        class: "breadcrumbs__sep",
                        aria_hidden: "true",
                        ChevronRightIcon { size: 12 }
                    }
                }
            }
        }
    }
}

#[component]
fn BreadcrumbSegment(item: BreadcrumbItem, is_last: bool) -> Element {
    let mut class_str = String::from("breadcrumbs__item");
    if item.on_click.is_some() && !is_last {
        class_str.push_str(" breadcrumbs__item--clickable");
    }
    if is_last {
        class_str.push_str(" breadcrumbs__item--current");
    }

    let label = item.label.clone();
    let on_click = item.on_click.clone();
    let href = item.href.clone();

    if let (Some(handler), Some(h)) = (on_click.clone(), href.clone()) {
        rsx! {
            a {
                class: "{class_str}",
                href: "{h}",
                onclick: move |evt| {
                    evt.prevent_default();
                    handler.call(());
                },
                "{label}"
            }
        }
    } else if let Some(handler) = on_click {
        rsx! {
            button {
                r#type: "button",
                class: "{class_str}",
                onclick: move |_| handler.call(()),
                "{label}"
            }
        }
    } else {
        rsx! {
            span {
                class: "{class_str}",
                aria_current: if is_last { "page" } else { "false" },
                "{label}"
            }
        }
    }
}
