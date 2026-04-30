//! MetaHeader — document/page title with a metadata row of chips.
//!
//! Pairs with [`Chip`] for tag/status pills.  Use this above markdown
//! bodies, spec details, or any "header" needing a title + meta line.
//!
//! CSS: `viewer-api/public/css/meta-header.css` and `chip.css`.
use dioxus::prelude::*;

/// Visual variants for [`Chip`].  Maps to a CSS modifier class.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChipKind {
    /// Neutral grey — default tag.
    Neutral,
    /// Blue accent — informational.
    Info,
    /// Green accent — success / active.
    Success,
    /// Yellow accent — warning / draft.
    Warning,
    /// Red accent — error / blocked.
    Danger,
    /// Purple accent — feature / spec.
    Feature,
}

impl ChipKind {
    fn css_modifier(self) -> &'static str {
        match self {
            ChipKind::Neutral => "chip--neutral",
            ChipKind::Info => "chip--info",
            ChipKind::Success => "chip--success",
            ChipKind::Warning => "chip--warning",
            ChipKind::Danger => "chip--danger",
            ChipKind::Feature => "chip--feature",
        }
    }
}

impl Default for ChipKind {
    fn default() -> Self {
        ChipKind::Neutral
    }
}

/// A small inline pill — used for tags, statuses, and counts.
#[component]
pub fn Chip(
    text: String,
    #[props(default)]
    kind: ChipKind,
    /// Extra CSS classes appended to the root `.chip` span.
    #[props(default)]
    class: String,
) -> Element {
    let mut class_str = format!("chip {}", kind.css_modifier());
    if !class.is_empty() {
        class_str.push(' ');
        class_str.push_str(&class);
    }
    rsx! {
        span { class: "{class_str}", "{text}" }
    }
}

/// Inline horizontal row of [`Chip`]s with consistent spacing.
#[component]
pub fn ChipRow(
    children: Element,
    #[props(default)]
    class: String,
) -> Element {
    let combined = if class.is_empty() {
        "chip-row".to_string()
    } else {
        format!("chip-row {class}")
    };
    rsx! {
        div { class: "{combined}", {children} }
    }
}

/// A title + metadata header block.
///
/// `tags` are rendered as neutral chips; `status` as an informational chip;
/// `date` as plain inline text.  Pass `None` / empty to omit.
#[component]
pub fn MetaHeader(
    title: String,
    #[props(default)]
    date: Option<String>,
    #[props(default)]
    tags: Vec<String>,
    #[props(default)]
    status: Option<String>,
    /// Optional content rendered below the meta row (e.g. a description).
    #[props(default)]
    children: Element,
    #[props(default)]
    class: String,
) -> Element {
    let combined = if class.is_empty() {
        "meta-header".to_string()
    } else {
        format!("meta-header {class}")
    };

    let has_meta_row = date.is_some() || !tags.is_empty() || status.is_some();

    rsx! {
        header {
            class: "{combined}",
            h1 { class: "meta-header__title", "{title}" }

            if has_meta_row {
                div {
                    class: "meta-header__meta",

                    if let Some(d) = date {
                        span { class: "meta-header__date", "{d}" }
                    }

                    if let Some(s) = status {
                        Chip { text: s, kind: ChipKind::Info, class: "meta-header__status".to_string() }
                    }

                    if !tags.is_empty() {
                        ChipRow {
                            class: "meta-header__tags".to_string(),
                            for tag in tags {
                                Chip {
                                    key: "{tag}",
                                    text: format!("#{tag}"),
                                    kind: ChipKind::Neutral,
                                }
                            }
                        }
                    }
                }
            }

            {children}
        }
    }
}
