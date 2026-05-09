use dioxus::prelude::*;
use viewer_api_dioxus::{
    FileTree,
    FilterDef,
    GlassPanel,
    Header,
    Layout,
    Panel,
    PanelPlacement,
    Sidebar,
    SortKey,
    ThemeSettings,
    TreeNode,
    TreeView,
};

use crate::{
    session_demo::SessionDemo,
    url_state_demo::UrlStateDemo,
};

#[component]
pub(super) fn Demo() -> Element {
    let mut sidebar_collapsed = use_signal(|| false);
    let mut show_theme = use_signal(|| false);
    let mut selected_node: Signal<Option<String>> = use_signal(|| None);
    let mut active_filters: Signal<Vec<String>> = use_signal(Vec::new);

    let nodes = vec![
        TreeNode::dir(
            "src",
            "src",
            vec![
                TreeNode::dir(
                    "components",
                    "components",
                    vec![
                        TreeNode::leaf("layout", "layout.rs"),
                        TreeNode::leaf("tree_view", "tree_view.rs"),
                        TreeNode::leaf("theme_settings", "theme_settings.rs"),
                        TreeNode::leaf("resize_handle", "resize_handle.rs"),
                    ],
                ),
                TreeNode::dir(
                    "store",
                    "store",
                    vec![TreeNode::leaf("theme_rs", "theme.rs")],
                ),
                TreeNode::leaf("main_rs", "main.rs"),
                TreeNode::leaf("lib_rs", "lib.rs"),
            ],
        ),
        TreeNode::dir(
            "public",
            "public",
            vec![TreeNode::leaf("css", "viewer-api.css")],
        ),
    ];

    let sort_keys = vec![
        SortKey {
            key: "name".into(),
            label: "Name".into(),
            ascending: true,
        },
        SortKey {
            key: "type".into(),
            label: "Type".into(),
            ascending: false,
        },
    ];

    let filters = vec![
        FilterDef {
            key: "rs".into(),
            label: ".rs files".into(),
            count: 7,
            color: Some("var(--accent-orange)".into()),
        },
        FilterDef {
            key: "dirs".into(),
            label: "Dirs".into(),
            count: 3,
            color: Some("var(--accent-yellow)".into()),
        },
    ];

    rsx! {
        Layout {
            header: rsx! {
                Header {
                    left: rsx! {
                        span { class: "header-icon", "◈" }
                        span { class: "header-title", "viewer-api-dioxus" }
                        span { class: "header-subtitle", "Component Demo" }
                    },
                    right: rsx! {
                        button {
                            style: "padding: 4px 12px; border-radius: 4px; border: 1px solid var(--border-primary); background: var(--bg-tertiary); color: var(--text-primary); cursor: pointer;",
                            onclick: move |_| {
                                let visible = *show_theme.read();
                                show_theme.set(!visible);
                            },
                            "🎨 Theme"
                        }
                    },
                }
            },

            Sidebar {
                title: "Files",
                badge: "10",
                collapsed: *sidebar_collapsed.read(),
                on_toggle: move |_| sidebar_collapsed.toggle(),

                FileTree {
                    nodes: nodes.clone(),
                    sort_keys: sort_keys.clone(),
                    filters: filters.clone(),
                    active_filters: active_filters.read().clone(),
                    on_filter: move |key: String| {
                        let mut filters = active_filters.write();
                        if let Some(position) = filters.iter().position(|entry| entry == &key) {
                            filters.remove(position);
                        } else {
                            filters.push(key);
                        }
                    },
                    on_sort: move |_key: String| {},
                    selected_id: selected_node.read().clone(),
                    on_select: move |id: String| selected_node.set(Some(id)),
                    initially_expanded: vec!["src".into(), "components".into()],
                }
            }

            div {
                class: "content",
                style: "overflow: auto; padding: var(--spacing-md); display: flex; flex-direction: column; gap: 16px;",

                GlassPanel {
                    title: "GlassPanel",
                    div {
                        style: "color: var(--text-secondary); font-size: 13px;",
                        "A frosted-glass card container with optional title. CSS class: .glass-panel"
                    }
                }

                div {
                    style: "display: flex; gap: 8px; height: 120px; position: relative;",
                    Panel {
                        placement: PanelPlacement::Left,
                        initial_size: 180.0,
                        div {
                            style: "padding: 8px; font-size: 12px; color: var(--text-muted);",
                            "Panel — Left (resizable →)"
                        }
                    }
                    div {
                        style: "flex:1; background: var(--bg-tertiary); border-radius: 4px; display:flex; align-items:center; justify-content:center; font-size:12px; color: var(--text-muted);",
                        "Main content area"
                    }
                    Panel {
                        placement: PanelPlacement::Right,
                        initial_size: 140.0,
                        div {
                            style: "padding: 8px; font-size: 12px; color: var(--text-muted);",
                            "(← resizable) Right Panel"
                        }
                    }
                }

                GlassPanel {
                    title: "TreeView (bare — no FileTree wrapper)",
                    div {
                        style: "max-height: 200px; overflow-y: auto;",
                        TreeView {
                            nodes: nodes.clone(),
                            initially_expanded: vec!["src".into(), "store".into()],
                            on_select: move |id: String| selected_node.set(Some(id)),
                        }
                    }
                }

                if let Some(id) = selected_node.read().as_ref() {
                    div {
                        style: "font-size: 12px; color: var(--accent-green); padding: 4px 8px; background: var(--bg-tertiary); border-radius: 4px;",
                        "Selected: {id}"
                    }
                }

                GlassPanel {
                    title: "URL State",
                    UrlStateDemo {}
                }

                GlassPanel {
                    title: "Session",
                    SessionDemo {}
                }
            }

            Panel {
                placement: PanelPlacement::Bottom,
                initial_size: 80.0,
                div {
                    style: "padding: 8px 16px; font-size: 12px; color: var(--text-muted); display: flex; align-items: center; gap: 8px;",
                    "Bottom Panel (resizable ↑)"
                }
            }
        }

        if *show_theme.read() {
            div {
                style: "position: fixed; inset: 0; z-index: 9000; display: flex; align-items: center; justify-content: center; background: rgba(0,0,0,.45);",
                onclick: move |_| show_theme.set(false),
                div {
                    style: "max-width: 540px; width: 100%; max-height: 90vh; overflow-y: auto;",
                    onclick: move |event| event.stop_propagation(),
                    ThemeSettings {
                        on_close: move |_| show_theme.set(false),
                    }
                }
            }
        }
    }
}
