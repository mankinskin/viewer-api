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
//! While mounted, this component reuses the `WgpuOverlay`'s shared
//! `GPUDevice` and registers a per-frame callback (via
//! [`crate::effects::register_frame_callback`]) so its pass composites into
//! the same swap-chain texture as the smoke / particle effects, with
//! `loadOp: "load"` preserving the overlay's render underneath.

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
mod settings_overlay;

pub use data::{EdgeRef3D, Layout3D, Node3D, NodeCardProfile};
pub use camera::{Camera, CameraCommand, LayoutMode, Projection};

use dioxus::prelude::*;
use self::settings_overlay::GraphSettingsOverlay;

#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};

#[cfg(target_arch = "wasm32")]
use gloo_events::EventListener;
#[cfg(target_arch = "wasm32")]
use js_sys::Promise;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::GpuDevice;

#[cfg(target_arch = "wasm32")]
use crate::effects::{register_frame_callback, shared_gpu, FrameCallbackHandle};
#[cfg(target_arch = "wasm32")]
use gpu::init_gpu;
#[cfg(target_arch = "wasm32")]
use interop::{create_buf, create_buf_init, USAGE_COPY_DST, USAGE_VERTEX};
#[cfg(target_arch = "wasm32")]
use render::{render_frame, RenderState};

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
    /// Called when the interactive canvas mutates the layout, for example
    /// after a node drag completes.
    #[props(default)]
    pub on_layout_change: Option<EventHandler<Layout3D>>,
    /// Initial camera state restored on mount.
    #[props(default)]
    pub initial_camera: Option<Camera>,
    /// Called when the interactive canvas mutates the camera state.
    #[props(default)]
    pub on_camera_change: Option<EventHandler<Camera>>,
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
    /// Optional imperative camera command (e.g. "reset to top-down").
    /// Paired with `camera_command_seq` so the same command value can be
    /// re-applied by bumping the seq.  See [`CameraCommand`] for details.
    #[props(default)]
    pub camera_command: Option<CameraCommand>,
    /// Monotonic generation counter for `camera_command`.  The component
    /// applies the command once per new `seq` value via an internal
    /// `use_hook(last_seq)` tracker.  Defaults to `0`; callers issuing a
    /// command should always pass a strictly increasing value.
    #[props(default = 0)]
    pub camera_command_seq: u64,
    /// Camera projection mode.  Defaults to [`Projection::Perspective`].
    #[props(default)]
    pub projection: Projection,
    /// Currently active layout mode — shown in the built-in settings panel.
    /// Defaults to [`LayoutMode::Hierarchical3D`].
    #[props(default)]
    pub layout_mode: LayoutMode,
    /// Called when the user picks a different layout mode in the settings panel.
    #[props(default)]
    pub on_layout_mode_change: Option<EventHandler<LayoutMode>>,
    /// Called when the user picks a different projection in the settings panel.
    #[props(default)]
    pub on_projection_change: Option<EventHandler<Projection>>,
}

