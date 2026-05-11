use std::{
    cell::{
        Cell,
        RefCell,
    },
    rc::Rc,
};

use dioxus::prelude::EventHandler;
use gloo_events::EventListener;
use wasm_bindgen::{
    closure::Closure,
    JsCast,
    JsValue,
};

use super::{
    super::{
        camera::{
            Camera,
            MouseState,
            CAMERA_FOV,
        },
        data::Layout3D,
        render::RenderState,
    },
    cross,
    normalise,
    target_is_passthrough_blocked,
    DragState,
    DRAG_THRESHOLD_PX,
};

pub(super) fn mouse_down_listener(
    container_target: &web_sys::EventTarget,
    mouse_state: Rc<RefCell<MouseState>>,
    drag_state: Rc<RefCell<DragState>>,
    state_rc: Rc<RefCell<RenderState>>,
    suppress_contextmenu: Rc<Cell<bool>>,
) -> EventListener {
    EventListener::new(container_target, "mousedown", {
        move |evt| {
            let Some(event) = evt.dyn_ref::<web_sys::MouseEvent>() else {
                return;
            };
            if target_is_passthrough_blocked(evt) {
                return;
            }

            suppress_contextmenu.set(false);
            let cursor_x = event.client_x() as f64;
            let cursor_y = event.client_y() as f64;

            if let Some(card_idx) = find_card_index(event) {
                if event.button() == 0 {
                    record_drag_candidate(
                        &drag_state,
                        &state_rc,
                        card_idx,
                        cursor_x,
                        cursor_y,
                    );
                    return;
                }
            }

            start_camera_interaction(
                &mouse_state,
                cursor_x,
                cursor_y,
                event.button(),
                event.shift_key(),
            );
        }
    })
}

pub(super) fn mouse_move_listener(
    document: &web_sys::Document,
    mouse_state: Rc<RefCell<MouseState>>,
    drag_state: Rc<RefCell<DragState>>,
    state_rc: Rc<RefCell<RenderState>>,
    suppress_contextmenu: Rc<Cell<bool>>,
    on_camera_change: Option<EventHandler<Camera>>,
) -> EventListener {
    EventListener::new(document, "mousemove", {
        move |evt| {
            let Some(event) = evt.dyn_ref::<web_sys::MouseEvent>() else {
                return;
            };

            if mouse_state.borrow().panning {
                suppress_contextmenu.set(true);
            }

            let cursor_x = event.client_x() as f64;
            let cursor_y = event.client_y() as f64;
            if handle_active_drag(&drag_state, &state_rc, cursor_x, cursor_y) {
                return;
            }

            update_camera_motion(
                &mouse_state,
                &state_rc,
                cursor_x,
                cursor_y,
                on_camera_change.as_ref(),
            );
        }
    })
}

pub(super) fn mouse_up_listener(
    document: &web_sys::Document,
    mouse_state: Rc<RefCell<MouseState>>,
    drag_state: Rc<RefCell<DragState>>,
    state_rc: Rc<RefCell<RenderState>>,
    on_layout_change: Option<EventHandler<Layout3D>>,
    on_camera_change: Option<EventHandler<Camera>>,
) -> EventListener {
    EventListener::new(document, "mouseup", move |_| {
        let camera_was_active = {
            let state = mouse_state.borrow();
            state.orbiting || state.panning
        };
        let was_drag = clear_interaction_state(&mouse_state, &drag_state);
        if was_drag {
            if let Some(handler) = on_layout_change.clone() {
                if let Ok(state) = state_rc.try_borrow() {
                    handler.call(state.layout.clone());
                }
            }
            install_click_suppressor();
        }
        if camera_was_active {
            if let Some(handler) = on_camera_change.clone() {
                if let Ok(state) = state_rc.try_borrow() {
                    handler.call(state.camera.clone());
                }
            }
        }
    })
}

