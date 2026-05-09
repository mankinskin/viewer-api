use dioxus::prelude::*;

use crate::components::{ResizeDirection, ResizeEdge, ResizeHandle};

#[derive(Clone, PartialEq, Default)]
pub enum PanelPlacement {
    #[default]
    Left,
    Right,
    Top,
    Bottom,
}

impl PanelPlacement {
    fn css_class(&self) -> &'static str {
        match self {
            PanelPlacement::Left => "panel panel-left",
            PanelPlacement::Right => "panel panel-right",
            PanelPlacement::Top => "panel panel-top",
            PanelPlacement::Bottom => "panel panel-bottom",
        }
    }

    fn resize_edge(&self) -> ResizeEdge {
        match self {
            PanelPlacement::Left => ResizeEdge::Right,
            PanelPlacement::Right => ResizeEdge::Left,
            PanelPlacement::Top => ResizeEdge::Bottom,
            PanelPlacement::Bottom => ResizeEdge::Top,
        }
    }

    fn resize_direction(&self) -> ResizeDirection {
        match self {
            PanelPlacement::Left | PanelPlacement::Right => ResizeDirection::Horizontal,
            PanelPlacement::Top | PanelPlacement::Bottom => ResizeDirection::Vertical,
        }
    }

    fn is_horizontal(&self) -> bool {
        matches!(self, PanelPlacement::Left | PanelPlacement::Right)
    }
}

/// A resizable panel anchored to one edge of its container.
#[component]
pub fn Panel(
    children: Element,
    #[props(default)]
    placement: PanelPlacement,
    #[props(default = 300.0)]
    initial_size: f64,
    #[props(default = 80.0)]
    min_size: f64,
    #[props(default = true)]
    resizable: bool,
    #[props(default)]
    class: String,
) -> Element {
    let mut size = use_signal(|| initial_size);
    let resizing = use_signal(|| false);

    let base_class = placement.css_class();
    let is_horizontal = placement.is_horizontal();
    let resize_edge = placement.resize_edge();
    let resize_direction = placement.resize_direction();

    let panel_class = use_memo(move || {
        let result = if *resizing.read() {
            format!("{base_class} panel-resizing")
        } else {
            base_class.to_string()
        };
        if class.is_empty() {
            result
        } else {
            format!("{result} {class}")
        }
    });

    let inline_style = use_memo(move || {
        if is_horizontal {
            format!("width: {}px", *size.read())
        } else {
            format!("height: {}px", *size.read())
        }
    });

    rsx! {
        div {
            class: "{panel_class}",
            style: "{inline_style}",
            {children}
            if resizable {
                ResizeHandle {
                    edge: resize_edge,
                    direction: resize_direction,
                    min_size: min_size,
                    on_resize: move |delta: f64| {
                        let new_size = (*size.read() + delta).max(min_size);
                        size.set(new_size);
                    },
                }
            }
        }
    }
}

/// Frosted-glass card panel — `backdrop-filter: blur` overlay.
#[component]
pub fn GlassPanel(
    #[props(default)]
    title: Option<String>,
    children: Element,
    #[props(default)]
    class: String,
) -> Element {
    let combined = if class.is_empty() {
        "glass-panel".to_string()
    } else {
        format!("glass-panel {class}")
    };

    rsx! {
        div {
            class: "{combined}",
            if let Some(title) = title {
                div {
                    class: "glass-panel__header",
                    span { class: "glass-panel__title", "{title}" }
                }
            }
            div {
                class: "glass-panel__body",
                {children}
            }
        }
    }
}