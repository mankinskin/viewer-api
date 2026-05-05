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

pub use data::{EdgeRef3D, Layout3D, Node3D};
pub use camera::{CameraCommand, LayoutMode, Projection};

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
    let style = if props.container_style.is_empty() {
        "position: absolute; inset: 0; overflow: hidden;".to_string()
    } else {
        props.container_style.clone()
    };
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
    use std::cell::RefCell;
    use std::rc::Rc;

    use gloo_events::EventListener;
    use js_sys::Promise;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::GpuDevice;

    use crate::effects::{register_frame_callback, shared_gpu, FrameCallbackHandle};
    use camera::{Camera, CAMERA_FOV};
    use gpu::init_gpu;
    use interop::{create_buf, create_buf_init, USAGE_COPY_DST, USAGE_VERTEX};
    use render::{render_frame, RenderState};

    let layout       = props.layout.clone();
    let container_id = props.container_id.clone();
    let projection   = props.projection;
    let layout_mode  = props.layout_mode;
    let on_layout_mode_change = props.on_layout_mode_change.clone();
    let on_projection_change  = props.on_projection_change.clone();
    let style = if props.container_style.is_empty() {
        "position: absolute; inset: 0; overflow: hidden; user-select: none; cursor: grab;".to_string()
    } else {
        props.container_style.clone()
    };

    let mut status: Signal<String> = use_signal(|| "Initialising WebGPU\u{2026}".to_string());
    let _listeners: Signal<Vec<EventListener>> = use_signal(Vec::new);
    let render_rc: Signal<Option<Rc<RefCell<RenderState>>>> = use_signal(|| None);
    // Holding the FrameCallbackHandle in a signal ensures the callback is
    // unregistered from the overlay loop when this component unmounts.
    let frame_handle: Signal<Option<Rc<FrameCallbackHandle>>> = use_signal(|| None);

    use_effect(move || {
        let layout       = layout.clone();
        let container_id = container_id.clone();
        let mut status_w = status;
        let mut render_w = render_rc;
        let mut listeners_w = _listeners;
        let mut handle_w = frame_handle;

        spawn(async move {
            // Wait until the WgpuOverlay has bootstrapped its shared device.
            let shared = loop {
                if let Some(g) = shared_gpu() { break g; }
                let p = Promise::new(&mut |resolve, _reject| {
                    if let Some(win) = web_sys::window() {
                        let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                            resolve.unchecked_ref(), 16);
                    }
                });
                let _ = JsFuture::from(p).await;
            };

            let device: GpuDevice = match shared.device.clone().dyn_into() {
                Ok(d)  => d,
                Err(_) => { status_w.set("Shared GPU device cast failed".into()); return; }
            };
            {
                let lbl = js_sys::Reflect::get(&shared.device, &"label".into())
                    .ok().and_then(|v| v.as_string()).unwrap_or_default();
                tracing::info!(target: "graph3d::init", device_label = %lbl, "received shared device");
            }

            // Read current canvas backing-store size; the overlay loop
            // resizes it each frame, so any value here will be replaced
            // before our first draw — but we need an initial depth texture.
            let (init_w, init_h) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id("webgpu-canvas"))
                .and_then(|el| el.dyn_into::<web_sys::HtmlCanvasElement>().ok())
                .map(|c| (c.width().max(1), c.height().max(1)))
                .unwrap_or((1, 1));

            let gpu = match init_gpu(device, &shared.format, init_w, init_h) {
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
                dirty_layout: false, projection,
            }));
            render_w.set(Some(state_rc.clone()));
            status_w.set(String::new());

            listeners_w.set(interaction::install(&container_id, state_rc.clone()));

            // Register per-frame callback into the overlay's loop.
            let state_for_cb = state_rc.clone();
            let handle = register_frame_callback(move |frame| {
                if let Ok(mut st) = state_for_cb.try_borrow_mut() {
                    render_frame(&mut st, frame);
                }
            });
            handle_w.set(Some(Rc::new(handle)));
        });
    });

    let status_text = status.read().clone();

    // Push layout updates from props into the live RenderState so callers can
    // change the Layout3D (e.g. switch algorithms, edit parameters) without
    // re-mounting the component. Setting `dirty_layout` re-uploads the GPU
    // instance buffers on the next frame and reframes the camera if the
    // bounds shifted significantly.
    if let Some(rc) = render_rc.read().as_ref() {
        if let Ok(mut st) = rc.try_borrow_mut() {
            if st.layout != props.layout {
                let (centre, radius) = props.layout.bounds();
                st.layout = props.layout.clone();
                st.dirty_layout = true;
                st.camera.frame(centre, radius);
            }
            if st.projection != props.projection {
                st.projection = props.projection;
            }
        }
    }

    // Apply imperative camera commands.  We use a `use_hook` to remember
    // the last applied `seq` so each unique generation triggers exactly
    // one command.  This pattern lets the parent re-apply the same
    // logical command (e.g. "reset camera") by simply bumping the seq.
    let mut last_cam_seq: Signal<u64> = use_hook(|| Signal::new(0));
    if props.camera_command_seq != *last_cam_seq.peek() {
        last_cam_seq.set(props.camera_command_seq);
        if let Some(cmd) = props.camera_command.as_ref() {
            if let Some(rc) = render_rc.read().as_ref() {
                if let Ok(mut st) = rc.try_borrow_mut() {
                    let bounds = st.layout.bounds();
                    st.camera.apply_command(cmd, bounds);
                }
            }
        }
    }

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

