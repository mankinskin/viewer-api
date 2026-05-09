use dioxus::prelude::*;
use std::rc::Rc;
use viewer_api_dioxus::{
    get_hash_param, remove_hash_param, set_hash_param, UrlStateManager,
};

#[component]
pub(super) fn UrlStateDemo() -> Element {
    let mut get_key = use_signal(String::new);
    let mut get_result = use_signal(|| "\u{2014}".to_owned());
    let mut set_key = use_signal(String::new);
    let mut set_value = use_signal(String::new);
    let mut remove_key = use_signal(String::new);

    let mut current_hash = use_signal(|| {
        web_sys::window()
            .map(|window| window.location().hash().unwrap_or_default())
            .unwrap_or_default()
    });
    let popstate_count = use_signal(|| 0u32);

    let _url_state_manager = use_hook(|| {
        let mut count = popstate_count;
        let mut hash_sig = current_hash;
        Rc::new(UrlStateManager::new(move || {
            let new_count = *count.read() + 1;
            count.set(new_count);
            let hash = web_sys::window()
                .map(|window| window.location().hash().unwrap_or_default())
                .unwrap_or_default();
            hash_sig.set(hash);
        }))
    });

    let btn_style = "padding: 4px 10px; border: 1px solid var(--border-primary); background: var(--bg-tertiary); color: var(--text-primary); cursor: pointer; border-radius: 3px; font-size: 12px;";
    let inp_style = "flex: 1; padding: 4px 6px; border: 1px solid var(--border-primary); background: var(--bg-secondary); color: var(--text-primary); border-radius: 3px; font-size: 12px;";
    let row_style = "display: flex; gap: 6px; align-items: center;";

    rsx! {
        div {
            "data-testid": "url-state-demo",
            style: "display: flex; flex-direction: column; gap: 8px;",

            div {
                style: "{row_style}",
                input {
                    "data-testid": "hash-get-key",
                    r#type: "text",
                    placeholder: "key",
                    value: "{get_key}",
                    style: "{inp_style}",
                    oninput: move |event| get_key.set(event.value()),
                }
                button {
                    "data-testid": "hash-get-btn",
                    style: "{btn_style}",
                    onclick: move |_| {
                        let value = get_hash_param(&get_key.read());
                        get_result.set(value.unwrap_or_else(|| "\u{2014}".to_owned()));
                    },
                    "get"
                }
                code {
                    "data-testid": "hash-get-result",
                    style: "color: var(--accent-green); font-size: 12px; min-width: 60px;",
                    "{get_result}"
                }
            }

            div {
                style: "{row_style}",
                input {
                    "data-testid": "hash-set-key",
                    r#type: "text",
                    placeholder: "key",
                    value: "{set_key}",
                    style: "{inp_style}",
                    oninput: move |event| set_key.set(event.value()),
                }
                input {
                    "data-testid": "hash-set-value",
                    r#type: "text",
                    placeholder: "value",
                    value: "{set_value}",
                    style: "{inp_style}",
                    oninput: move |event| set_value.set(event.value()),
                }
                button {
                    "data-testid": "hash-set-btn",
                    style: "{btn_style}",
                    onclick: move |_| {
                        set_hash_param(&set_key.read(), &set_value.read());
                        let hash = web_sys::window()
                            .map(|window| window.location().hash().unwrap_or_default())
                            .unwrap_or_default();
                        current_hash.set(hash);
                    },
                    "set"
                }
            }

            div {
                style: "{row_style}",
                input {
                    "data-testid": "hash-remove-key",
                    r#type: "text",
                    placeholder: "key",
                    value: "{remove_key}",
                    style: "{inp_style}",
                    oninput: move |event| remove_key.set(event.value()),
                }
                button {
                    "data-testid": "hash-remove-btn",
                    style: "{btn_style}",
                    onclick: move |_| {
                        remove_hash_param(&remove_key.read());
                        let hash = web_sys::window()
                            .map(|window| window.location().hash().unwrap_or_default())
                            .unwrap_or_default();
                        current_hash.set(hash);
                    },
                    "remove"
                }
            }

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