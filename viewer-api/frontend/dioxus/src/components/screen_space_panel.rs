use dioxus::prelude::*;

/// Wraps screen-space overlay panels so pointer and wheel input do not reach
/// graph interaction handlers behind them.
#[component]
pub fn ScreenSpacePanel(
    #[props(default)] class: String,
    #[props(default)] style: String,
    children: Element,
) -> Element {
    rsx! {
        div {
            class: "{class}",
            style: "{style}",
            "data-graph-passthrough": "false",
            onclick: move |event| event.stop_propagation(),
            ondoubleclick: move |event| event.stop_propagation(),
            onmousedown: move |event| event.stop_propagation(),
            onmousemove: move |event| event.stop_propagation(),
            onmouseup: move |event| event.stop_propagation(),
            onwheel: move |event| event.stop_propagation(),
            ontouchstart: move |event| event.stop_propagation(),
            ontouchmove: move |event| event.stop_propagation(),
            ontouchend: move |event| event.stop_propagation(),
            {children}
        }
    }
}