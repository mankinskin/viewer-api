//! SVG icon components ported from viewer-api TypeScript Icons.tsx.
//!
//! All icons accept optional `size`, `class`, and `color` props.
use dioxus::prelude::*;

// ── Document / File icons ─────────────────────────────────────────────────────

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

// ── Navigation icons ──────────────────────────────────────────────────────────

#[component]
pub fn ChevronRightIcon(
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
            polyline { points: "9 18 15 12 9 6" }
        }
    }
}

#[component]
pub fn ChevronDownIcon(
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
            polyline { points: "6 9 12 15 18 9" }
        }
    }
}

// ── Action icons ──────────────────────────────────────────────────────────────

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

// ── Status icons ──────────────────────────────────────────────────────────────

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

// ── Specialized icons ─────────────────────────────────────────────────────────

#[component]
pub fn CrateIcon(
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
            path { d: "M21 16V8a2 2 0 00-1-1.73l-7-4a2 2 0 00-2 0l-7 4A2 2 0 003 8v8a2 2 0 001 1.73l7 4a2 2 0 002 0l7-4A2 2 0 0021 16z" }
            polyline { points: "3.27 6.96 12 12.01 20.73 6.96" }
            line { x1: "12", y1: "22.08", x2: "12", y2: "12" }
        }
    }
}

#[component]
pub fn LogIcon(
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
            path { d: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" }
            polyline { points: "14 2 14 8 20 8" }
            line { x1: "16", y1: "13", x2: "8", y2: "13" }
            line { x1: "16", y1: "17", x2: "8", y2: "17" }
            line { x1: "10", y1: "9", x2: "8", y2: "9" }
        }
    }
}

#[component]
pub fn CodeIcon(
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
            polyline { points: "16 18 22 12 16 6" }
            polyline { points: "8 6 2 12 8 18" }
        }
    }
}

#[component]
pub fn GraphIcon(
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
            circle { cx: "18", cy: "5", r: "3" }
            circle { cx: "6", cy: "12", r: "3" }
            circle { cx: "18", cy: "19", r: "3" }
            line { x1: "8.59", y1: "13.51", x2: "15.42", y2: "17.49" }
            line { x1: "15.41", y1: "6.51", x2: "8.59", y2: "10.49" }
        }
    }
}

#[component]
pub fn StatsIcon(
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
            line { x1: "18", y1: "20", x2: "18", y2: "10" }
            line { x1: "12", y1: "20", x2: "12", y2: "4" }
            line { x1: "6", y1: "20", x2: "6", y2: "14" }
        }
    }
}

#[component]
pub fn HomeIcon(
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
            path { d: "M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" }
            polyline { points: "9 22 9 12 15 12 15 22" }
        }
    }
}

/// Module icon — file with code lines (represents a Rust module).
#[component]
pub fn ModuleIcon(
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
            path { d: "M14.5 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V7.5L14.5 2z" }
            polyline { points: "14 2 14 8 20 8" }
            polyline { points: "10 13 8 15 10 17" }
            polyline { points: "14 13 16 15 14 17" }
        }
    }
}

/// Source file icon — file with bracket symbols (represents a .rs source file).
#[component]
pub fn SourceFileIcon(
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
            path { d: "M14.5 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V7.5L14.5 2z" }
            polyline { points: "14 2 14 8 20 8" }
            path { d: "M9 13l-2 2 2 2" }
            path { d: "M15 13l2 2-2 2" }
        }
    }
}

/// Hamburger icon (three horizontal lines) for mobile sidebar toggle.
#[component]
pub fn HamburgerIcon(
    #[props(default = 20)]
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
            line { x1: "3", y1: "6", x2: "21", y2: "6" }
            line { x1: "3", y1: "12", x2: "21", y2: "12" }
            line { x1: "3", y1: "18", x2: "21", y2: "18" }
        }
    }
}
