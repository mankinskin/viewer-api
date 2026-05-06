//! Card / CardGrid / CardSection — landing-page primitives.
//!
//! Mirrors the doc-viewer category landing layout: an optional icon, a
//! title, an optional description, an optional badge, and a responsive
//! grid container.
//!
//! CSS: `viewer-api/public/css/cards.css`.
use dioxus::prelude::*;

/// A single clickable card with icon, title, optional description, and
/// optional badge.
///
/// When `on_click` is `None` the card is rendered as a `<div>` rather
/// than a `<button>` and is not focusable / interactive.
#[component]
pub fn Card(
    title: String,
    #[props(default)]
    description: Option<String>,
    /// Optional small text rendered in the top-right corner (e.g. a count).
    #[props(default)]
    badge: Option<String>,
    /// Optional icon element rendered to the left of the title.
    #[props(default)]
    icon: Option<Element>,
    /// Optional click handler.  When `Some`, the card renders as a button.
    #[props(default)]
    on_click: Option<EventHandler<()>>,
    /// Extra CSS classes appended to the root `.card` element.
    #[props(default)]
    class: String,
) -> Element {
    let mut class_str = String::from("card");
    if on_click.is_some() {
        class_str.push_str(" card--clickable");
    }
    if !class.is_empty() {
        class_str.push(' ');
        class_str.push_str(&class);
    }

    let body = rsx! {
        if let Some(b) = badge {
            span { class: "card__badge", "{b}" }
        }
        div {
            class: "card__head",
            if let Some(i) = icon {
                span { class: "card__icon", {i} }
            }
            h3 { class: "card__title", "{title}" }
        }
        if let Some(d) = description {
            p { class: "card__description", "{d}" }
        }
    };

    if let Some(handler) = on_click {
        rsx! {
            button {
                r#type: "button",
                class: "{class_str}",
                onclick: move |_| handler.call(()),
                {body}
            }
        }
    } else {
        rsx! {
            div { class: "{class_str}", {body} }
        }
    }
}

/// Responsive auto-fit grid of [`Card`]s.  Use as a parent of one or more
/// cards.
#[component]
pub fn CardGrid(
    children: Element,
    #[props(default)]
    class: String,
) -> Element {
    let combined = if class.is_empty() {
        "card-grid".to_string()
    } else {
        format!("card-grid {class}")
    };
    rsx! {
        div { class: "{combined}", {children} }
    }
}

/// A titled section grouping a [`CardGrid`] underneath.
///
/// Renders an `<h2>` with optional count badge and the children below.
#[component]
pub fn CardSection(
    title: String,
    #[props(default)]
    count: Option<usize>,
    children: Element,
    #[props(default)]
    class: String,
) -> Element {
    let combined = if class.is_empty() {
        "card-section".to_string()
    } else {
        format!("card-section {class}")
    };
    rsx! {
        section {
            class: "{combined}",
            h2 {
                class: "card-section__title",
                "{title}"
                if let Some(n) = count {
                    span { class: "card-section__count", "({n})" }
                }
            }
            {children}
        }
    }
}
