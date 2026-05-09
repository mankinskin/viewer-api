use dioxus::prelude::*;
use viewer_api_dioxus::{clear_session, get_session_id, with_session};

#[component]
pub(super) fn SessionDemo() -> Element {
    let mut session_id = use_signal(get_session_id);
    let init_headers = with_session(vec![
        ("Content-Type".to_owned(), "application/json".to_owned()),
    ]);
    let mut headers_display = use_signal(move || {
        init_headers
            .into_iter()
            .map(|(key, value)| format!("{key}: {value}"))
            .collect::<Vec<_>>()
            .join("\n")
    });

    let btn_style = "padding: 4px 10px; border: 1px solid var(--border-primary); background: var(--bg-tertiary); color: var(--text-primary); cursor: pointer; border-radius: 3px; font-size: 12px;";

    rsx! {
        div {
            "data-testid": "session-demo",
            style: "display: flex; flex-direction: column; gap: 8px;",

            div {
                style: "display: flex; gap: 8px; align-items: baseline; font-size: 12px;",
                span { style: "color: var(--text-muted);", "Session ID:" }
                code {
                    "data-testid": "session-id",
                    style: "color: var(--accent-blue); font-size: 11px; word-break: break-all;",
                    "{session_id}"
                }
            }

            div {
                style: "display: flex; gap: 6px;",
                button {
                    "data-testid": "session-clear-btn",
                    style: "{btn_style}",
                    onclick: move |_| clear_session(),
                    "clear"
                }
                button {
                    "data-testid": "session-refresh-btn",
                    style: "{btn_style}",
                    onclick: move |_| {
                        let id = get_session_id();
                        session_id.set(id);
                        let headers = with_session(vec![
                            ("Content-Type".to_owned(), "application/json".to_owned()),
                        ]);
                        headers_display.set(
                            headers
                                .into_iter()
                                .map(|(key, value)| format!("{key}: {value}"))
                                .collect::<Vec<_>>()
                                .join("\n"),
                        );
                    },
                    "refresh"
                }
            }

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