#[cfg(not(target_arch = "wasm32"))]
#[component]
pub fn Graph3D(props: Graph3DProps) -> Element {
    let style = graph_container_style(&props.container_style, false);
    rsx! {
        div { id: "{props.container_id}", style: "{style}",
            {props.children}
            GraphSettingsOverlay {
                layout_mode: props.layout_mode,
                projection: props.projection,
                on_layout_mode_change: props.on_layout_mode_change,
                on_projection_change: props.on_projection_change,
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[component]
pub fn Graph3D(props: Graph3DProps) -> Element {
    let layout = props.layout.clone();
    let container_id = props.container_id.clone();
    let initial_camera = props.initial_camera.clone();
    let on_layout_change = props.on_layout_change.clone();
    let on_camera_change = props.on_camera_change.clone();
    let projection = props.projection;
    let layout_mode = props.layout_mode;
    let on_layout_mode_change = props.on_layout_mode_change.clone();
    let on_projection_change = props.on_projection_change.clone();
    let style = graph_container_style(&props.container_style, true);

    let status: Signal<String> = use_signal(|| "Initialising WebGPU\u{2026}".to_string());
    let listeners: Signal<Vec<EventListener>> = use_signal(Vec::new);
    let render_rc: Signal<Option<Rc<RefCell<RenderState>>>> = use_signal(|| None);
    let frame_handle: Signal<Option<Rc<FrameCallbackHandle>>> = use_signal(|| None);

    use_effect(move || {
        let layout = layout.clone();
        let container_id = container_id.clone();
        let initial_camera = initial_camera.clone();
        start_graph_bootstrap(
            layout,
            container_id,
            initial_camera,
            on_layout_change,
            on_camera_change,
            projection,
            status,
            render_rc,
            listeners,
            frame_handle,
        );
    });

    let status_text = status.read().clone();

    sync_render_state(&render_rc, &props.layout, props.projection, props.on_camera_change.as_ref());

    let mut last_cam_seq: Signal<u64> = use_hook(|| Signal::new(0));
    apply_camera_command_update(&mut last_cam_seq, &props, &render_rc, props.on_camera_change.as_ref());

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
            GraphSettingsOverlay {
                layout_mode: layout_mode,
                projection: projection,
                on_layout_mode_change: on_layout_mode_change,
                on_projection_change: on_projection_change,
            }
        }
    }
}

fn graph_container_style(container_style: &str, interactive: bool) -> String {
    if !container_style.is_empty() {
        return container_style.to_string();
    }

    if interactive {
        "position: absolute; inset: 0; overflow: hidden; user-select: none; cursor: grab;"
            .to_string()
    } else {
        "position: absolute; inset: 0; overflow: hidden;".to_string()
    }
}

#[cfg(target_arch = "wasm32")]
fn start_graph_bootstrap(
    layout: Layout3D,
    container_id: String,
    initial_camera: Option<Camera>,
    on_layout_change: Option<EventHandler<Layout3D>>,
    on_camera_change: Option<EventHandler<Camera>>,
    projection: Projection,
    mut status: Signal<String>,
    mut render_rc: Signal<Option<Rc<RefCell<RenderState>>>>,
    mut listeners: Signal<Vec<EventListener>>,
    mut frame_handle: Signal<Option<Rc<FrameCallbackHandle>>>,
) {
    spawn(async move {
        let shared = loop {
            if let Some(shared) = shared_gpu() {
                break shared;
            }
            let promise = Promise::new(&mut |resolve, _reject| {
                if let Some(window) = web_sys::window() {
                    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                        resolve.unchecked_ref(),
                        16,
                    );
                }
            });
            let _ = JsFuture::from(promise).await;
        };

        let device: GpuDevice = match shared.device.clone().dyn_into() {
            Ok(device) => device,
            Err(_) => {
                status.set("Shared GPU device cast failed".into());
                return;
            }
        };

        let label = js_sys::Reflect::get(&shared.device, &"label".into())
            .ok()
            .and_then(|value| value.as_string())
            .unwrap_or_default();
        tracing::info!(target: "graph3d::init", device_label = %label, "received shared device");

        let (init_w, init_h) = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("webgpu-canvas"))
            .and_then(|element| element.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            .map(|canvas| (canvas.width().max(1), canvas.height().max(1)))
            .unwrap_or((1, 1));

        let gpu = match init_gpu(device, &shared.format, init_w, init_h) {
            Ok(gpu) => gpu,
            Err(error) => {
                status.set(format!("GPU init failed: {error}"));
                return;
            }
        };

        let (edge_data, edge_count) = layout.build_edge_instances();
        let edge_buf = if edge_data.is_empty() {
            create_buf(&gpu.device, 48, USAGE_VERTEX | USAGE_COPY_DST)
        } else {
            create_buf_init(&gpu.device, &edge_data, USAGE_VERTEX)
        };

        let camera = initial_camera.unwrap_or_else(|| {
            let mut camera = Camera::default();
            if !layout.nodes.is_empty() {
                let (centre, radius) = layout.bounds();
                camera.frame(centre, radius);
            }
            camera
        });
        let emitted_camera = camera.clone();

        let (node_data, node_count) = layout.build_node_quads();
        let node_quad_buf = if node_data.is_empty() {
            create_buf(&gpu.device, 16, USAGE_VERTEX | USAGE_COPY_DST)
        } else {
            create_buf_init(&gpu.device, &node_data, USAGE_VERTEX)
        };

        let state_rc = Rc::new(RefCell::new(RenderState {
            gpu,
            layout,
            camera,
            edge_buf,
            edge_count,
            node_quad_buf,
            node_count,
            container_id: container_id.clone(),
            dirty_layout: false,
            projection,
        }));
        render_rc.set(Some(state_rc.clone()));
        status.set(String::new());
        if let Some(handler) = on_camera_change.as_ref() {
            handler.call(emitted_camera);
        }
        listeners.set(interaction::install(
            &container_id,
            state_rc.clone(),
            on_layout_change,
            on_camera_change,
        ));

        let state_for_callback = state_rc.clone();
        let handle = register_frame_callback(move |frame| {
            if let Ok(mut state) = state_for_callback.try_borrow_mut() {
                render_frame(&mut state, frame);
            }
        });
        frame_handle.set(Some(Rc::new(handle)));
    });
}

#[cfg(target_arch = "wasm32")]
fn sync_render_state(
    render_rc: &Signal<Option<Rc<RefCell<RenderState>>>>,
    layout: &Layout3D,
    projection: Projection,
    on_camera_change: Option<&EventHandler<Camera>>,
) {
    let render_state = render_rc.read();
    let Some(render_state) = render_state.as_ref() else {
        return;
    };
    let Ok(mut state) = render_state.try_borrow_mut() else {
        return;
    };

    if state.layout != *layout {
        let (centre, radius) = layout.bounds();
        state.layout = layout.clone();
        state.dirty_layout = true;
        state.camera.frame(centre, radius);
        if let Some(handler) = on_camera_change {
            handler.call(state.camera.clone());
        }
    }

    if state.projection != projection {
        state.projection = projection;
    }
}

#[cfg(target_arch = "wasm32")]
fn apply_camera_command_update(
    last_cam_seq: &mut Signal<u64>,
    props: &Graph3DProps,
    render_rc: &Signal<Option<Rc<RefCell<RenderState>>>>,
    on_camera_change: Option<&EventHandler<Camera>>,
) {
    if props.camera_command_seq == *last_cam_seq.peek() {
        return;
    }

    last_cam_seq.set(props.camera_command_seq);
    let Some(command) = props.camera_command.as_ref() else {
        return;
    };
    let render_state = render_rc.read();
    let Some(render_state) = render_state.as_ref() else {
        return;
    };
    let Ok(mut state) = render_state.try_borrow_mut() else {
        return;
    };

    let bounds = state.layout.bounds();
    state.camera.apply_command(command, bounds);
    if let Some(handler) = on_camera_change {
        handler.call(state.camera.clone());
    }
}
