use dioxus::prelude::*;
use viewer_api_dioxus::{
    FileTree,
    FilterDef,
    GlassPanel,
    Header,
    Layout,
    NodeIcon,
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

fn demo_leaf(
    id: &str,
    label: &str,
    kind: &str,
    summary: &str,
) -> TreeNode {
    let mut node = TreeNode::leaf(id, label);
    let tooltip_label = label.to_string();
    let tooltip_kind = kind.to_string();
    let tooltip_summary = summary.to_string();

    node.icon = NodeIcon::SourceFile;
    node.tooltip = Some(summary.to_string());
    node.with_tooltip_render(move || {
        rsx! {
            div {
                style: "display: flex; flex-direction: column; gap: 4px; max-width: 280px;",
                div {
                    style: "font-weight: 600; color: var(--text-primary);",
                    "{tooltip_label}"
                }
                div { "kind: {tooltip_kind}" }
                div {
                    style: "color: var(--text-muted); font-size: 11px; overflow-wrap: anywhere;",
                    "{tooltip_summary}"
                }
            }
        }
    })
}

fn demo_dir(
    id: &str,
    label: &str,
    summary: &str,
    children: Vec<TreeNode>,
) -> TreeNode {
    let child_count = children.len();
    let mut node = TreeNode::dir(id, label, children);
    let tooltip_label = label.to_string();
    let tooltip_summary = summary.to_string();

    node.icon = NodeIcon::Module;
    node.tooltip = Some(summary.to_string());
    node.with_tooltip_render(move || {
        rsx! {
            div {
                style: "display: flex; flex-direction: column; gap: 4px; max-width: 280px;",
                div {
                    style: "font-weight: 600; color: var(--text-primary);",
                    "{tooltip_label}"
                }
                div { "children: {child_count}" }
                div {
                    style: "color: var(--text-muted); font-size: 11px; overflow-wrap: anywhere;",
                    "{tooltip_summary}"
                }
            }
        }
    })
}

#[component]
pub(super) fn Demo() -> Element {
    let mut sidebar_collapsed = use_signal(|| false);
    let mut show_theme = use_signal(|| false);
    let mut selected_node: Signal<Option<String>> = use_signal(|| None);
    let mut active_filters: Signal<Vec<String>> = use_signal(Vec::new);

    let nodes = vec![
        demo_dir(
            "src",
            "src",
            "Source tree for shared viewer primitives.",
            vec![
                demo_dir(
                    "components",
                    "components",
                    "Reusable shell, tree, and theme components shared across viewers.",
                    vec![
                        demo_leaf(
                            "layout",
                            "layout.rs",
                            "component",
                            "Hosts the shared page shell with header, sidebar, and content regions.",
                        ),
                        demo_leaf(
                            "tree_view",
                            "tree_view.rs",
                            "component",
                            "Renders the hierarchical tree rows used by explorer sidebars.",
                        ),
                        demo_leaf(
                            "theme_settings",
                            "theme_settings.rs",
                            "component",
                            "Controls theme tokens, palettes, and appearance settings.",
                        ),
                        demo_leaf(
                            "resize_handle",
                            "resize_handle.rs",
                            "component",
                            "Provides draggable panel resizing for shared layouts.",
                        ),
                    ],
                ),
                demo_dir(
                    "store",
                    "store",
                    "Shared client-side stores and URL/session state helpers.",
                    vec![demo_leaf(
                        "theme_rs",
                        "theme.rs",
                        "store",
                        "Persists theme selection and exposes tokens to Dioxus components.",
                    )],
                ),
                demo_leaf(
                    "main_rs",
                    "main.rs",
                    "entrypoint",
                    "Starts the demo app and wires shared examples into the shell.",
                ),
                demo_leaf(
                    "lib_rs",
                    "lib.rs",
                    "library",
                    "Exports the shared viewer-api Dioxus component surface.",
                ),
            ],
        ),
        demo_dir(
            "public",
            "public",
            "Static assets and CSS used by the shared demo viewer.",
            vec![demo_leaf(
                "css",
                "viewer-api.css",
                "asset",
                "Defines the shared theme tokens and core viewer presentation styles.",
            )],
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
