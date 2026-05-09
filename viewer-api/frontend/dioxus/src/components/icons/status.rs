use dioxus::prelude::*;

#[component]
pub fn CheckIcon(
    #[props(default = 16)]
    size: u32,
    #[props(default)]
    class: String,
    #[props(default = "currentColor".to_string())]
    color: String,
) -> Element {
    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            class: "{class}",
            polyline { points: "20 6 9 17 4 12" }
        }
    }
}

#[component]
pub fn AlertIcon(
    #[props(default = 16)]
    size: u32,
    #[props(default)]
    class: String,
    #[props(default = "currentColor".to_string())]
    color: String,
) -> Element {
    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            class: "{class}",
            circle { cx: "12", cy: "12", r: "10" }
            line { x1: "12", y1: "8", x2: "12", y2: "12" }
            line { x1: "12", y1: "16", x2: "12.01", y2: "16" }
        }
    }
}

#[component]
pub fn InfoIcon(
    #[props(default = 16)]
    size: u32,
    #[props(default)]
    class: String,
    #[props(default = "currentColor".to_string())]
    color: String,
) -> Element {
    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            class: "{class}",
            circle { cx: "12", cy: "12", r: "10" }
            line { x1: "12", y1: "16", x2: "12", y2: "12" }
            line { x1: "12", y1: "8", x2: "12.01", y2: "8" }
        }
    }
}