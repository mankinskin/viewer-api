use dioxus::prelude::*;

#[component]
pub fn SearchIcon(
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
            circle { cx: "11", cy: "11", r: "8" }
            line { x1: "21", y1: "21", x2: "16.65", y2: "16.65" }
        }
    }
}

#[component]
pub fn FilterIcon(
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
            polygon { points: "22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" }
        }
    }
}

#[component]
pub fn RefreshIcon(
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
            polyline { points: "23 4 23 10 17 10" }
            polyline { points: "1 20 1 14 7 14" }
            path { d: "M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" }
        }
    }
}

#[component]
pub fn CloseIcon(
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
            line { x1: "18", y1: "6", x2: "6", y2: "18" }
            line { x1: "6", y1: "6", x2: "18", y2: "18" }
        }
    }
}

#[component]
pub fn PlusIcon(
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
            line { x1: "12", y1: "5", x2: "12", y2: "19" }
            line { x1: "5", y1: "12", x2: "19", y2: "12" }
        }
    }
}

#[component]
pub fn MinusIcon(
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
            line { x1: "5", y1: "12", x2: "19", y2: "12" }
        }
    }
}