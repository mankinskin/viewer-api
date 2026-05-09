use dioxus::prelude::*;

/// Slim top bar matching `.header` / `.header-left` / `.header-right` CSS.
///
/// Slot props default to `None` so callers only supply what they need.
#[component]
pub fn Header(
    #[props(default)]
    left: Option<Element>,
    #[props(default)]
    middle: Option<Element>,
    #[props(default)]
    right: Option<Element>,
    #[props(default)]
    class: String,
) -> Element {
    let combined = if class.is_empty() {
        "header".to_string()
    } else {
        format!("header {class}")
    };

    rsx! {
        header {
            class: "{combined}",
            div {
                class: "header-left",
                if let Some(left) = left { {left} }
            }
            if let Some(middle) = middle {
                div {
                    class: "header-middle",
                    {middle}
                }
            }
            div {
                class: "header-right",
                if let Some(right) = right { {right} }
            }
        }
    }
}

/// Full-page shell: `.app` column wrapping an optional [`Header`] and the
/// `.main-layout` flex row that holds sidebar + content children.
#[component]
pub fn Layout(
    #[props(default)]
    header: Option<Element>,
    children: Element,
    #[props(default)]
    class: String,
) -> Element {
    let combined = if class.is_empty() {
        "app".to_string()
    } else {
        format!("app {class}")
    };

    rsx! {
        div {
            class: "{combined}",
            if let Some(header) = header { {header} }
            div {
                class: "main-layout",
                {children}
            }
        }
    }
}