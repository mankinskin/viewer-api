use dioxus::prelude::*;

#[component]
pub fn DocumentIcon(
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
            class: "{class}",
            path { d: "M14.5 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V7.5L14.5 2z" }
            polyline { points: "14 2 14 8 20 8" }
            line { x1: "16", y1: "13", x2: "8", y2: "13" }
            line { x1: "16", y1: "17", x2: "8", y2: "17" }
            line { x1: "10", y1: "9", x2: "8", y2: "9" }
        }
    }
}

#[component]
pub fn FileIcon(
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
            class: "{class}",
            path { d: "M14.5 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V7.5L14.5 2z" }
            polyline { points: "14 2 14 8 20 8" }
        }
    }
}

#[component]
pub fn FolderIcon(
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
            fill: "{color}",
            class: "{class}",
            path { d: "M10 4H4a2 2 0 00-2 2v12a2 2 0 002 2h16a2 2 0 002-2V8a2 2 0 00-2-2h-8l-2-2z" }
        }
    }
}

#[component]
pub fn FolderOpenIcon(
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
            class: "{class}",
            path { d: "M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z" }
        }
    }
}