// ── Built-in settings overlay ─────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct GraphSettingsOverlayProps {
    layout_mode: LayoutMode,
    projection:  Projection,
    on_layout_mode_change: Option<EventHandler<LayoutMode>>,
    on_projection_change:  Option<EventHandler<Projection>>,
}

fn opt_btn_style(active: bool) -> String {
    let (bg, border, color) = if active {
        ("rgba(79,140,255,0.20)", "1px solid rgba(79,140,255,0.50)", "#93bbff")
    } else {
        ("rgba(255,255,255,0.05)", "1px solid rgba(255,255,255,0.10)", "#aaa")
    };
    format!(
        "flex:1; padding:5px 0; border-radius:5px; border:{border}; \
         background:{bg}; color:{color}; font-size:11px; font-weight:500; \
         cursor:pointer; text-align:center; white-space:nowrap;"
    )
}

#[component]
fn GraphSettingsOverlay(props: GraphSettingsOverlayProps) -> Element {
    let mut open: Signal<bool> = use_hook(|| Signal::new(false));
    let cur_layout = props.layout_mode;
    let cur_proj   = props.projection;

    let has_callbacks =
        props.on_layout_mode_change.is_some() || props.on_projection_change.is_some();

    if !has_callbacks { return rsx! {}; }

    let on_lm_change = props.on_layout_mode_change.clone();
    let on_pr_change = props.on_projection_change.clone();

    rsx! {
        div {
            style: "position: absolute; bottom: 12px; right: 12px; z-index: 100; display: flex; flex-direction: column; align-items: flex-end;",
            // Floating panel (rendered above the button when open)
            if *open.read() {
                div {
                    style: "
                        margin-bottom: 6px;
                        min-width: 200px;
                        background: rgba(18, 20, 28, 0.88);
                        border: 1px solid rgba(255,255,255,0.10);
                        border-radius: 9px;
                        padding: 12px 14px;
                        box-shadow: 0 6px 24px rgba(0,0,0,0.5);
                        font-family: sans-serif;
                        font-size: 12px;
                        color: #ccc;
                        backdrop-filter: blur(8px);
                        -webkit-backdrop-filter: blur(8px);
                    ",
                    if on_lm_change.is_some() {
                        {
                            let on_h3d = on_lm_change.clone();
                            let on_f2d = on_lm_change.clone();
                            rsx! {
                                div {
                                    style: "font-size:10px; font-weight:700; letter-spacing:0.07em; text-transform:uppercase; color:#666; margin-bottom:7px;",
                                    "Layout"
                                }
                                div { style: "display:flex; gap:6px; margin-bottom:10px;",
                                    button {
                                        style: "{opt_btn_style(cur_layout == LayoutMode::Hierarchical3D)}",
                                        onclick: move |_| {
                                            if let Some(ref cb) = on_h3d { cb.call(LayoutMode::Hierarchical3D); }
                                        },
                                        "Hierarchical 3D"
                                    }
                                    button {
                                        style: "{opt_btn_style(cur_layout == LayoutMode::Flat2D)}",
                                        onclick: move |_| {
                                            if let Some(ref cb) = on_f2d { cb.call(LayoutMode::Flat2D); }
                                        },
                                        "Flat 2D"
                                    }
                                }
                            }
                        }
                    }
                    if on_pr_change.is_some() {
                        {
                            let on_persp = on_pr_change.clone();
                            let on_ortho = on_pr_change.clone();
                            rsx! {
                                div {
                                    style: "font-size:10px; font-weight:700; letter-spacing:0.07em; text-transform:uppercase; color:#666; margin-bottom:7px;",
                                    "Projection"
                                }
                                div { style: "display:flex; gap:6px;",
                                    button {
                                        style: "{opt_btn_style(cur_proj == Projection::Perspective)}",
                                        onclick: move |_| {
                                            if let Some(ref cb) = on_persp { cb.call(Projection::Perspective); }
                                        },
                                        "Perspective"
                                    }
                                    button {
                                        style: "{opt_btn_style(cur_proj == Projection::Orthographic)}",
                                        onclick: move |_| {
                                            if let Some(ref cb) = on_ortho { cb.call(Projection::Orthographic); }
                                        },
                                        "Orthographic"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Gear button — minimal, transparent, bottom-right corner
            button {
                title: "Graph settings",
                style: "
                    width: 28px; height: 28px;
                    border-radius: 6px;
                    border: 1px solid rgba(255,255,255,0.08);
                    background: rgba(0,0,0,0.30);
                    color: rgba(255,255,255,0.45);
                    font-size: 14px;
                    cursor: pointer;
                    display: flex; align-items: center; justify-content: center;
                    backdrop-filter: blur(4px);
                    -webkit-backdrop-filter: blur(4px);
                    padding: 0;
                    line-height: 1;
                ",
                onclick: move |evt| {
                    evt.stop_propagation();
                    let cur = *open.read();
                    *open.write() = !cur;
                },
                "\u{2699}"
            }
        }
    }
}
