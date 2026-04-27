//! Pointer interaction: orbit, pan, zoom, **and node drag** for the 3-D
//! graph.
//!
//! Drag works by projecting the cursor's screen-space delta onto a plane
//! perpendicular to the camera fwd at the picked node's depth — matches
//! the TS reference (`useMouseInteraction.ts`). No matrix inversion needed:
//! we scale Δpx by `(2·depth·tan(fov/2)) / canvas_height` to convert
//! pixels → world units along the camera's right/up basis.

#![cfg(target_arch = "wasm32")]

use std::cell::RefCell;
use std::rc::Rc;

use gloo_events::EventListener;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};

use super::camera::{MouseState, CAMERA_FOV};
use super::render::RenderState;

/// Pixels of cursor travel before a click on a card is treated as a drag.
/// Matches the TS reference (`DRAG_THRESHOLD = 5`).
const DRAG_THRESHOLD_PX: f64 = 5.0;

/// Drag candidate / active state, separate from orbit/pan.
#[derive(Default)]
struct DragState {
    /// Card index recorded on `mousedown`; `None` ⇒ no candidate.
    candidate_idx: Option<usize>,
    /// Cursor position when `mousedown` happened (used for threshold check).
    start_x: f64,
    start_y: f64,
    /// `true` once the cursor has moved past `DRAG_THRESHOLD_PX`.
    active: bool,
    /// World-space anchor (initial node position).
    anchor: [f32; 3],
    /// Camera basis snapshot at drag start (so the drag plane stays fixed).
    cam_right: [f32; 3],
    cam_up:    [f32; 3],
    /// Pixels-per-world-unit at the node's depth.
    px_per_world: f32,
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalise(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-6 { [0.0, 0.0, 1.0] } else { [v[0]/len, v[1]/len, v[2]/len] }
}

/// Install mouse listeners on the graph container + document, returning the
/// listener handles. Drop them to detach (Dioxus stores them in a Signal).
pub(crate) fn install(
    container_id: &str,
    state_rc: Rc<RefCell<RenderState>>,
) -> Vec<EventListener> {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return Vec::new();
    };
    let container = doc.get_element_by_id(container_id);
    let container_target: &web_sys::EventTarget = match &container {
        Some(el) => el.as_ref(),
        None     => doc.as_ref(),
    };

    let ms = Rc::new(RefCell::new(MouseState::default()));
    let drag = Rc::new(RefCell::new(DragState::default()));

