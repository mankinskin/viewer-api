//! Pointer interaction: orbit, pan, zoom, **and node drag** for the 3-D
//! graph.
//!
//! Drag works by projecting the cursor's screen-space delta onto a plane
//! perpendicular to the camera fwd at the picked node's depth — matches
//! the TS reference (`useMouseInteraction.ts`). No matrix inversion needed:
//! we scale Δpx by `(2·depth·tan(fov/2)) / canvas_height` to convert
//! pixels → world units along the camera's right/up basis.

#![cfg(target_arch = "wasm32")]

use std::{
    cell::{
        Cell,
        RefCell,
    },
    rc::Rc,
};

use dioxus::prelude::EventHandler;
use gloo_events::EventListener;
use wasm_bindgen::JsCast;

use super::{
    camera::{
        Camera,
        MouseState,
    },
    data::Layout3D,
    render::RenderState,
};

mod handlers;

use self::handlers::{
    contextmenu_listener,
    mouse_down_listener,
    mouse_move_listener,
    mouse_up_listener,
    wheel_listener,
};

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
    cam_up: [f32; 3],
    /// Pixels-per-world-unit at the node's depth.
    px_per_world: f32,
}

fn cross(
    a: [f32; 3],
    b: [f32; 3],
) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalise(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-6 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

/// Returns `true` if the event's target lives inside a DOM subtree marked
/// with `data-graph-passthrough="false"`. Used to keep settings panels and
/// other overlay UI from triggering camera orbit/pan/zoom.
fn target_is_passthrough_blocked(evt: &web_sys::Event) -> bool {
    let Some(target) = evt.target() else {
        return false;
    };

    // Pointer events can target non-Element nodes (for example text nodes).
    // Walk to a containing Element so `closest(...)` works reliably.
    let el = if let Ok(el) = target.clone().dyn_into::<web_sys::Element>() {
        el
    } else if let Ok(node) = target.dyn_into::<web_sys::Node>() {
        let Some(parent) = node.parent_element() else {
            return false;
        };
        parent
    } else {
        return false;
    };

    matches!(
        el.closest(
            ".graph-settings-overlay, [data-graph-passthrough=\"false\"]",
        ),
        Ok(Some(_))
    )
}

/// Install mouse listeners on the graph container + document, returning the
/// listener handles. Drop them to detach (Dioxus stores them in a Signal).
pub(crate) fn install(
    container_id: &str,
    state_rc: Rc<RefCell<RenderState>>,
    on_layout_change: Option<EventHandler<Layout3D>>,
    on_camera_change: Option<EventHandler<Camera>>,
    on_deselect: Option<EventHandler<()>>,
) -> Vec<EventListener> {
    let Some(document) = web_sys::window().and_then(|window| window.document())
    else {
        return Vec::new();
    };
    let container = document.get_element_by_id(container_id);
    let container_target: &web_sys::EventTarget = match &container {
        Some(el) => el.as_ref(),
        None => document.as_ref(),
    };

    let mouse_state = Rc::new(RefCell::new(MouseState::default()));
    let drag_state = Rc::new(RefCell::new(DragState::default()));
    let suppress_contextmenu = Rc::new(Cell::new(false));

    let mouse_down = mouse_down_listener(
        container_target,
        mouse_state.clone(),
        drag_state.clone(),
        state_rc.clone(),
        suppress_contextmenu.clone(),
        on_deselect,
    );
    let mouse_move = mouse_move_listener(
        &document,
        mouse_state.clone(),
        drag_state.clone(),
        state_rc.clone(),
        suppress_contextmenu.clone(),
        on_camera_change.clone(),
    );
    let mouse_up = mouse_up_listener(
        &document,
        mouse_state,
        drag_state,
        state_rc.clone(),
        on_layout_change,
        on_camera_change.clone(),
    );
    let wheel = wheel_listener(container_target, state_rc, on_camera_change);
    let contextmenu = contextmenu_listener(&document, suppress_contextmenu);

    vec![mouse_down, mouse_move, mouse_up, wheel, contextmenu]
}
