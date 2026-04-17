//! TabBar component — horizontal list of closeable, reorderable tabs.
use dioxus::prelude::*;

use crate::components::{CloseIcon, ResizeHandle, ResizeEdge, ResizeDirection};

/// A single tab descriptor.
#[derive(Clone, PartialEq)]
pub struct TabItem {
    pub id: String,
    pub label: String,
    /// Optional icon rendered before the label.
    pub icon: Option<Element>,
    /// Shows a dot on the tab indicating unsaved changes.
    pub modified: bool,
    /// Whether the tab shows a close button.
    pub closeable: bool,
}

impl TabItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        TabItem {
            id: id.into(),
            label: label.into(),
            icon: None,
            modified: false,
            closeable: false,
        }
    }
}

/// Horizontal tab bar with optional bottom-edge resize handle.
///
/// - `tabs`: ordered list of tabs.
/// - `active_id`: which tab is selected.
/// - `on_select`: called with the tab id when a tab is clicked.
/// - `on_close`: called with the tab id when the close button is clicked.
/// - `resizable`: if true, renders a bottom resize handle.
/// - `initial_height`: starting height when `resizable` is true.
#[component]
pub fn TabBar(
    tabs: Vec<TabItem>,
    #[props(default)]
    active_id: String,
    #[props(default)]
    on_select: EventHandler<String>,
    #[props(default)]
    on_close: EventHandler<String>,
    #[props(default = false)]
    resizable: bool,
    #[props(default = 40.0)]
    initial_height: f64,
    #[props(default)]
    class: String,
) -> Element {
    let mut height = use_signal(|| initial_height);

    let outer_css = if class.is_empty() {
        "tab-bar".to_string()
    } else {
        format!("tab-bar {class}")
    };

    let inline_style = if resizable {
        format!("height: {}px", *height.read())
    } else {
        String::new()
    };

    rsx! {
        div {
            class: "{outer_css}",
            style: "{inline_style}",
            div {
                class: "tabs",
                for tab in &tabs {
                    {
                        let tab_id = tab.id.clone();
                        let tab_id_close = tab.id.clone();
                        let is_active = tab.id == active_id;
                        let tab_class = if is_active { "tab tab--active" } else { "tab" };

                        rsx! {
                            div {
                                key: "{tab.id}",
                                class: "{tab_class}",
                                onclick: move |_| on_select.call(tab_id.clone()),
                                // Icon slot
                                if let Some(icon_el) = tab.icon.clone() {
                                    span { class: "tab-icon", { icon_el } }
                                }
                                span { class: "tab-label", "{tab.label}" }
                                if tab.modified {
                                    span { class: "tab-modified", title: "Modified" }
                                }
                                if tab.closeable {
                                    button {
                                        class: "tab-close",
                                        title: "Close tab",
                                        aria_label: "Close {tab.label}",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            on_close.call(tab_id_close.clone());
                                        },
                                        CloseIcon { size: 12 }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if resizable {
                ResizeHandle {
                    edge: ResizeEdge::Bottom,
                    direction: ResizeDirection::Vertical,
                    min_size: 32.0,
                    on_resize: move |delta: f64| {
                        let new_h = (*height.read() + delta).max(32.0);
                        height.set(new_h);
                    },
                }
            }
        }
    }
}
