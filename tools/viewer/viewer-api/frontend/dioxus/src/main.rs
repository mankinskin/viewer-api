use dioxus::prelude::*;
use std::rc::Rc;
use viewer_api_dioxus::{
    clear_session, get_hash_param, get_session_id, remove_hash_param, set_hash_param,
    with_session, UrlStateManager,
    store::ThemeProvider,
    FileTree, FilterDef, GlassPanel, Header, Layout, Panel, PanelPlacement, Sidebar, SortKey,
    ThemeSettings, TreeNode, TreeView,
};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        ThemeProvider {
            Demo {}
        }
    }
}

// ── Demo ──────────────────────────────────────────────────────────────────────

#[component]
fn Demo() -> Element {
    let mut sidebar_collapsed = use_signal(|| false);
    let mut show_theme = use_signal(|| false);
    let mut selected_node: Signal<Option<String>> = use_signal(|| None);
    let mut active_filters: Signal<Vec<String>> = use_signal(Vec::new);

    // Sample tree data
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
        SortKey { key: "name".into(), label: "Name".into(), ascending: true },
        SortKey { key: "type".into(), label: "Type".into(), ascending: false },
    ];

    let filters = vec![
        FilterDef { key: "rs".into(), label: ".rs files".into(), count: 7, color: Some("var(--accent-orange)".into()) },
        FilterDef { key: "dirs".into(), label: "Dirs".into(), count: 3, color: Some("var(--accent-yellow)".into()) },
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
                            onclick: move |_| { let v = *show_theme.read(); show_theme.set(!v); },
                            "🎨 Theme"
                        }
                    },
                }
            },

            // ── Sidebar ──
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
                        let mut afl = active_filters.write();
                        if let Some(pos) = afl.iter().position(|k| k == &key) {
                            afl.remove(pos);
                        } else {
                            afl.push(key);
                        }
                    },
                    on_sort: move |_key: String| {},
                    selected_id: selected_node.read().clone(),
                    on_select: move |id: String| selected_node.set(Some(id)),
                    initially_expanded: vec!["src".into(), "components".into()],
                }
            }

            // ── Main content ──
            div {
                class: "content",
                style: "overflow: auto; padding: var(--spacing-md); display: flex; flex-direction: column; gap: 16px;",

                // Glass panel demo
                GlassPanel {
                    title: "GlassPanel",
                    div {
                        style: "color: var(--text-secondary); font-size: 13px;",
                        "A frosted-glass card container with optional title. CSS class: .glass-panel"
                    }
                }

                // Panel placement demo
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

                // TreeView bare demo
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

                // Selected node display
                if let Some(id) = selected_node.read().as_ref() {
                    div {
                        style: "font-size: 12px; color: var(--accent-green); padding: 4px 8px; background: var(--bg-tertiary); border-radius: 4px;",
                        "Selected: {id}"
                    }
                }

                // URL State demo — exercises the new url_state module
                GlassPanel {
                    title: "URL State",
                    UrlStateDemo {}
                }

                // Session demo — exercises the new session module
                GlassPanel {
                    title: "Session",
                    SessionDemo {}
                }
            }

            // ── Bottom panel ──
            Panel {
                placement: PanelPlacement::Bottom,
                initial_size: 80.0,
                div {
                    style: "padding: 8px 16px; font-size: 12px; color: var(--text-muted); display: flex; align-items: center; gap: 8px;",
                    "Bottom Panel (resizable ↑)"
                }
            }
        }

        // ── ThemeSettings modal ──
        if *show_theme.read() {
            div {
                style: "position: fixed; inset: 0; z-index: 9000; display: flex; align-items: center; justify-content: center; background: rgba(0,0,0,.45);",
                onclick: move |_| show_theme.set(false),
                div {
                    style: "max-width: 540px; width: 100%; max-height: 90vh; overflow-y: auto;",
                    // stop click bubbling to the backdrop
                    onclick: move |e| e.stop_propagation(),
                    ThemeSettings {
                        on_close: move |_| show_theme.set(false),
                    }
                }
            }
        }
    }
}