    let md = EventListener::new(container_target, "mousedown", {
        let ms = ms.clone();
        let drag = drag.clone();
        let st = state_rc.clone();
        move |evt| {
            let Some(e) = evt.dyn_ref::<web_sys::MouseEvent>() else { return };
            let cx = e.client_x() as f64;
            let cy = e.client_y() as f64;

            // If the click landed on a node card, record a drag candidate
            // (don't immediately start orbiting — wait for cursor movement
            // past the threshold to decide between drag vs click).
            let mut card_idx: Option<usize> = None;
            if let Some(target) = e.target() {
                if let Ok(el) = target.dyn_into::<web_sys::Element>() {
                    if let Ok(Some(card)) = el.closest("[data-node-idx]") {
                        if let Some(s) = card.get_attribute("data-node-idx") {
                            card_idx = s.parse::<usize>().ok();
                        }
                    }
                }
            }

            if let Some(idx) = card_idx {
                if e.button() == 0 {
                    let mut d = drag.borrow_mut();
                    d.candidate_idx = Some(idx);
                    d.start_x = cx;
                    d.start_y = cy;
                    d.active  = false;
                    // Snapshot anchor + camera basis for the duration of
                    // this drag so the projection plane stays stable.
                    if let Ok(s) = st.try_borrow() {
                        if let Some(node) = s.layout.nodes.get(idx) {
                            d.anchor = [node.x, node.y, node.z];
                            let eye = s.camera.eye();
                            let target = s.camera.target;
                            let fwd = normalise([
                                target[0] - eye[0],
                                target[1] - eye[1],
                                target[2] - eye[2],
                            ]);
                            let right = normalise(cross(fwd, [0.0, 1.0, 0.0]));
                            let up    = normalise(cross(right, fwd));
                            d.cam_right = right;
                            d.cam_up    = up;
                            // depth = projection of (anchor - eye) onto fwd
                            let to_node = [
                                d.anchor[0] - eye[0],
                                d.anchor[1] - eye[1],
                                d.anchor[2] - eye[2],
                            ];
                            let depth = (to_node[0] * fwd[0]
                                + to_node[1] * fwd[1]
                                + to_node[2] * fwd[2]).abs().max(0.1);
                            // Canvas height in CSS pixels (mirrors render).
                            let canvas_h = s.gpu.canvas_h.max(1) as f32;
                            // World units per pixel at this depth (vertical).
                            let world_per_px =
                                2.0 * depth * (CAMERA_FOV * 0.5).tan() / canvas_h;
                            d.px_per_world = if world_per_px > 1e-6 {
                                1.0 / world_per_px
                            } else {
                                1.0
                            };
                        }
                    }
                    return;
                }
            }

            // Plain background click: orbit / pan as before.
            let mut m = ms.borrow_mut();
            m.last_x = cx;
            m.last_y = cy;
            let button = e.button();
            if button == 2 || (button == 0 && e.shift_key()) {
                m.panning = true;
            } else if button == 0 {
                m.orbiting = true;
            }
        }
    });

    let mm = EventListener::new(&doc, "mousemove", {
        let ms = ms.clone();
        let drag = drag.clone();
        let st = state_rc.clone();
        move |evt| {
            let Some(e) = evt.dyn_ref::<web_sys::MouseEvent>() else { return };
            let cx = e.client_x() as f64;
            let cy = e.client_y() as f64;

            // ── Active drag wins over orbit/pan ──
            let drag_snapshot = drag.borrow();
            if let Some(idx) = drag_snapshot.candidate_idx {
                let dx_total = cx - drag_snapshot.start_x;
                let dy_total = cy - drag_snapshot.start_y;
                let dist = (dx_total * dx_total + dy_total * dy_total).sqrt();
                let already_active = drag_snapshot.active;
                let anchor    = drag_snapshot.anchor;
                let cam_right = drag_snapshot.cam_right;
                let cam_up    = drag_snapshot.cam_up;
                let px_per_world = drag_snapshot.px_per_world;
                drop(drag_snapshot);

                if !already_active && dist < DRAG_THRESHOLD_PX {
                    return; // still indistinguishable from a click
                }
                if !already_active {
                    drag.borrow_mut().active = true;
                }

                // Convert pixel delta → world delta on the camera-aligned
                // plane through the original node position.
                let world_per_px = if px_per_world > 1e-6 { 1.0 / px_per_world } else { 1.0 };
                let dxw = dx_total as f32 * world_per_px;
                // Screen Y grows downward → invert for world-up axis.
                let dyw = -(dy_total as f32) * world_per_px;
                let new_x = anchor[0] + cam_right[0] * dxw + cam_up[0] * dyw;
                let new_y = anchor[1] + cam_right[1] * dxw + cam_up[1] * dyw;
                let new_z = anchor[2] + cam_right[2] * dxw + cam_up[2] * dyw;
                if let Ok(mut s) = st.try_borrow_mut() {
                    if let Some(node) = s.layout.nodes.get_mut(idx) {
                        node.x = new_x; node.y = new_y; node.z = new_z;
                        s.dirty_layout = true;
                    }
                }
                return;
            }
            drop(drag_snapshot);

            // ── Orbit / pan fallback ──
            let m = ms.borrow().clone();
            if !m.orbiting && !m.panning { return; }
            let dx = (cx - m.last_x) as f32;
            let dy = (cy - m.last_y) as f32;
            ms.borrow_mut().last_x = cx;
            ms.borrow_mut().last_y = cy;

            let Ok(mut s) = st.try_borrow_mut() else { return };
            if m.orbiting {
                s.camera.yaw   -= dx * 0.005;
                s.camera.pitch  = (s.camera.pitch + dy * 0.005).clamp(-1.4, 1.4);
            } else if m.panning {
                let speed = s.camera.distance * 0.002;
                let cos_y = s.camera.yaw.cos();
                let sin_y = s.camera.yaw.sin();
                s.camera.target[0] -= dx * speed * cos_y;
                s.camera.target[1] += dy * speed;
                s.camera.target[2] += dx * speed * sin_y;
            }
        }
    });