pub(super) fn wheel_listener(
    container_target: &web_sys::EventTarget,
    state_rc: Rc<RefCell<RenderState>>,
    on_camera_change: Option<EventHandler<Camera>>,
) -> EventListener {
    EventListener::new_with_options(
        container_target,
        "wheel",
        gloo_events::EventListenerOptions::enable_prevent_default(),
        move |evt| {
            if target_is_passthrough_blocked(evt) {
                return;
            }
            evt.prevent_default();
            let Some(event) = evt.dyn_ref::<web_sys::WheelEvent>() else {
                return;
            };

            let delta = event.delta_y() as f32;
            let factor = if delta < 0.0 { 0.92 } else { 1.08 };
            if let Ok(mut state) = state_rc.try_borrow_mut() {
                state.camera_goal = None;
                state.camera.distance =
                    (state.camera.distance * factor).clamp(3.0, 100.0);
                if let Some(handler) = on_camera_change.as_ref() {
                    handler.call(state.camera.clone());
                }
            }
        },
    )
}

pub(super) fn contextmenu_listener(
    document: &web_sys::Document,
    suppress_contextmenu: Rc<Cell<bool>>,
) -> EventListener {
    EventListener::new_with_options(
        document,
        "contextmenu",
        gloo_events::EventListenerOptions::enable_prevent_default(),
        move |evt| {
            if suppress_contextmenu.replace(false) {
                evt.prevent_default();
            }
        },
    )
}

fn find_card_index(event: &web_sys::MouseEvent) -> Option<usize> {
    let target = event.target()?;
    let element = target.dyn_into::<web_sys::Element>().ok()?;
    let card = element.closest("[data-node-idx]").ok()??;
    let index = card.get_attribute("data-node-idx")?;
    index.parse::<usize>().ok()
}

fn record_drag_candidate(
    drag_state: &Rc<RefCell<DragState>>,
    state_rc: &Rc<RefCell<RenderState>>,
    idx: usize,
    cursor_x: f64,
    cursor_y: f64,
) {
    let mut drag = drag_state.borrow_mut();
    drag.candidate_idx = Some(idx);
    drag.start_x = cursor_x;
    drag.start_y = cursor_y;
    drag.active = false;

    let Ok(state) = state_rc.try_borrow() else {
        return;
    };
    let Some(node) = state.layout.nodes.get(idx) else {
        return;
    };

    drag.anchor = [node.x, node.y, node.z];
    let eye = state.camera.eye();
    let target = state.camera.target;
    let forward =
        normalise([target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]]);
    let right = normalise(cross(forward, [0.0, 1.0, 0.0]));
    let up = normalise(cross(right, forward));
    drag.cam_right = right;
    drag.cam_up = up;

    let to_node = [
        drag.anchor[0] - eye[0],
        drag.anchor[1] - eye[1],
        drag.anchor[2] - eye[2],
    ];
    let depth = (to_node[0] * forward[0]
        + to_node[1] * forward[1]
        + to_node[2] * forward[2])
        .abs()
        .max(0.1);
    let dpr = web_sys::window()
        .map(|window| window.device_pixel_ratio().clamp(1.0, 4.0))
        .unwrap_or(1.0) as f32;
    let canvas_h = (state.gpu.canvas_h.max(1) as f32) / dpr;
    let world_per_px = 2.0 * depth * (CAMERA_FOV * 0.5).tan() / canvas_h;
    drag.px_per_world = if world_per_px > 1e-6 {
        1.0 / world_per_px
    } else {
        1.0
    };
}

fn start_camera_interaction(
    mouse_state: &Rc<RefCell<MouseState>>,
    cursor_x: f64,
    cursor_y: f64,
    button: i16,
    shift_key: bool,
) {
    let mut state = mouse_state.borrow_mut();
    state.last_x = cursor_x;
    state.last_y = cursor_y;
    if button == 2 || (button == 0 && shift_key) {
        state.panning = true;
    } else if button == 0 {
        state.orbiting = true;
    }
}