// ── UrlStateDemo ──────────────────────────────────────────────────────────────

/// Interactive demo of the hash-based URL state utilities.
///
/// Rendered inside the main demo app so Playwright tests can drive the public
/// API (`get_hash_param` / `set_hash_param` / `remove_hash_param` /
/// `UrlStateManager`) without any inline JS.
#[component]
fn UrlStateDemo() -> Element {
    let mut get_key = use_signal(String::new);
    let mut get_result = use_signal(|| "\u{2014}".to_owned()); // em dash
    let mut set_key = use_signal(String::new);
    let mut set_value = use_signal(String::new);
    let mut remove_key = use_signal(String::new);

    // Initialised from the current URL hash so the display is accurate on mount.
    let mut current_hash = use_signal(|| {
        web_sys::window()
            .map(|w| w.location().hash().unwrap_or_default())
            .unwrap_or_default()
    });
    let popstate_count = use_signal(|| 0u32);

    // Keep the UrlStateManager alive for the component lifetime.
    // The popstate listener is removed automatically when the manager drops.
    // Wrapped in Rc to satisfy use_hook's Clone requirement — the Rc is never
    // actually cloned; it just gives the type system what it needs.
    let _url_state_manager = use_hook(|| {
        let mut count = popstate_count;
        let mut hash_sig = current_hash;
        Rc::new(UrlStateManager::new(move || {
            // Signal::set requires &mut self, so the closure is FnMut.
            // Read into a local first so the borrow ends before the set call.
            let new_count = *count.read() + 1;
            count.set(new_count);
            let h = web_sys::window()
                .map(|w| w.location().hash().unwrap_or_default())
                .unwrap_or_default();
            hash_sig.set(h);
        }))
    });

    let btn_style = "padding: 4px 10px; border: 1px solid var(--border-primary); background: var(--bg-tertiary); color: var(--text-primary); cursor: pointer; border-radius: 3px; font-size: 12px;";
    let inp_style = "flex: 1; padding: 4px 6px; border: 1px solid var(--border-primary); background: var(--bg-secondary); color: var(--text-primary); border-radius: 3px; font-size: 12px;";
    let row_style = "display: flex; gap: 6px; align-items: center;";

    rsx! {
        div {
            "data-testid": "url-state-demo",
            style: "display: flex; flex-direction: column; gap: 8px;",

            // ── get ──
            div {
                style: "{row_style}",
                input {
                    "data-testid": "hash-get-key",
                    r#type: "text",
                    placeholder: "key",
                    value: "{get_key}",
                    style: "{inp_style}",
                    oninput: move |e| get_key.set(e.value()),
                }
                button {
                    "data-testid": "hash-get-btn",
                    style: "{btn_style}",
                    onclick: move |_| {
                        let v = get_hash_param(&get_key.read());
                        get_result.set(v.unwrap_or_else(|| "\u{2014}".to_owned()));
                    },
                    "get"
                }
                code {
                    "data-testid": "hash-get-result",
                    style: "color: var(--accent-green); font-size: 12px; min-width: 60px;",
                    "{get_result}"
                }
            }

            // ── set ──
            div {
                style: "{row_style}",
                input {
                    "data-testid": "hash-set-key",
                    r#type: "text",
                    placeholder: "key",
                    value: "{set_key}",
                    style: "{inp_style}",
                    oninput: move |e| set_key.set(e.value()),
                }
                input {
                    "data-testid": "hash-set-value",
                    r#type: "text",
                    placeholder: "value",
                    value: "{set_value}",
                    style: "{inp_style}",
                    oninput: move |e| set_value.set(e.value()),
                }
                button {
                    "data-testid": "hash-set-btn",
                    style: "{btn_style}",
                    onclick: move |_| {
                        set_hash_param(&set_key.read(), &set_value.read());
                        let h = web_sys::window()
                            .map(|w| w.location().hash().unwrap_or_default())
                            .unwrap_or_default();
                        current_hash.set(h);
                    },
                    "set"
                }
            }

            // ── remove ──
            div {
                style: "{row_style}",
                input {
                    "data-testid": "hash-remove-key",
                    r#type: "text",
                    placeholder: "key",
                    value: "{remove_key}",
                    style: "{inp_style}",
                    oninput: move |e| remove_key.set(e.value()),
                }
                button {
                    "data-testid": "hash-remove-btn",
                    style: "{btn_style}",
                    onclick: move |_| {
                        remove_hash_param(&remove_key.read());
                        let h = web_sys::window()
                            .map(|w| w.location().hash().unwrap_or_default())
                            .unwrap_or_default();
                        current_hash.set(h);
                    },
                    "remove"
                }
            }

            // ── status ──
            div {
                style: "display: flex; gap: 16px; font-size: 11px; color: var(--text-muted);",
                span {
                    "Hash: "
                    code {
                        "data-testid": "hash-current",
                        style: "color: var(--accent-cyan);",
                        "{current_hash}"
                    }
                }
                span {
                    "Popstate count: "
                    code {
                        "data-testid": "popstate-count",
                        style: "color: var(--accent-yellow);",
                        "{popstate_count}"
                    }
                }
            }
        }
    }
}

