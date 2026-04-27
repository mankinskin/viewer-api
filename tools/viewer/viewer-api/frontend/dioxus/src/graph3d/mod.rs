//! Shared 3-D graph view (WebGPU edges + DOM node cards).
//!
//! Architecture (mirrors the `viewer-api` TS reference and the original
//! ticket-viewer implementation):
//!
//!   - **GPU canvas** (`#webgpu-canvas`): renders edges as animated energy
//!     beams and node occluder quads (depth-only)
//!   - **DOM layer**: caller-supplied node cards positioned each frame via
//!     CSS 3-D transforms
//!   - **Camera**: orbit camera with yaw/pitch/distance/target
//!
//! Caller responsibilities:
//!   1. Build a [`Layout3D`] (positioned nodes + indexed edges) from your
//!      domain data.
//!   2. Render node cards as children of `<Graph3D>` and tag each card
//!      element with a `data-node-idx="N"` attribute matching its index in
//!      `layout.nodes`. The renderer projects world coordinates to screen
//!      pixels and updates `style.transform` on every frame.
//!
//! While mounted, this component takes ownership of `#webgpu-canvas` from
//! `WgpuOverlay` (via [`crate::set_gpu_canvas_owner`]) so the two render
//! pipelines do not clash.

pub mod camera;
pub mod data;
pub mod math;

#[cfg(target_arch = "wasm32")]
mod gpu;
#[cfg(target_arch = "wasm32")]
mod interaction;
#[cfg(target_arch = "wasm32")]
mod interop;
#[cfg(target_arch = "wasm32")]
mod render;

pub use data::{EdgeRef3D, Layout3D, Node3D};

use dioxus::prelude::*;

/// Returns true if the browser exposes `navigator.gpu`.
#[cfg(target_arch = "wasm32")]
pub fn can_use_webgpu() -> bool {
    use js_sys::Reflect;
    use wasm_bindgen::JsValue;
    web_sys::window()
        .map(|w| {
            let nav: JsValue = w.navigator().into();
            let gpu = Reflect::get(&nav, &JsValue::from_str("gpu"))
                .unwrap_or(JsValue::UNDEFINED);
            !gpu.is_undefined()
        })
        .unwrap_or(false)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn can_use_webgpu() -> bool { false }

/// Default DOM container id used by `<Graph3D>`.
pub const DEFAULT_CONTAINER_ID: &str = "graph3d-container";

#[derive(Props, Clone, PartialEq)]
pub struct Graph3DProps {
    /// Positioned nodes and edges to render.
    pub layout: Layout3D,
    /// Node cards. Each card must carry a `data-node-idx="N"` attribute
    /// matching its index in `layout.nodes`.
    pub children: Element,
    /// Optional override for the container element id (used to scope DOM
    /// queries and event listeners).
    #[props(default = DEFAULT_CONTAINER_ID.to_string())]
    pub container_id: String,
    /// Optional override for the inline container `style` attribute.
    #[props(default = String::new())]
    pub container_style: String,
}

#[cfg(not(target_arch = "wasm32"))]
#[component]
pub fn Graph3D(props: Graph3DProps) -> Element {
    let style = if props.container_style.is_empty() {
        "position: absolute; inset: 0; overflow: hidden;".to_string()
    } else {
        props.container_style.clone()
    };
    rsx! {
        div { id: "{props.container_id}", style: "{style}", {props.children} }
    }
}

#[cfg(target_arch = "wasm32")]
#[component]
pub fn Graph3D(props: Graph3DProps) -> Element {
    use std::cell::RefCell;
    use std::rc::Rc;

    use gloo_events::EventListener;
    use wasm_bindgen::JsCast;
    use web_sys::HtmlCanvasElement;

    use crate::set_gpu_canvas_owner;
    use camera::{Camera, CAMERA_FOV};
    use gpu::init_gpu;
    use interop::{create_buf, create_buf_init, USAGE_COPY_DST, USAGE_VERTEX};
    use render::{schedule_raf, RenderState};

    let layout       = props.layout.clone();
    let container_id = props.container_id.clone();
    let style = if props.container_style.is_empty() {
        "position: absolute; inset: 0; overflow: hidden; user-select: none; cursor: grab;".to_string()
    } else {
        props.container_style.clone()
    };

    let mut status: Signal<String> = use_signal(|| "Initialising WebGPU\u{2026}".to_string());
    let alive: Signal<Option<()>>  = use_signal(|| Some(()));
    let _listeners: Signal<Vec<EventListener>> = use_signal(Vec::new);
    let render_rc: Signal<Option<Rc<RefCell<RenderState>>>> = use_signal(|| None);

    set_gpu_canvas_owner(true);
    use_drop(|| set_gpu_canvas_owner(false));

    use_effect(move || {
        let layout       = layout.clone();
        let container_id = container_id.clone();
        let mut status_w = status;
        let mut render_w = render_rc;
        let mut listeners_w = _listeners;
        let alive_r = alive;

        spawn(async move {
            let doc = web_sys::window().unwrap().document().unwrap();
            let canvas: HtmlCanvasElement = match doc.get_element_by_id("webgpu-canvas") {
                Some(el) => match el.dyn_into() {
                    Ok(c) => c,
                    Err(_) => { status_w.set("Canvas element not found".into()); return; }
                },
                None => { status_w.set("No #webgpu-canvas".into()); return; }
            };
            canvas.set_width(canvas.client_width().max(1) as u32);
            canvas.set_height(canvas.client_height().max(1) as u32);

            status_w.set("Requesting GPU device\u{2026}".into());
            let gpu = match init_gpu(canvas).await {
                Ok(g)  => g,
                Err(e) => { status_w.set(format!("GPU init failed: {e}")); return; }
            };

            // Edge instances.
            let (edge_data, edge_count) = layout.build_edge_instances();
            let edge_buf = if edge_data.is_empty() {
                create_buf(&gpu.device, 48, USAGE_VERTEX | USAGE_COPY_DST)
            } else {
                create_buf_init(&gpu.device, &edge_data, USAGE_VERTEX)
            };

            // Camera framing.
            let mut camera = Camera::default();
            if !layout.nodes.is_empty() {
                let (centre, radius) = layout.bounds();
                let _ = CAMERA_FOV;
                camera.frame(centre, radius);
            }

            // Node occluder quads.
            let (node_data, node_count) = layout.build_node_quads();
            let node_quad_buf = if node_data.is_empty() {
                create_buf(&gpu.device, 16, USAGE_VERTEX | USAGE_COPY_DST)
            } else {
                create_buf_init(&gpu.device, &node_data, USAGE_VERTEX)
            };

            let state_rc = Rc::new(RefCell::new(RenderState {
                gpu, layout, camera, edge_buf, edge_count,
                node_quad_buf, node_count, container_id: container_id.clone(),
            }));
            render_w.set(Some(state_rc.clone()));
            status_w.set(String::new());

            listeners_w.set(interaction::install(&container_id, state_rc.clone()));
            schedule_raf(state_rc, alive_r);
        });
    });

    let status_text = status.read().clone();

    rsx! {
        div {
            id: "{props.container_id}",
            style: "{style}",
            {props.children}
            if !status_text.is_empty() {
                div {
                    style: "position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); color: #aaa; font-size: 14px; font-family: sans-serif; text-align: center; pointer-events: none;",
                    "{status_text}"
                }
            }
        }
    }
}
