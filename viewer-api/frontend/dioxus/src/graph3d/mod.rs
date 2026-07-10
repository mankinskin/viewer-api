#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

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
pub(crate) mod theme;

#[cfg(target_arch = "wasm32")]
mod gpu;
#[cfg(target_arch = "wasm32")]
mod interaction;
#[cfg(target_arch = "wasm32")]
mod interop;
#[cfg(target_arch = "wasm32")]
mod render;
mod settings_overlay;

pub use camera::{
    Camera,
    CameraCommand,
    CameraMode,
    LayoutMode,
    Projection,
};
pub use data::{
    EdgeRef3D,
    Layout3D,
    Node3D,
    NodeCardProfile,
    NodeViewTransform,
};

use self::{
    settings_overlay::GraphSettingsOverlay,
    theme::GraphThemeSettings,
};
use dioxus::prelude::*;
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use std::{
    cell::RefCell,
    rc::Rc,
};

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
use crate::effects::{
    register_frame_callback,
    shared_gpu,
    FrameCallbackHandle,
};
use crate::{
    effects::wgpu_overlay::PaletteColor,
    store::ThemeStore,
};
#[cfg(target_arch = "wasm32")]
use gpu::init_gpu;
#[cfg(target_arch = "wasm32")]
use interop::{
    create_buf,
    create_buf_init,
    USAGE_COPY_DST,
    USAGE_VERTEX,
};
#[cfg(target_arch = "wasm32")]
use render::{
    render_frame,
    RenderState,
};

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
pub fn can_use_webgpu() -> bool {
    false
}

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
    /// Interactive camera behavior. Defaults to orbit controls.
    #[props(default)]
    pub camera_mode: CameraMode,
    /// Called when the interactive canvas mutates the camera state.
    #[props(default)]
    pub on_camera_change: Option<EventHandler<Camera>>,
    /// Called when the user clicks empty graph space, clearing selection.
    #[props(default)]
    pub on_deselect: Option<EventHandler<()>>,
    /// Currently selected node id. Used to emphasize incident edges.
    #[props(default)]
    pub selected_node_id: Option<String>,
    /// Currently hovered node id. Used for transient edge emphasis.
    #[props(default)]
    pub hovered_node_id: Option<String>,
    /// When enabled, selecting a node applies a temporary focused layout
    /// transform that repositions surrounding nodes around the selection.
    #[props(default)]
    pub selection_auto_layout: bool,
    /// When enabled, a selected node also animates the camera target so the
    /// active node recenters without remounting the graph.
    #[props(default)]
    pub selection_auto_focus: bool,
    /// Optional per-frame camera-relative transform applied inside the shared
    /// renderer after layout animation but before projection.
    #[props(default)]
    pub node_view_transform: NodeViewTransform,
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
    /// CSS-pixel insets applied to the interactive viewport inside the
    /// container: [left, top, right, bottom]. This keeps the graph centred in
    /// the visible region when overlay panels occlude part of the container.
    #[props(default = [0.0; 4])]
    pub viewport_insets: [f32; 4],
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
    let theme_store = use_context::<ThemeStore>();
    let graph_theme = theme_store.graph_theme();
    let style =
        graph_container_style(&props.container_style, false, &graph_theme);
    let edge_count = props.layout.edges.len();
    let edge_overlay_style = "position:absolute; inset:0; width:100%; height:100%; overflow:visible; pointer-events:none; z-index:1; mix-blend-mode:var(--graph-edge-blend-mode); isolation:isolate; background:transparent;";
    rsx! {
        div { id: "{props.container_id}", style: "{style}",
            svg {
                class: "graph-edge-overlay",
                style: "{edge_overlay_style}",
                "data-graph-edge-overlay": "true",
                "preserveAspectRatio": "none",
                for index in 0..edge_count {
                    path {
                        "data-edge-idx": "{index}",
                    }
                }
            }
            div {
                style: "position: absolute; inset: 0; z-index: 0; pointer-events: none;",
                {props.children}
            }
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
    let theme_store = use_context::<ThemeStore>();
    let graph_theme = theme_store.graph_theme();
    let layout = props.layout.clone();
    let edge_count = layout.edges.len();
    let container_id = props.container_id.clone();
    let initial_camera = props.initial_camera.clone();
    let initial_camera_for_bootstrap = initial_camera.clone();
    let edge_overlay_style = "position:absolute; inset:0; width:100%; height:100%; overflow:visible; pointer-events:none; z-index:1; mix-blend-mode:var(--graph-edge-blend-mode); isolation:isolate; background:transparent;";
    let on_layout_change = props.on_layout_change.clone();
    let on_camera_change = props.on_camera_change.clone();
    let on_deselect = props.on_deselect.clone();
    let camera_mode = props.camera_mode;
    let selected_node_id = props.selected_node_id.clone();
    let hovered_node_id = props.hovered_node_id.clone();
    let selection_auto_layout = props.selection_auto_layout;
    let selection_auto_focus = props.selection_auto_focus;
    let projection = props.projection;
    let node_view_transform = props.node_view_transform;
    let layout_mode = props.layout_mode;
    let on_layout_mode_change = props.on_layout_mode_change.clone();
    let on_projection_change = props.on_projection_change.clone();
    let style =
        graph_container_style(&props.container_style, true, &graph_theme);
    let viewport_insets = props.viewport_insets;
    let mut last_layout_mode: Signal<LayoutMode> =
        use_hook(|| Signal::new(props.layout_mode));
    let layout_mode_changed = *last_layout_mode.peek() != props.layout_mode;
    if layout_mode_changed {
        last_layout_mode.set(props.layout_mode);
    }

    let status: Signal<String> =
        use_signal(|| "Initialising WebGPU\u{2026}".to_string());
    let listeners: Signal<Vec<EventListener>> = use_signal(Vec::new);
    let render_rc: Signal<Option<Rc<RefCell<RenderState>>>> =
        use_signal(|| None);
    let frame_handle: Signal<Option<Rc<FrameCallbackHandle>>> =
        use_signal(|| None);

    use_effect(move || {
        let layout = layout.clone();
        let container_id = container_id.clone();
        let initial_camera = initial_camera_for_bootstrap.clone();
        let selected_node_id = selected_node_id.clone();
        let hovered_node_id = hovered_node_id.clone();
        let graph_theme = graph_theme;
        start_graph_bootstrap(
            layout,
            container_id,
            initial_camera,
            camera_mode,
            selected_node_id,
            hovered_node_id,
            graph_theme,
            selection_auto_layout,
            node_view_transform,
            viewport_insets,
            on_layout_change,
            on_camera_change,
            on_deselect,
            projection,
            status,
            render_rc,
            listeners,
            frame_handle,
        );
    });

    let status_text = status.read().clone();

    sync_render_state(
        &render_rc,
        &props.layout,
        props.camera_mode,
        props.projection,
        &props.selected_node_id,
        &props.hovered_node_id,
        &graph_theme,
        props.selection_auto_layout,
        layout_mode_changed,
        initial_camera.as_ref(),
        props.selection_auto_focus,
        props.node_view_transform,
        props.viewport_insets,
        props.on_camera_change.as_ref(),
    );

    let mut last_cam_seq: Signal<u64> = use_hook(|| Signal::new(0));
    apply_camera_command_update(
        &mut last_cam_seq,
        &props,
        &render_rc,
        props.on_camera_change.as_ref(),
    );

    rsx! {
        div {
            id: "{props.container_id}",
            style: "{style}",
            svg {
                class: "graph-edge-overlay",
                style: "{edge_overlay_style}",
                "data-graph-edge-overlay": "true",
                "preserveAspectRatio": "none",
                for index in 0..edge_count {
                    path {
                        "data-edge-idx": "{index}",
                    }
                }
            }
            div {
                style: "position: absolute; inset: 0; z-index: 0; pointer-events: none;",
                {props.children}
            }
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

fn graph_container_style(
    container_style: &str,
    interactive: bool,
    graph_theme: &GraphThemeSettings,
) -> String {
    let mut style = if !container_style.is_empty() {
        container_style.to_string()
    } else if interactive {
        "position: absolute; inset: 0; overflow: hidden; user-select: none; cursor: grab;"
            .to_string()
    } else {
        "position: absolute; inset: 0; overflow: hidden;".to_string()
    };

    if !style.trim_end().ends_with(';') {
        style.push(';');
    }
    style.push_str(&graph_theme_css_vars(graph_theme));
    style
}

fn graph_theme_css_vars(graph_theme: &GraphThemeSettings) -> String {
    let muted_text = [
        graph_theme.node_text[0],
        graph_theme.node_text[1],
        graph_theme.node_text[2],
        0.74,
    ];
    format!(
        "--graph-edge-opacity:{:.3};--graph-edge-blend-mode:{};--graph-node-surface:{};--graph-node-border:{};--graph-node-text:{};--graph-node-muted-text:{};--graph-node-shadow:0 16px 34px rgba(0,0,0,{:.3});",
        graph_theme.edge_overlay_opacity,
        graph_theme.edge_blend_mode.css_value(),
        rgba_css(graph_theme.node_surface),
        rgba_css(graph_theme.node_border),
        rgba_css(graph_theme.node_text),
        rgba_css(muted_text),
        graph_theme.node_shadow_alpha,
    )
}

fn rgba_css(color: PaletteColor) -> String {
    format!(
        "rgba({:.0}, {:.0}, {:.0}, {:.3})",
        color[0] * 255.0,
        color[1] * 255.0,
        color[2] * 255.0,
        color[3],
    )
}

#[cfg(target_arch = "wasm32")]
fn start_graph_bootstrap(
    layout: Layout3D,
    container_id: String,
    initial_camera: Option<Camera>,
    camera_mode: CameraMode,
    selected_node_id: Option<String>,
    hovered_node_id: Option<String>,
    graph_theme: GraphThemeSettings,
    selection_auto_layout: bool,
    node_view_transform: NodeViewTransform,
    viewport_insets: [f32; 4],
    on_layout_change: Option<EventHandler<Layout3D>>,
    on_camera_change: Option<EventHandler<Camera>>,
    on_deselect: Option<EventHandler<()>>,
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
                    let _ = window
                        .set_timeout_with_callback_and_timeout_and_arguments_0(
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
            },
        };

        let label = js_sys::Reflect::get(&shared.device, &"label".into())
            .ok()
            .and_then(|value| value.as_string())
            .unwrap_or_default();
        tracing::info!(target: "graph3d::init", device_label = %label, "received shared device");

        let (init_w, init_h) = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("webgpu-canvas"))
            .and_then(|element| {
                element.dyn_into::<web_sys::HtmlCanvasElement>().ok()
            })
            .map(|canvas| (canvas.width().max(1), canvas.height().max(1)))
            .unwrap_or((1, 1));

        let gpu = match init_gpu(device, &shared.format, init_w, init_h) {
            Ok(gpu) => gpu,
            Err(error) => {
                status.set(format!("GPU init failed: {error}"));
                return;
            },
        };

        let base_layout = layout.clone();
        let target_layout = data::apply_selected_node_auto_layout(
            &base_layout,
            selected_node_id.as_deref(),
            selection_auto_layout,
        );

        let (edge_data, edge_count) = target_layout.build_gpu_edge_instances();
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

        let (node_data, node_count) = target_layout.build_node_quads();
        let node_quad_buf = if node_data.is_empty() {
            create_buf(&gpu.device, 16, USAGE_VERTEX | USAGE_COPY_DST)
        } else {
            create_buf_init(&gpu.device, &node_data, USAGE_VERTEX)
        };

        let state_rc = Rc::new(RefCell::new(RenderState {
            gpu,
            base_layout,
            layout: target_layout.clone(),
            target_layout,
            camera,
            camera_goal: None,
            camera_mode,
            edge_buf,
            edge_count,
            node_quad_buf,
            node_count,
            container_id: container_id.clone(),
            dirty_layout: false,
            dirty_edges: false,
            projection,
            graph_theme,
            viewport_insets,
            selection_auto_layout,
            node_view_transform,
            selected_node_id,
            hovered_node_id,
            last_frame_time: None,
            interaction_active: false,
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
            on_deselect,
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
    camera_mode: CameraMode,
    projection: Projection,
    selected_node_id: &Option<String>,
    // Retained for API symmetry with the bootstrap path; hover is now owned
    // imperatively by the interaction handler and intentionally not synced here.
    _hovered_node_id: &Option<String>,
    graph_theme: &GraphThemeSettings,
    selection_auto_layout: bool,
    layout_mode_changed: bool,
    initial_camera: Option<&Camera>,
    selection_auto_focus: bool,
    node_view_transform: NodeViewTransform,
    viewport_insets: [f32; 4],
    on_camera_change: Option<&EventHandler<Camera>>,
) {
    let render_state = render_rc.read();
    let Some(render_state) = render_state.as_ref() else {
        return;
    };
    let Ok(mut state) = render_state.try_borrow_mut() else {
        return;
    };

    let target_layout = data::apply_selected_node_auto_layout(
        layout,
        selected_node_id.as_deref(),
        selection_auto_layout,
    );

    if state.interaction_active {
        if state.base_layout != *layout || state.target_layout != target_layout
        {
            for node in &layout.nodes {
                if let Some(target_node) =
                    state.layout.nodes.iter_mut().find(|n| n.id == node.id)
                {
                    target_node.label = node.label.clone();
                    target_node.state = node.state.clone();
                }
                if let Some(target_node) = state
                    .target_layout
                    .nodes
                    .iter_mut()
                    .find(|n| n.id == node.id)
                {
                    target_node.label = node.label.clone();
                    target_node.state = node.state.clone();
                }
                if let Some(target_node) =
                    state.base_layout.nodes.iter_mut().find(|n| n.id == node.id)
                {
                    target_node.label = node.label.clone();
                    target_node.state = node.state.clone();
                }
            }
            state.selection_auto_layout = selection_auto_layout;
            state.dirty_layout = true;
        }
    } else if state.base_layout != *layout
        || state.selection_auto_layout != selection_auto_layout
        || state.target_layout != target_layout
    {
        state.selection_auto_layout = selection_auto_layout;
        if let Some((preserved_base_layout, preserved_target_layout)) =
            preserve_same_topology_layout(
                &state.base_layout,
                &state.layout,
                layout,
                &target_layout,
                selection_auto_layout,
                layout_mode_changed,
            )
        {
            state.base_layout = preserved_base_layout;
            state.layout = preserved_target_layout.clone();
            state.target_layout = preserved_target_layout;
        } else if data::layout_nodes_match(&state.layout, &target_layout) {
            state.base_layout = layout.clone();
            let mut animated_layout = target_layout.clone();
            for (animated_node, current_node) in animated_layout
                .nodes
                .iter_mut()
                .zip(state.layout.nodes.iter())
            {
                animated_node.x = current_node.x;
                animated_node.y = current_node.y;
                animated_node.z = current_node.z;
            }
            state.layout = animated_layout;
            state.target_layout = target_layout.clone();
        } else {
            state.base_layout = layout.clone();
            state.layout = target_layout.clone();
            state.target_layout = target_layout.clone();
        }
        state.dirty_layout = true;
    }

    if layout_mode_changed {
        let goal = initial_camera.cloned().unwrap_or_else(|| {
            let mut camera = Camera::default();
            let bounds = state.target_layout.bounds();
            camera.frame(bounds.0, bounds.1);
            camera
        });
        state.camera_goal = Some(goal.clone());
        if let Some(handler) = on_camera_change {
            handler.call(goal);
        }
    }

    if state.projection != projection {
        state.projection = projection;
    }

    if state.camera_mode != camera_mode {
        state.camera_mode = camera_mode;
    }

    if state.graph_theme != *graph_theme {
        state.graph_theme = *graph_theme;
        state.dirty_edges = true;
    }

    if state.viewport_insets != viewport_insets {
        state.viewport_insets = viewport_insets;
    }

    if state.node_view_transform != node_view_transform {
        state.node_view_transform = node_view_transform;
        state.dirty_layout = true;
        state.dirty_edges = true;
    }

    let selected_changed = state.selected_node_id != *selected_node_id;
    // NOTE: `hovered_node_id` is owned imperatively by the mousemove
    // interaction handler (`interaction::handlers::update_hover`) so per-pointer
    // hover never routes through Dioxus reactivity. We deliberately do not write
    // it here — doing so would clobber the live hover on every re-render.
    if selected_changed {
        state.selected_node_id = selected_node_id.clone();
        state.dirty_edges = true;
        if selection_auto_focus && selected_changed {
            if let Some(selected_id) = selected_node_id.as_deref() {
                if let Some(node) = state
                    .target_layout
                    .nodes
                    .iter()
                    .find(|node| node.id == selected_id)
                {
                    let goal = selection_focus_goal(
                        &state.camera,
                        [node.x, node.y, node.z],
                    );
                    state.camera_goal = Some(goal.clone());
                    if let Some(handler) = on_camera_change {
                        handler.call(goal);
                    }
                }
            }
        }
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

    let bounds = state.target_layout.bounds();
    let mut goal = state.camera.clone();
    goal.apply_command_for_mode(command, state.camera_mode, bounds);
    state.camera_goal = Some(goal.clone());
    if let Some(handler) = on_camera_change {
        handler.call(goal);
    }
}

fn selection_focus_goal(
    current_camera: &Camera,
    target: [f32; 3],
) -> Camera {
    let mut goal = current_camera.clone();
    goal.target = target;
    goal
}

fn preserve_same_topology_layout(
    current_base_layout: &Layout3D,
    current_visible_layout: &Layout3D,
    incoming_base_layout: &Layout3D,
    incoming_target_layout: &Layout3D,
    selection_auto_layout: bool,
    layout_mode_changed: bool,
) -> Option<(Layout3D, Layout3D)> {
    if selection_auto_layout || layout_mode_changed {
        return None;
    }
    if !layout_topology_matches(current_base_layout, incoming_base_layout)
        || !layout_topology_matches(current_base_layout, incoming_target_layout)
    {
        return None;
    }

    let mut preserved_base_layout = incoming_base_layout.clone();
    let mut preserved_target_layout = incoming_target_layout.clone();
    copy_node_positions(current_visible_layout, &mut preserved_base_layout);
    copy_node_positions(current_visible_layout, &mut preserved_target_layout);
    Some((preserved_base_layout, preserved_target_layout))
}

fn copy_node_positions(
    source_layout: &Layout3D,
    target_layout: &mut Layout3D,
) {
    let positions = source_layout
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), [node.x, node.y, node.z]))
        .collect::<HashMap<_, _>>();

    for node in &mut target_layout.nodes {
        if let Some(position) = positions.get(node.id.as_str()) {
            node.x = position[0];
            node.y = position[1];
            node.z = position[2];
        }
    }
}

fn layout_topology_matches(
    left: &Layout3D,
    right: &Layout3D,
) -> bool {
    left.node_card_profile == right.node_card_profile
        && left.nodes.len() == right.nodes.len()
        && left.edges.len() == right.edges.len()
        && sorted_node_ids(left) == sorted_node_ids(right)
        && edge_signatures(left) == edge_signatures(right)
}

fn sorted_node_ids(layout: &Layout3D) -> Vec<String> {
    let mut node_ids = layout
        .nodes
        .iter()
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    node_ids.sort();
    node_ids
}

fn edge_signatures(layout: &Layout3D) -> Option<Vec<(String, String, String)>> {
    let mut signatures = Vec::with_capacity(layout.edges.len());
    for edge in &layout.edges {
        let from_id = layout.nodes.get(edge.from_idx)?.id.clone();
        let to_id = layout.nodes.get(edge.to_idx)?.id.clone();
        signatures.push((from_id, to_id, edge.kind.clone()));
    }
    signatures.sort();
    Some(signatures)
}

#[cfg(test)]
mod tests {
    use super::{
        preserve_same_topology_layout,
        selection_focus_goal,
    };
    use crate::graph3d::{
        Camera,
        EdgeRef3D,
        Layout3D,
        Node3D,
        NodeCardProfile,
    };

    fn sample_layout() -> Layout3D {
        Layout3D {
            nodes: vec![
                Node3D {
                    id: "root".into(),
                    label: Some("Root".into()),
                    state: Some("ready".into()),
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                Node3D {
                    id: "child".into(),
                    label: Some("Child".into()),
                    state: Some("new".into()),
                    x: 12.0,
                    y: -10.0,
                    z: 0.0,
                },
            ],
            edges: vec![EdgeRef3D {
                from_idx: 0,
                to_idx: 1,
                kind: "depends_on".into(),
            }],
            node_card_profile: NodeCardProfile::TicketWide,
        }
    }

    #[test]
    fn preserve_same_topology_layout_keeps_dragged_positions() {
        let current_base_layout = sample_layout();
        let mut current_visible_layout = current_base_layout.clone();
        current_visible_layout.nodes[0].x = 18.0;
        current_visible_layout.nodes[0].y = -6.0;
        current_visible_layout.nodes[1].x = 28.0;
        current_visible_layout.nodes[1].y = -18.0;

        let mut incoming_base_layout = sample_layout();
        incoming_base_layout.nodes[0].label = Some("Updated Root".into());
        let incoming_target_layout = incoming_base_layout.clone();

        let Some((preserved_base_layout, preserved_target_layout)) =
            preserve_same_topology_layout(
                &current_base_layout,
                &current_visible_layout,
                &incoming_base_layout,
                &incoming_target_layout,
                false,
                false,
            )
        else {
            panic!(
                "expected same-topology layout refresh to preserve positions"
            );
        };

        assert_eq!(preserved_base_layout.nodes[0].x, 18.0);
        assert_eq!(preserved_base_layout.nodes[0].y, -6.0);
        assert_eq!(preserved_target_layout.nodes[1].x, 28.0);
        assert_eq!(preserved_target_layout.nodes[1].y, -18.0);
        assert_eq!(
            preserved_base_layout.nodes[0].label.as_deref(),
            Some("Updated Root")
        );
    }

    #[test]
    fn preserve_same_topology_layout_skips_layout_mode_changes() {
        let layout = sample_layout();
        assert!(preserve_same_topology_layout(
            &layout, &layout, &layout, &layout, false, true,
        )
        .is_none());
    }

    #[test]
    fn selection_focus_goal_preserves_distance() {
        let camera = Camera {
            yaw: 0.7,
            pitch: 0.45,
            distance: 41.5,
            target: [1.0, 2.0, 3.0],
        };

        let goal = selection_focus_goal(&camera, [8.0, -4.0, 12.0]);

        assert_eq!(goal.distance, camera.distance);
        assert_eq!(goal.target, [8.0, -4.0, 12.0]);
        assert_eq!(goal.yaw, camera.yaw);
        assert_eq!(goal.pitch, camera.pitch);
    }
}
