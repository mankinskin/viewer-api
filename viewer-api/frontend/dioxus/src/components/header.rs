//! `HeaderActions` — standard header button row used by viewer apps.
//!
//! All actions are optional `EventHandler<()>`; only buttons whose handler
//! is `Some` are rendered.  Visual style follows the shared `.btn` /
//! `.btn-icon` / `.btn-active` classes from `buttons.css`.
use dioxus::prelude::*;

use crate::components::{
    icons::InfoIcon,
    layout::Header,
    AlertIcon,
    FilterIcon,
    HomeIcon,
    RefreshIcon,
};

/// Shared page-level header shell for Dioxus viewers.
///
/// Builds on top of [`Header`] and [`HeaderActions`] so routes can keep
/// their own left/right extras without re-implementing the shared action row.
#[component]
pub fn PageHeader(
    #[props(default)] lead: Option<Element>,
    #[props(default)] icon: Option<Element>,
    #[props(default)] title: Option<String>,
    #[props(default)] subtitle: Option<String>,
    #[props(default)] left_extra: Option<Element>,
    #[props(default)] middle: Option<Element>,
    #[props(default)] right_prefix: Option<Element>,
    #[props(default)] right_suffix: Option<Element>,
    #[props(default)] on_home: Option<EventHandler<()>>,
    #[props(default)] on_refresh: Option<EventHandler<()>>,
    #[props(default)] on_filter_toggle: Option<EventHandler<()>>,
    #[props(default)] on_clear: Option<EventHandler<()>>,
    #[props(default)] on_theme_toggle: Option<EventHandler<()>>,
    #[props(default = false)] filter_active: bool,
    #[props(default = false)] has_active_filters: bool,
    #[props(default)] class: String,
    #[props(default)] actions_class: String,
) -> Element {
    let has_shared_actions = on_home.is_some()
        || on_refresh.is_some()
        || on_filter_toggle.is_some()
        || (has_active_filters && on_clear.is_some())
        || on_theme_toggle.is_some();

    let left = rsx! {
        if let Some(lead) = lead { {lead} }
        if let Some(icon) = icon {
            span {
                class: "header-icon",
                {icon}
            }
        }
        if let Some(title) = title.as_deref() {
            span {
                class: "header-title",
                "{title}"
            }
        }
        if let Some(subtitle) = subtitle.as_deref() {
            span {
                class: "header-subtitle",
                "{subtitle}"
            }
        }
        if let Some(left_extra) = left_extra { {left_extra} }
    };

    let right = rsx! {
        if let Some(right_prefix) = right_prefix { {right_prefix} }
        if has_shared_actions {
            HeaderActions {
                on_home,
                on_refresh,
                on_filter_toggle,
                on_clear,
                on_theme_toggle,
                filter_active,
                has_active_filters,
                class: actions_class,
            }
        }
        if let Some(right_suffix) = right_suffix { {right_suffix} }
    };

    rsx! {
        Header {
            class,
            left,
            middle,
            right,
        }
    }
}

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