    let mu = EventListener::new(&doc, "mouseup", {
        let ms = ms.clone();
        let drag = drag.clone();
        move |_| {
            let was_drag = {
                let d = drag.borrow();
                d.candidate_idx.is_some() && d.active
            };

            // Clear drag candidate / orbit / pan state.
            {
                let mut d = drag.borrow_mut();
                d.candidate_idx = None;
                d.active = false;
            }
            let mut m = ms.borrow_mut();
            m.orbiting = false;
            m.panning  = false;
            drop(m);

            // After a real drag, swallow the impending `click` event so
            // the card's onclick (which would navigate) does not fire.
            if was_drag {
                if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                    let target: web_sys::EventTarget = doc.into();
                    let cb_holder: Rc<RefCell<Option<Closure<dyn FnMut(web_sys::Event)>>>> =
                        Rc::new(RefCell::new(None));
                    let cb_holder2 = cb_holder.clone();
                    let target2 = target.clone();
                    let cb = Closure::wrap(Box::new(move |evt: web_sys::Event| {
                        evt.stop_propagation();
                        evt.prevent_default();
                        if let Some(f) = cb_holder2.borrow_mut().take() {
                            let _ = target2.remove_event_listener_with_callback_and_bool(
                                "click",
                                f.as_ref().unchecked_ref(),
                                true,
                            );
                            // Drop f → frees closure.
                            drop(f);
                        }
                    }) as Box<dyn FnMut(web_sys::Event)>);
                    let _ = target.add_event_listener_with_callback_and_bool(
                        "click",
                        cb.as_ref().unchecked_ref(),
                        true, // capture
                    );
                    *cb_holder.borrow_mut() = Some(cb);
                    // Fallback: if no click arrives within the same tick,
                    // remove the listener via setTimeout.
                    if let Some(win) = web_sys::window() {
                        let target3 = target.clone();
                        let cb_holder3 = cb_holder.clone();
                        let timer = Closure::once_into_js(move || {
                            if let Some(f) = cb_holder3.borrow_mut().take() {
                                let _ = target3.remove_event_listener_with_callback_and_bool(
                                    "click",
                                    f.as_ref().unchecked_ref(),
                                    true,
                                );
                            }
                        });
                        let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                            timer.as_ref().unchecked_ref(),
                            0,
                        );
                        // Leak the timer closure into JS GC (called once).
                        let _ = JsValue::from(timer);
                    }
                }
            }
        }
    });

    let wh = EventListener::new_with_options(
        container_target,
        "wheel",
        gloo_events::EventListenerOptions::enable_prevent_default(),
        {
            let st = state_rc.clone();
            move |evt| {
                evt.prevent_default();
                let Some(e) = evt.dyn_ref::<web_sys::WheelEvent>() else { return };
                let delta = e.delta_y() as f32;
                let factor = if delta < 0.0 { 0.92 } else { 1.08 };
                if let Ok(mut s) = st.try_borrow_mut() {
                    s.camera.distance = (s.camera.distance * factor).clamp(3.0, 100.0);
                }
            }
        },
    );

    let cm = EventListener::new(container_target, "contextmenu", |evt| {
        evt.prevent_default();
    });

    vec![md, mm, mu, wh, cm]
}
