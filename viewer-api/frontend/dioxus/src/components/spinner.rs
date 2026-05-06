//! Spinner component — CSS-animated loading indicator.
use dioxus::prelude::*;

/// Controls the rendered size of a [`Spinner`].
#[derive(Clone, PartialEq)]
pub enum SpinnerSize {
    Sm,
    Md,
    Lg,
}

impl SpinnerSize {
    fn px(&self) -> u32 {
        match self {
            SpinnerSize::Sm => 16,
            SpinnerSize::Md => 24,
            SpinnerSize::Lg => 32,
        }
    }
}

impl Default for SpinnerSize {
    fn default() -> Self {
        SpinnerSize::Md
    }
}

/// Animated loading spinner.
///
/// Animation is defined in `viewer-api.css` via the `va-spin` keyframe.
#[component]
pub fn Spinner(
    #[props(default)]
    size: SpinnerSize,
    #[props(default)]
    class: String,
) -> Element {
    let px = size.px();
    let combined = if class.is_empty() {
        "spinner".to_string()
    } else {
        format!("spinner {class}")
    };
    rsx! {
        svg {
            class: "{combined}",
            width: "{px}",
            height: "{px}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle {
                cx: "12",
                cy: "12",
                r: "10",
                stroke: "var(--border-primary)",
            }
            path {
                d: "M12 2a10 10 0 0 1 10 10",
                stroke: "var(--accent-blue)",
            }
        }
    }
}
