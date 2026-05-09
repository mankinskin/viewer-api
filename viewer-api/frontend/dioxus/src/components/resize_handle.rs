//! ResizeHandle component — drag to resize adjacent panels.
//!
//! Mirrors viewer-api TypeScript `ResizeHandle.tsx` with:
//!  - requestAnimationFrame batching so DOM writes happen once per frame.
//!  - Document-level mousemove / mouseup listeners cleaned up on drop.
//!  - Touch support via touchmove / touchend.
//!  - `Closure::into_js_value()` — never `forget()`.
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
mod wasm;

/// Which axis the handle resizes.
#[derive(Clone, Copy, PartialEq)]
pub enum ResizeDirection {
    Horizontal,
    Vertical,
}

impl Default for ResizeDirection {
    fn default() -> Self {
        ResizeDirection::Horizontal
    }
}

/// Which edge of the adjacent panel the handle is attached to.
#[derive(Clone, Copy, PartialEq)]
pub enum ResizeEdge {
    Left,
    Right,
    Top,
    Bottom,
}

impl ResizeEdge {
    fn css_class(&self) -> &'static str {
        match self {
            ResizeEdge::Left => "resize-handle resize-handle-left",
            ResizeEdge::Right => "resize-handle resize-handle-right",
            ResizeEdge::Top => "resize-handle resize-handle-top",
            ResizeEdge::Bottom => "resize-handle resize-handle-bottom",
        }
    }
}

impl Default for ResizeEdge {
    fn default() -> Self {
        ResizeEdge::Right
    }
}

/// A drag-handle that invokes `on_resize` with the delta in pixels.
#[component]
pub fn ResizeHandle(
    #[props(default)]
    edge: ResizeEdge,
    #[props(default)]
    direction: ResizeDirection,
    #[props(default = 100.0)]
    min_size: f64,
    #[props(default = 0.0)]
    max_size: f64,
    on_resize: EventHandler<f64>,
    #[props(default)]
    class: String,
) -> Element {
    #[cfg(target_arch = "wasm32")]
    {
        use self::wasm::{
            cleanup_drag_states, new_drag_state, start_mouse_drag, start_touch_drag, DragState,
        };

        let mouse_state: DragState = use_hook(new_drag_state);
        let touch_state: DragState = use_hook(new_drag_state);

        let is_horizontal = direction == ResizeDirection::Horizontal;

        {
            let mouse_state = mouse_state.clone();
            let touch_state = touch_state.clone();
            use_drop(move || cleanup_drag_states(&mouse_state, &touch_state));
        }

        let start_mouse = {
            let mouse_state = mouse_state.clone();
            let on_resize = on_resize.clone();
            move |evt: Event<MouseData>| {
                start_mouse_drag(mouse_state.clone(), on_resize.clone(), is_horizontal, evt);
            }
        };

        let start_touch = {
            let touch_state = touch_state.clone();
            let on_resize = on_resize.clone();
            move |evt: Event<TouchData>| {
                start_touch_drag(touch_state.clone(), on_resize.clone(), is_horizontal, evt);
            }
        };

        let css = if class.is_empty() {
            edge.css_class().to_string()
        } else {
            format!("{} {class}", edge.css_class())
        };
        let cursor_style = if is_horizontal { "col-resize" } else { "row-resize" };

        rsx! {
            div {
                class: "{css}",
                style: "cursor: {cursor_style}",
                onmousedown: start_mouse,
                ontouchstart: start_touch,
                role: "separator",
                aria_label: "Resize panel",
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (min_size, max_size, on_resize, direction);
        let css = if class.is_empty() {
            edge.css_class().to_string()
        } else {
            format!("{} {class}", edge.css_class())
        };
        rsx! {
            div { class: "{css}", role: "separator" }
        }
    }
}

// ── PanelResizer ──────────────────────────────────────────────────────────────

/// Flow-positioned drag divider placed **between** adjacent flex items.
///
/// Unlike [`ResizeHandle`] (positioned absolutely inside a panel at its edge),
/// `PanelResizer` occupies a thin in-flow slice of the flex row/column and
/// stretches to fill the cross-axis.  Dragging it calls `on_resize(delta)`:
/// - Horizontal (default): positive delta = dragged right, negative = left.
/// - Vertical: positive delta = dragged down, negative = up.
///
/// Apply the delta to the adjacent panel whose size you control:
/// - Left panel:   `width  += delta`
/// - Right panel:  `width  -= delta`
/// - Top panel:    `height += delta`
/// - Bottom panel: `height -= delta`
///
/// CSS class `.panel-resizer` (defined in `layout.css`) overrides the
/// absolute positioning from `.resize-handle` so the element sits in-flow.
#[component]
pub fn PanelResizer(
    /// Resize axis — `Horizontal` (default) for side-by-side panels,
    /// `Vertical` for stacked panels.
    #[props(default)]
    direction: ResizeDirection,
    /// Called with the pixel delta on each animation frame during a drag.
    on_resize: EventHandler<f64>,
    /// Extra CSS classes appended to the element.
    #[props(default)]
    class: String,
) -> Element {
    let edge = match direction {
        ResizeDirection::Horizontal => ResizeEdge::Right,
        ResizeDirection::Vertical  => ResizeEdge::Bottom,
    };
    let extra_class = if class.is_empty() {
        "panel-resizer".to_string()
    } else {
        format!("panel-resizer {class}")
    };
    rsx! {
        ResizeHandle {
            edge: edge,
            direction: direction,
            on_resize: on_resize,
            class: extra_class,
        }
    }
}
