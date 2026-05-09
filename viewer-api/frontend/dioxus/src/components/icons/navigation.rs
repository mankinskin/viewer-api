use dioxus::prelude::*;

#[component]
pub fn ChevronRightIcon(
    #[props(default = 16)] size: u32,
    #[props(default)] class: String,
    #[props(default = "currentColor".to_string())] color: String,
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
            polyline { points: "9 18 15 12 9 6" }
        }
    }
}

#[component]
pub fn ChevronDownIcon(
    #[props(default = 16)] size: u32,
    #[props(default)] class: String,
    #[props(default = "currentColor".to_string())] color: String,
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
            polyline { points: "6 9 12 15 18 9" }
        }
    }
}
