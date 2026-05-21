use dioxus::prelude::*;

#[component]
pub fn ExplorerShell(
    #[props(default)] search: Option<Element>,
    #[props(default)] controls: Option<Element>,
    #[props(default)] status: Option<Element>,
    #[props(default)] body: Option<Element>,
    #[props(default)] class: String,
) -> Element {
    let combined = if class.is_empty() {
        "explorer-shell".to_string()
    } else {
        format!("explorer-shell {class}")
    };

    rsx! {
        div {
            class: "{combined}",
            style: "display: flex; flex-direction: column; height: 100%; overflow: hidden;",

            if let Some(search) = search { {search} }
            if let Some(controls) = controls { {controls} }
            if let Some(status) = status { {status} }
            if let Some(body) = body { {body} }
        }
    }
}

#[component]
pub fn SidebarSearch(
    value: String,
    on_input: EventHandler<String>,
    #[props(default)] placeholder: String,
    #[props(default)] hint: Option<String>,
    #[props(default)] input_testid: Option<String>,
    #[props(default)] hint_testid: Option<String>,
    #[props(default)] on_focus: Option<EventHandler<FocusEvent>>,
    #[props(default)] on_keydown: Option<EventHandler<KeyboardEvent>>,
) -> Element {
    let focus_handler = on_focus.clone();
    let keydown_handler = on_keydown.clone();

    rsx! {
        div {
            class: "sidebar-search",
            input {
                r#type: "text",
                "data-testid": "{input_testid.clone().unwrap_or_default()}",
                value: "{value}",
                placeholder: "{placeholder}",
                oninput: move |event| on_input.call(event.value()),
                onfocus: move |event| {
                    if let Some(handler) = focus_handler.as_ref() {
                        handler.call(event);
                    }
                },
                onkeydown: move |event| {
                    if let Some(handler) = keydown_handler.as_ref() {
                        handler.call(event);
                    }
                }
            }
            if let Some(hint) = hint.as_deref() {
                div {
                    style: "color: var(--text-muted); font-size: 11px; line-height: 1.35;",
                    "data-testid": "{hint_testid.clone().unwrap_or_default()}",
                    "{hint}"
                }
            }
        }
    }
}
