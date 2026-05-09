use std::collections::BTreeSet;

use dioxus::prelude::*;

use super::types::{FilterDef, SortKey, TreeNode};
use super::view::TreeView;
use crate::components::{FilterIcon, Spinner};

#[component]
pub fn FileTree(
    nodes: Vec<TreeNode>,
    #[props(default)]
    sort_keys: Vec<SortKey>,
    #[props(default)]
    on_sort: EventHandler<String>,
    #[props(default)]
    filters: Vec<FilterDef>,
    #[props(default)]
    active_filters: Vec<String>,
    #[props(default)]
    on_filter: EventHandler<String>,
    #[props(default)]
    selected_id: Option<String>,
    #[props(default)]
    on_select: EventHandler<String>,
    #[props(default)]
    initially_expanded: Vec<String>,
    #[props(default = false)]
    loading: bool,
    #[props(default)]
    class: String,
    #[props(default = false)]
    show_checkboxes: bool,
    #[props(default)]
    on_selection_change: EventHandler<BTreeSet<String>>,
) -> Element {
    let combined = if class.is_empty() {
        "file-tree".to_string()
    } else {
        format!("file-tree {class}")
    };

    rsx! {
        div {
            class: "{combined}",
            if !sort_keys.is_empty() {
                div {
                    class: "file-tree__sort-header",
                    for sort_key in &sort_keys {
                        {
                            let key = sort_key.key.clone();
                            let button_class = if sort_key.ascending {
                                "file-tree__sort-btn file-tree__sort-btn--active"
                            } else {
                                "file-tree__sort-btn"
                            };
                            rsx! {
                                button {
                                    key: "{sort_key.key}",
                                    class: "{button_class}",
                                    onclick: move |_| on_sort.call(key.clone()),
                                    "{sort_key.label}"
                                    if sort_key.ascending { " ↑" } else { " ↓" }
                                }
                            }
                        }
                    }
                }
            }

            if !filters.is_empty() {
                div {
                    class: "file-tree__filter-header",
                    for filter in &filters {
                        {
                            let filter_key = filter.key.clone();
                            let is_active = active_filters.contains(&filter.key);
                            let button_class = if is_active {
                                "file-tree__filter-btn file-tree__filter-btn--active"
                            } else {
                                "file-tree__filter-btn"
                            };
                            let badge_style = filter
                                .color
                                .as_deref()
                                .map(|color| format!("color: {color}"))
                                .unwrap_or_default();
                            rsx! {
                                button {
                                    key: "{filter.key}",
                                    class: "{button_class}",
                                    onclick: move |_| on_filter.call(filter_key.clone()),
                                    span {
                                        class: "file-tree__filter-icon",
                                        FilterIcon { size: 12 }
                                    }
                                    span { class: "file-tree__filter-label", "{filter.label}" }
                                    span {
                                        class: "tree-badge",
                                        style: "{badge_style}",
                                        "{filter.count}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if loading {
                div {
                    class: "file-tree__loading",
                    Spinner {}
                    span { " Loading…" }
                }
            } else if nodes.is_empty() {
                div {
                    class: "file-tree__empty",
                    "No items to display."
                }
            } else {
                TreeView {
                    nodes,
                    selected_id,
                    on_select,
                    initially_expanded,
                    show_checkboxes,
                    on_selection_change,
                }
            }
        }
    }
}