fn handle_active_drag(
    drag_state: &Rc<RefCell<DragState>>,
    state_rc: &Rc<RefCell<RenderState>>,
    cursor_x: f64,
    cursor_y: f64,
) -> bool {
    let drag_snapshot = drag_state.borrow();
    let Some(idx) = drag_snapshot.candidate_idx else {
        return false;
    };

    let dx_total = cursor_x - drag_snapshot.start_x;
    let dy_total = cursor_y - drag_snapshot.start_y;
    let dist = (dx_total * dx_total + dy_total * dy_total).sqrt();
    let already_active = drag_snapshot.active;
    let anchor = drag_snapshot.anchor;
    let cam_right = drag_snapshot.cam_right;
    let cam_up = drag_snapshot.cam_up;
    let px_per_world = drag_snapshot.px_per_world;
    drop(drag_snapshot);

    if !already_active && dist < DRAG_THRESHOLD_PX {
        return true;
    }
    if !already_active {
        drag_state.borrow_mut().active = true;
    }

    update_dragged_node(
        state_rc,
        idx,
        anchor,
        cam_right,
        cam_up,
        px_per_world,
        dx_total as f32,
        dy_total as f32,
    );
    true
}

fn update_dragged_node(
    state_rc: &Rc<RefCell<RenderState>>,
    idx: usize,
    anchor: [f32; 3],
    cam_right: [f32; 3],
    cam_up: [f32; 3],
    px_per_world: f32,
    dx_total: f32,
    dy_total: f32,
) {
    let world_per_px = if px_per_world > 1e-6 {
        1.0 / px_per_world
    } else {
        1.0
    };
    let dxw = dx_total * world_per_px;
    let dyw = -dy_total * world_per_px;
    let new_x = anchor[0] + cam_right[0] * dxw + cam_up[0] * dyw;
    let new_y = anchor[1] + cam_right[1] * dxw + cam_up[1] * dyw;
    let new_z = anchor[2] + cam_right[2] * dxw + cam_up[2] * dyw;

    if let Ok(mut state) = state_rc.try_borrow_mut() {
        if let Some(node) = state.layout.nodes.get_mut(idx) {
            node.x = new_x;
            node.y = new_y;
            node.z = new_z;
        }
        if let Some(node) = state.target_layout.nodes.get_mut(idx) {
            node.x = new_x;
            node.y = new_y;
            node.z = new_z;
        }
        if let Some(node) = state.base_layout.nodes.get_mut(idx) {
            node.x = new_x;
            node.y = new_y;
            node.z = new_z;
        }
        state.dirty_layout = true;
    }
}

fn update_camera_motion(
    mouse_state: &Rc<RefCell<MouseState>>,
    state_rc: &Rc<RefCell<RenderState>>,
    cursor_x: f64,
    cursor_y: f64,
    on_camera_change: Option<&EventHandler<Camera>>,
) {
    let state = mouse_state.borrow().clone();
    if !state.orbiting && !state.panning {
        return;
    }

    let dx = (cursor_x - state.last_x) as f32;
    let dy = (cursor_y - state.last_y) as f32;
    mouse_state.borrow_mut().last_x = cursor_x;
    mouse_state.borrow_mut().last_y = cursor_y;
    if dx.abs() < 0.001 && dy.abs() < 0.001 {
        return;
    }

    let Ok(mut render_state) = state_rc.try_borrow_mut() else {
        return;
    };
    render_state.camera_goal = None;
    if state.orbiting {
        render_state.camera.yaw -= dx * 0.005;
        render_state.camera.pitch =
            (render_state.camera.pitch + dy * 0.005).clamp(-1.4, 1.4);
    } else if state.panning {
        apply_screen_plane_pan(&mut render_state.camera, dx, dy);
    }

    let camera = render_state.camera.clone();
    drop(render_state);
    if let Some(handler) = on_camera_change {
        handler.call(camera);
    }
}

