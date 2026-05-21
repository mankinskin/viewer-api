use dioxus::prelude::*;

#[component]
pub fn FilterToggleButton(
    onclick: EventHandler<MouseEvent>,
    #[props(default = false)] active: bool,
    #[props(default)] class: String,
    #[props(default)] active_class: String,
    #[props(default)] inactive_class: String,
    #[props(default)] test_id: String,
    #[props(default)] aria_label: String,
    #[props(default)] title: String,
    children: Element,
) -> Element {
    let mut class_names = Vec::new();
    if !class.is_empty() {
        class_names.push(class);
    }

    let state_class = if active { active_class } else { inactive_class };
    if !state_class.is_empty() {
        class_names.push(state_class);
    }

    let class_name = class_names.join(" ");

    rsx! {
        button {
            class: "{class_name}",
            "data-testid": "{test_id}",
            aria_pressed: if active { "true" } else { "false" },
            aria_label: "{aria_label}",
            title: "{title}",
            onclick: move |event| onclick.call(event),
            {children}
        }
    }
}
