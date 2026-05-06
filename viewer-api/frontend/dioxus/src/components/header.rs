//! `HeaderActions` — standard header button row used by viewer apps.
//!
//! All actions are optional `EventHandler<()>`; only buttons whose handler
//! is `Some` are rendered.  Visual style follows the shared `.btn` /
//! `.btn-icon` / `.btn-active` classes from `buttons.css`.
use dioxus::prelude::*;

use crate::components::{AlertIcon, FilterIcon, HomeIcon, RefreshIcon};
use crate::components::icons::InfoIcon;

/// Button row for the viewer header.
///
/// ```ignore
/// HeaderActions {
///     on_home: Some(EventHandler::new(move |_| go_home())),
///     on_refresh: Some(EventHandler::new(move |_| reload())),
///     on_filter_toggle: Some(EventHandler::new(move |_| toggle_filter())),
///     filter_active: filter_open,
///     has_active_filters: filter_count > 0,
///     ..Default::default()
/// }
/// ```
#[component]
pub fn HeaderActions(
    /// Home / root navigation.  Renders a home-icon button.
    #[props(default)]
    on_home: Option<EventHandler<()>>,
    /// Reload current view.  Renders a refresh-icon button.
    #[props(default)]
    on_refresh: Option<EventHandler<()>>,
    /// Toggle filter panel open/closed.  Renders a filter-icon button.
    /// When `filter_active` is `true` the button gets `.btn-active`.
    /// When `has_active_filters` is `true`, a small dot overlay is shown.
    #[props(default)]
    on_filter_toggle: Option<EventHandler<()>>,
    /// Clear current filters.  Renders an alert/clear button (only when
    /// `has_active_filters` is `true`).
    #[props(default)]
    on_clear: Option<EventHandler<()>>,
    /// Open the theme settings popover/sidebar (caller-owned).  Renders an
    /// info-icon button.  Pair with the [`crate::components::ThemeSettings`]
    /// component to actually display the panel.
    #[props(default)]
    on_theme_toggle: Option<EventHandler<()>>,
    /// Whether the filter panel is currently open (controls `.btn-active`).
    #[props(default = false)]
    filter_active: bool,
    /// Whether any filters are currently set (shows a dot indicator).
    #[props(default = false)]
    has_active_filters: bool,
    /// Extra CSS classes appended to the root row.
    #[props(default)]
    class: String,
) -> Element {
    let row_class = if class.is_empty() {
        "header-actions".to_string()
    } else {
        format!("header-actions {class}")
    };

    let filter_btn_class = if filter_active {
        "btn btn-icon btn-active"
    } else {
        "btn btn-icon"
    };

    rsx! {
        div {
            class: "{row_class}",

            if let Some(handler) = on_home {
                button {
                    r#type: "button",
                    class: "btn btn-icon",
                    title: "Home",
                    aria_label: "Home",
                    onclick: move |_| handler.call(()),
                    HomeIcon { size: 16 }
                }
            }

            if let Some(handler) = on_refresh {
                button {
                    r#type: "button",
                    class: "btn btn-icon",
                    title: "Refresh",
                    aria_label: "Refresh",
                    onclick: move |_| handler.call(()),
                    RefreshIcon { size: 16 }
                }
            }

            if let Some(handler) = on_filter_toggle {
                button {
                    r#type: "button",
                    class: "{filter_btn_class}",
                    title: "Toggle filters",
                    aria_label: "Toggle filters",
                    aria_pressed: filter_active,
                    onclick: move |_| handler.call(()),
                    FilterIcon { size: 16 }
                    if has_active_filters {
                        span { class: "header-actions__filter-dot", aria_hidden: "true" }
                    }
                }
            }

            if has_active_filters {
                if let Some(handler) = on_clear {
                    button {
                        r#type: "button",
                        class: "btn btn-icon",
                        title: "Clear filters",
                        aria_label: "Clear filters",
                        onclick: move |_| handler.call(()),
                        AlertIcon { size: 16 }
                    }
                }
            }

            if let Some(handler) = on_theme_toggle {
                button {
                    r#type: "button",
                    class: "btn btn-icon",
                    title: "Theme settings",
                    aria_label: "Theme settings",
                    onclick: move |_| handler.call(()),
                    InfoIcon { size: 16 }
                }
            }
        }
    }
}
