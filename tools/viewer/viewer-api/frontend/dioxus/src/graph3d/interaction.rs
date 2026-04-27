//! Pointer interaction: orbit, pan, zoom for the 3-D graph.

#![cfg(target_arch = "wasm32")]

use std::cell::RefCell;
use std::rc::Rc;

use gloo_events::EventListener;
use wasm_bindgen::JsCast;

use super::camera::MouseState;
use super::render::RenderState;

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

    let md = EventListener::new(container_target, "mousedown", {
        let ms = ms.clone();
        move |evt| {
            let Some(e) = evt.dyn_ref::<web_sys::MouseEvent>() else { return };
            // Ignore clicks on a node card (they have data-node-idx and own
            // their click events).
            if let Some(target) = e.target() {
                if let Ok(el) = target.dyn_into::<web_sys::Element>() {
                    if el.closest("[data-node-idx]").ok().flatten().is_some() {
                        return;
                    }
                }
            }
            let mut m = ms.borrow_mut();
            m.last_x = e.client_x() as f64;
            m.last_y = e.client_y() as f64;
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
        let st = state_rc.clone();
        move |evt| {
            let m = ms.borrow().clone();
            if !m.orbiting && !m.panning { return; }
            let Some(e) = evt.dyn_ref::<web_sys::MouseEvent>() else { return };
            let cx = e.client_x() as f64;
            let cy = e.client_y() as f64;
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
        move |_| {
            let mut m = ms.borrow_mut();
            m.orbiting = false;
            m.panning  = false;
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