fn apply_screen_plane_pan(
    camera: &mut Camera,
    dx: f32,
    dy: f32,
) {
    let speed = camera.distance * 0.002;
    let eye = camera.eye();
    let forward = normalise([
        camera.target[0] - eye[0],
        camera.target[1] - eye[1],
        camera.target[2] - eye[2],
    ]);
    let raw_right = cross(forward, [0.0, 1.0, 0.0]);
    let right = if raw_right[0].abs() + raw_right[1].abs() + raw_right[2].abs()
        < 1e-6
    {
        [1.0, 0.0, 0.0]
    } else {
        normalise(raw_right)
    };
    let up = normalise(cross(right, forward));

    for axis in 0..3 {
        camera.target[axis] -= right[axis] * dx * speed;
        camera.target[axis] += up[axis] * dy * speed;
    }
}

fn clear_interaction_state(
    mouse_state: &Rc<RefCell<MouseState>>,
    drag_state: &Rc<RefCell<DragState>>,
) -> bool {
    let was_drag = {
        let drag = drag_state.borrow();
        drag.candidate_idx.is_some() && drag.active
    };

    {
        let mut drag = drag_state.borrow_mut();
        drag.candidate_idx = None;
        drag.active = false;
    }

    let mut state = mouse_state.borrow_mut();
    state.orbiting = false;
    state.panning = false;
    was_drag
}

fn install_click_suppressor() {
    let Some(document) = web_sys::window().and_then(|window| window.document())
    else {
        return;
    };
    let target: web_sys::EventTarget = document.into();
    let callback_holder: Rc<
        RefCell<Option<Closure<dyn FnMut(web_sys::Event)>>>,
    > = Rc::new(RefCell::new(None));
    let callback_holder_for_click = callback_holder.clone();
    let target_for_click = target.clone();
    let callback = Closure::wrap(Box::new(move |evt: web_sys::Event| {
        evt.stop_propagation();
        evt.prevent_default();
        if let Some(callback) = callback_holder_for_click.borrow_mut().take() {
            let _ = target_for_click
                .remove_event_listener_with_callback_and_bool(
                    "click",
                    callback.as_ref().unchecked_ref(),
                    true,
                );
            drop(callback);
        }
    }) as Box<dyn FnMut(web_sys::Event)>);
    let _ = target.add_event_listener_with_callback_and_bool(
        "click",
        callback.as_ref().unchecked_ref(),
        true,
    );
    *callback_holder.borrow_mut() = Some(callback);

    if let Some(window) = web_sys::window() {
        let target_for_timeout = target.clone();
        let callback_holder_for_timeout = callback_holder.clone();
        let timer = Closure::once_into_js(move || {
            if let Some(callback) =
                callback_holder_for_timeout.borrow_mut().take()
            {
                let _ = target_for_timeout
                    .remove_event_listener_with_callback_and_bool(
                        "click",
                        callback.as_ref().unchecked_ref(),
                        true,
                    );
            }
        });
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            timer.as_ref().unchecked_ref(),
            0,
        );
        let _ = JsValue::from(timer);
    }
}

#[cfg(test)]
mod tests {
    use super::apply_screen_plane_pan;
    use crate::graph3d::camera::Camera;

    fn normalised(vector: [f32; 3]) -> [f32; 3] {
        let length =
            (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2])
                .sqrt();
        [vector[0] / length, vector[1] / length, vector[2] / length]
    }

    fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    }

    #[test]
    fn screen_plane_pan_has_no_forward_component() {
        let mut camera = Camera {
            yaw: 0.8,
            pitch: 0.55,
            distance: 18.0,
            target: [2.0, -1.0, 4.0],
        };
        let eye = camera.eye();
        let forward = normalised([
            camera.target[0] - eye[0],
            camera.target[1] - eye[1],
            camera.target[2] - eye[2],
        ]);
        let before = camera.target;

        apply_screen_plane_pan(&mut camera, 120.0, 80.0);

        let delta = [
            camera.target[0] - before[0],
            camera.target[1] - before[1],
            camera.target[2] - before[2],
        ];

        assert!(dot(delta, forward).abs() < 1e-5, "delta={delta:?} forward={forward:?}");
    }
}
