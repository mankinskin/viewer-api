//! ResizeHandle component — drag to resize adjacent panels.
//!
//! Mirrors viewer-api TypeScript `ResizeHandle.tsx` with:
//!  - requestAnimationFrame batching so DOM writes happen once per frame.
//!  - Document-level mousemove / mouseup listeners cleaned up on drop.
//!  - Touch support via touchmove / touchend.
//!  - `Closure::into_js_value()` — never `forget()`.
use dioxus::prelude::*;

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
        use std::{cell::RefCell, rc::Rc};
        use wasm_bindgen::{closure::Closure, JsCast, JsValue};

        type DragState = Rc<RefCell<Option<(JsValue, JsValue)>>>;

        let mouse_state: DragState = use_hook(|| Rc::new(RefCell::new(None)));
        let touch_state: DragState = use_hook(|| Rc::new(RefCell::new(None)));

        let is_horizontal = direction == ResizeDirection::Horizontal;

        // Cleanup on unmount
        {
            let ms = Rc::clone(&mouse_state);
            let ts = Rc::clone(&touch_state);
            use_drop(move || {
                let doc_opt = web_sys::window().and_then(|w| w.document());
                if let Some((mm, mu)) = ms.borrow_mut().take() {
                    if let Some(doc) = &doc_opt {
                        let _ = doc.remove_event_listener_with_callback(
                            "mousemove",
                            mm.unchecked_ref::<js_sys::Function>(),
                        );
                        let _ = doc.remove_event_listener_with_callback(
                            "mouseup",
                            mu.unchecked_ref::<js_sys::Function>(),
                        );
                    }
                    drop((mm, mu));
                }
                if let Some((tm, te)) = ts.borrow_mut().take() {
                    if let Some(doc) = &doc_opt {
                        let _ = doc.remove_event_listener_with_callback(
                            "touchmove",
                            tm.unchecked_ref::<js_sys::Function>(),
                        );
                        let _ = doc.remove_event_listener_with_callback(
                            "touchend",
                            te.unchecked_ref::<js_sys::Function>(),
                        );
                    }
                    drop((tm, te));
                }
                if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                    if let Some(body) = doc.body() {
                        let _ = body.style().remove_property("cursor");
                    }
                }
            });
        }

        let start_mouse = {
            let mouse_state = Rc::clone(&mouse_state);

            move |evt: Event<MouseData>| {
                evt.prevent_default();
                let Some(window) = web_sys::window() else { return };
                let Some(doc) = window.document() else { return };

                let cursor = if is_horizontal { "col-resize" } else { "row-resize" };
                if let Some(body) = doc.body() {
                    let _ = body.style().set_property("cursor", cursor);
                }

                let coords = evt.client_coordinates();
                let initial: f64 = if is_horizontal { coords.x } else { coords.y };
                let start_pos: Rc<RefCell<f64>> = Rc::new(RefCell::new(initial));
                let raf_pending: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));

                // mousemove
                let on_resize_mm = on_resize.clone();
                let start_pos_mm = Rc::clone(&start_pos);
                let raf_pending_mm = Rc::clone(&raf_pending);

                let mm: Closure<dyn FnMut(web_sys::MouseEvent)> =
                    Closure::new(move |e: web_sys::MouseEvent| {
                        let pos = if is_horizontal { e.client_x() as f64 } else { e.client_y() as f64 };
                        if !*raf_pending_mm.borrow() {
                            *raf_pending_mm.borrow_mut() = true;
                            let delta = pos - *start_pos_mm.borrow();
                            *start_pos_mm.borrow_mut() = pos;
                            let on_resize_raf = on_resize_mm.clone();
                            let raf_pend_raf = Rc::clone(&raf_pending_mm);
                            if let Some(win) = web_sys::window() {
                                let cb = Closure::once_into_js(move |_: f64| {
                                    *raf_pend_raf.borrow_mut() = false;
                                    on_resize_raf.call(delta);
                                });
                                let _ = win.request_animation_frame(cb.unchecked_ref::<js_sys::Function>());
                            }
                        }
                    });
                let mm_js = mm.into_js_value();

                // mouseup
                let mouse_state_mu = Rc::clone(&mouse_state);
                let doc_mu = doc.clone();

                let mu: Closure<dyn FnMut(web_sys::MouseEvent)> =
                    Closure::new(move |_: web_sys::MouseEvent| {
                        if let Some((mm_h, mu_h)) = mouse_state_mu.borrow_mut().take() {
                            let _ = doc_mu.remove_event_listener_with_callback(
                                "mousemove", mm_h.unchecked_ref::<js_sys::Function>());
                            let _ = doc_mu.remove_event_listener_with_callback(
                                "mouseup", mu_h.unchecked_ref::<js_sys::Function>());
                            drop((mm_h, mu_h));
                        }
                        if let Some(body) = doc_mu.body() {
                            let _ = body.style().remove_property("cursor");
                        }
                    });
                let mu_js = mu.into_js_value();

                let _ = doc.add_event_listener_with_callback(
                    "mousemove", mm_js.unchecked_ref::<js_sys::Function>());
                let _ = doc.add_event_listener_with_callback(
                    "mouseup", mu_js.unchecked_ref::<js_sys::Function>());
                *mouse_state.borrow_mut() = Some((mm_js, mu_js));
            }
        };

        let start_touch = {
            let touch_state = Rc::clone(&touch_state);

            move |evt: Event<TouchData>| {
                evt.prevent_default();
                let Some(window) = web_sys::window() else { return };
                let Some(doc) = window.document() else { return };

                let initial: f64 = {
                    use dioxus::web::WebEventExt;
                    evt.data()
                        .try_as_web_event()
                        .and_then(|e: web_sys::TouchEvent| {
                            let t = e.touches().get(0)?;
                            Some(if is_horizontal { t.client_x() as f64 } else { t.client_y() as f64 })
                        })
                        .unwrap_or(0.0)
                };

                let start_pos: Rc<RefCell<f64>> = Rc::new(RefCell::new(initial));
                let raf_pending: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));

                // touchmove
                let on_resize_tm = on_resize.clone();
                let start_pos_tm = Rc::clone(&start_pos);
                let raf_pending_tm = Rc::clone(&raf_pending);

                let tm: Closure<dyn FnMut(web_sys::TouchEvent)> =
                    Closure::new(move |e: web_sys::TouchEvent| {
                        let Some(touch) = e.touches().get(0) else { return };
                        let pos = if is_horizontal { touch.client_x() as f64 } else { touch.client_y() as f64 };
                        if !*raf_pending_tm.borrow() {
                            *raf_pending_tm.borrow_mut() = true;
                            let delta = pos - *start_pos_tm.borrow();
                            *start_pos_tm.borrow_mut() = pos;
                            let on_resize_raf = on_resize_tm.clone();
                            let raf_pend_raf = Rc::clone(&raf_pending_tm);
                            if let Some(win) = web_sys::window() {
                                let cb = Closure::once_into_js(move |_: f64| {
                                    *raf_pend_raf.borrow_mut() = false;
                                    on_resize_raf.call(delta);
                                });
                                let _ = win.request_animation_frame(cb.unchecked_ref::<js_sys::Function>());
                            }
                        }
                    });
                let tm_js = tm.into_js_value();

                // touchend
                let touch_state_te = Rc::clone(&touch_state);
                let doc_te = doc.clone();

                let te: Closure<dyn FnMut(web_sys::TouchEvent)> =
                    Closure::new(move |_: web_sys::TouchEvent| {
                        if let Some((tm_h, te_h)) = touch_state_te.borrow_mut().take() {
                            let _ = doc_te.remove_event_listener_with_callback(
                                "touchmove", tm_h.unchecked_ref::<js_sys::Function>());
                            let _ = doc_te.remove_event_listener_with_callback(
                                "touchend", te_h.unchecked_ref::<js_sys::Function>());
                            drop((tm_h, te_h));
                        }
                    });
                let te_js = te.into_js_value();

                let _ = doc.add_event_listener_with_callback(
                    "touchmove", tm_js.unchecked_ref::<js_sys::Function>());
                let _ = doc.add_event_listener_with_callback(
                    "touchend", te_js.unchecked_ref::<js_sys::Function>());
                *touch_state.borrow_mut() = Some((tm_js, te_js));
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