// ── SessionDemo ───────────────────────────────────────────────────────────────

/// Interactive demo of session ID utilities.
///
/// Rendered inside the main demo app so Playwright tests can verify
/// `get_session_id` / `clear_session` / `with_session` without inline JS.
#[component]
fn SessionDemo() -> Element {
    let mut session_id = use_signal(get_session_id);

    // Build initial with_session output at mount time.
    let init_headers = with_session(vec![
        ("Content-Type".to_owned(), "application/json".to_owned()),
    ]);
    let mut headers_display = use_signal(move || {
        init_headers
            .into_iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect::<Vec<_>>()
            .join("\n")
    });

    let btn_style = "padding: 4px 10px; border: 1px solid var(--border-primary); background: var(--bg-tertiary); color: var(--text-primary); cursor: pointer; border-radius: 3px; font-size: 12px;";

    rsx! {
        div {
            "data-testid": "session-demo",
            style: "display: flex; flex-direction: column; gap: 8px;",

            // Session ID display
            div {
                style: "display: flex; gap: 8px; align-items: baseline; font-size: 12px;",
                span { style: "color: var(--text-muted);", "Session ID:" }
                code {
                    "data-testid": "session-id",
                    style: "color: var(--accent-blue); font-size: 11px; word-break: break-all;",
                    "{session_id}"
                }
            }

            // Actions
            div {
                style: "display: flex; gap: 6px;",
                button {
                    "data-testid": "session-clear-btn",
                    style: "{btn_style}",
                    onclick: move |_| {
                        clear_session();
                    },
                    "clear"
                }
                button {
                    "data-testid": "session-refresh-btn",
                    style: "{btn_style}",
                    onclick: move |_| {
                        let id = get_session_id();
                        session_id.set(id);
                        let hd = with_session(vec![
                            ("Content-Type".to_owned(), "application/json".to_owned()),
                        ]);
                        headers_display.set(
                            hd.into_iter()
                                .map(|(k, v)| format!("{k}: {v}"))
                                .collect::<Vec<_>>()
                                .join("\n"),
                        );
                    },
                    "refresh"
                }
            }

            // with_session header output
            div {
                style: "font-size: 11px;",
                span { style: "color: var(--text-muted);", "with_session headers:" }
                pre {
                    "data-testid": "with-session-output",
                    style: "margin: 4px 0; background: var(--bg-tertiary); padding: 6px 8px; border-radius: 3px; font-size: 11px; color: var(--text-secondary); overflow-x: auto;",
                    "{headers_display}"
                }
            }
        }
    }
}
