//! Per-frame render: GPU pass + CSS-3D DOM node positioning.

#![cfg(target_arch = "wasm32")]

use js_sys::{
    Array,
    Function,
    Reflect,
};
use wasm_bindgen::{
    prelude::*,
    JsCast,
};
use web_sys::{
    Element,
    HtmlElement,
};

use super::{
    camera::{
        animate_camera,
        Camera,
        CameraMode,
        Projection,
        CAMERA_FAR,
        CAMERA_FOV,
        CAMERA_NEAR,
        CAM_UNIFORM_FLOATS,
    },
    data::{
        apply_node_view_transform,
        animate_layout_nodes,
        edge_color,
        EdgeRef3D,
        EdgeVisualState,
        Layout3D,
        NodeCardProfile,
        NodeViewTransform,
        EDGE_FLAG_SELECTED,
    },
    gpu::GpuResources,
    interop::*,
    math,
    theme::GraphThemeSettings,
};

const CAMERA_ANIMATION_SPEED: f32 = 6.0;
const LAYOUT_ANIMATION_SPEED: f32 = 12.0;

pub(crate) struct RenderState {
    pub gpu: GpuResources,
    pub base_layout: Layout3D,
    pub layout: Layout3D,
    pub target_layout: Layout3D,
    pub camera: Camera,
    pub camera_goal: Option<Camera>,
    pub camera_mode: CameraMode,
    pub edge_buf: web_sys::GpuBuffer,
    pub edge_count: u32,
    pub node_quad_buf: web_sys::GpuBuffer,
    pub node_count: u32,
    /// CSS id of the DOM container that hosts the node cards. Used to
    /// translate world-space projections into container-local pixels.
    pub container_id: String,
    /// Set by the drag interaction when a node has moved this frame; the
    /// renderer will rewrite `edge_buf` + `node_quad_buf` from `layout`
    /// before drawing.
    pub dirty_layout: bool,
    /// Set when focus-only state changes and only edge instances need a rewrite.
    pub dirty_edges: bool,
    /// Camera projection mode (perspective or orthographic).
    pub projection: Projection,
    pub graph_theme: GraphThemeSettings,
    pub viewport_insets: [f32; 4],
    pub selection_auto_layout: bool,
    pub node_view_transform: NodeViewTransform,
    pub selected_node_id: Option<String>,
    pub hovered_node_id: Option<String>,
    pub last_frame_time: Option<f32>,
}

struct ScreenPos {
    x: f32,
    y: f32,
    z: f32,
    visible: bool,
}

#[derive(Clone, Copy)]
struct NodeScreenRect {
    center_x: f32,
    center_y: f32,
    half_w: f32,
    half_h: f32,
}

fn resolve_viewport_rect(
    container_w: f32,
    container_h: f32,
    viewport_insets: [f32; 4],
) -> (f32, f32, f32, f32) {
    let left = viewport_insets[0].max(0.0);
    let top = viewport_insets[1].max(0.0);
    let right = viewport_insets[2].max(0.0);
    let bottom = viewport_insets[3].max(0.0);
    let width = (container_w - left - right).max(1.0);
    let height = (container_h - top - bottom).max(1.0);

    let clamped_left = left.min((container_w - width).max(0.0));
    let clamped_top = top.min((container_h - height).max(0.0));
    (clamped_left, clamped_top, width, height)
}

fn world_to_screen(
    pos: [f32; 3],
    vp: &[f32; 16],
    vw: f32,
    vh: f32,
) -> ScreenPos {
    let x = vp[0] * pos[0] + vp[4] * pos[1] + vp[8] * pos[2] + vp[12];
    let y = vp[1] * pos[0] + vp[5] * pos[1] + vp[9] * pos[2] + vp[13];
    let z = vp[2] * pos[0] + vp[6] * pos[1] + vp[10] * pos[2] + vp[14];
    let w = vp[3] * pos[0] + vp[7] * pos[1] + vp[11] * pos[2] + vp[15];
    if w <= 0.001 {
        return ScreenPos {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            visible: false,
        };
    }
    let ndc_x = x / w;
    let ndc_y = y / w;
    let ndc_z = z / w;
    let sx = (ndc_x + 1.0) * 0.5 * vw;
    let sy = (1.0 - ndc_y) * 0.5 * vh;
    ScreenPos {
        x: sx,
        y: sy,
        z: ndc_z,
        visible: ndc_z >= 0.0 && ndc_z <= 1.0,
    }
}

fn collect_dom_node_rects(
    doc: &web_sys::Document,
    state: &RenderState,
    layout: &Layout3D,
) -> Vec<Option<NodeScreenRect>> {
    let mut rects = vec![None; layout.nodes.len()];
    let Some(container) = doc.get_element_by_id(&state.container_id) else {
        return rects;
    };
    let Ok(container) = container.dyn_into::<HtmlElement>() else {
        return rects;
    };
    let container_rect = container.get_bounding_client_rect();
    let Ok(node_list) = doc.query_selector_all(&format!(
        "#{} [data-node-idx]",
        state.container_id
    )) else {
        return rects;
    };

    for i in 0..node_list.length() {
        let Some(node) = node_list.item(i) else {
            continue;
        };
        let Ok(html_el) = node.dyn_into::<HtmlElement>() else {
            continue;
        };
        let Some(idx_str) = html_el.get_attribute("data-node-idx") else {
            continue;
        };
        let Ok(idx) = idx_str.parse::<usize>() else {
            continue;
        };
        let Some(slot) = rects.get_mut(idx) else {
            continue;
        };
        if html_el
            .style()
            .get_property_value("display")
            .ok()
            .as_deref()
            == Some("none")
        {
            continue;
        }

        let rect = html_el.get_bounding_client_rect();
        *slot = Some(NodeScreenRect {
            center_x: (rect.left() - container_rect.left() + rect.width() * 0.5)
                as f32,
            center_y: (rect.top() - container_rect.top() + rect.height() * 0.5)
                as f32,
            half_w: (rect.width() * 0.5) as f32,
            half_h: (rect.height() * 0.5) as f32,
        });
    }

    rects
}

fn clip_edge_endpoint(
    rect: NodeScreenRect,
    toward: (f32, f32),
    padding: f32,
) -> (f32, f32) {
    let dx = toward.0 - rect.center_x;
    let dy = toward.1 - rect.center_y;
    if dx.abs() < 0.001 && dy.abs() < 0.001 {
        return (rect.center_x, rect.center_y);
    }

    let tx = if dx.abs() > 0.001 {
        (rect.half_w + padding) / dx.abs()
    } else {
        f32::INFINITY
    };
    let ty = if dy.abs() > 0.001 {
        (rect.half_h + padding) / dy.abs()
    } else {
        f32::INFINITY
    };
    let t = tx.min(ty).max(0.0);

    (rect.center_x + dx * t, rect.center_y + dy * t)
}

fn position_dom_nodes(
    state: &RenderState,
    layout: &Layout3D,
    viewport_x: f32,
    viewport_y: f32,
    viewport_w: f32,
    viewport_h: f32,
) {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };

    // Use the container's physical dimensions for the projection.  Since the
    // camera's viewProj was computed with the same aspect ratio as the
    // container (and setViewport maps NDC → container region), NDC → pixels
    // is a simple [0, cont_w] × [0, cont_h] transform with no origin offset.
    let eye = state.camera.eye();
    let aspect = viewport_w / viewport_h.max(1.0);
    let proj = match state.projection {
        Projection::Perspective =>
            math::perspective(CAMERA_FOV, aspect, CAMERA_NEAR, CAMERA_FAR),
        Projection::Orthographic => {
            let half_h = state.camera.distance * (CAMERA_FOV * 0.5).tan();
            math::orthographic(half_h, aspect, CAMERA_NEAR, CAMERA_FAR)
        },
    };
    let view = math::look_at(eye, state.camera.target, [0.0, 1.0, 0.0]);
    let vp = math::mul(proj, view);

    let Ok(node_list) = doc.query_selector_all(&format!(
        "#{} [data-node-idx]",
        state.container_id
    )) else {
        return;
    };
    for i in 0..node_list.length() {
        let Some(el) = node_list.item(i) else {
            continue;
        };
        let Ok(html_el) = el.dyn_into::<HtmlElement>() else {
            continue;
        };
        let Some(idx_str) = html_el.get_attribute("data-node-idx") else {
            continue;
        };
        let Ok(idx) = idx_str.parse::<usize>() else {
            continue;
        };
        let Some(node) = layout.nodes.get(idx) else {
            continue;
        };

        let screen = world_to_screen(
            [node.x, node.y, node.z],
            &vp,
            viewport_w,
            viewport_h,
        );

        let dx = eye[0] - node.x;
        let dy = eye[1] - node.y;
        let dz = eye[2] - node.z;
        let dist = (dx * dx + dy * dy + dz * dz).sqrt().max(0.1);
        let pixel_scale = (22.0 / dist).clamp(0.14, 3.5);

        let margin = 300.0;
        if !screen.visible
            || screen.x < -margin
            || screen.x > viewport_w + margin
            || screen.y < -margin
            || screen.y > viewport_h + margin
            || pixel_scale < 0.08
        {
            let _ = html_el.style().set_property("display", "none");
            continue;
        }

        // screen.x / screen.y are already container-local (no origin offset
        // needed) because the projection uses container dimensions.
        let local_x = viewport_x + screen.x;
        let local_y = viewport_y + screen.y;

        // Use explicit "block" instead of "" — when callers style cards via
        // CSS classes (e.g. `.content { display: none }`), removing the
        // inline override falls back to the CSS rule and the card stays
        // hidden. "block" wins as an inline override regardless.
        let _ = html_el.style().set_property("display", "block");
        // Selected nodes (class "node-card-selected") get a very high z-index
        // so they always render in the foreground over overlapping neighbours.
        let is_selected = html_el
            .get_attribute("class")
            .map(|c| c.contains("node-card-selected"))
            .unwrap_or(false);
        let z_idx = if is_selected {
            100_000i32
        } else {
            ((1.0 - screen.z) * 10000.0) as i32
        };
        let _ = html_el.style().set_property("z-index", &z_idx.to_string());

        let transform = format!(
            "translate(-50%, -50%) translate({:.1}px, {:.1}px) scale({:.3})",
            local_x, local_y, pixel_scale,
        );
        let _ = html_el.style().set_property("transform", &transform);
    }
}

fn position_dom_edges(
    state: &RenderState,
    layout: &Layout3D,
    viewport_x: f32,
    viewport_y: f32,
    viewport_w: f32,
    viewport_h: f32,
) {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Some(container) = doc.get_element_by_id(&state.container_id) else {
        return;
    };
    let Ok(container) = container.dyn_into::<HtmlElement>() else {
        return;
    };
    let Some(svg) = doc
        .query_selector(&format!(
            "#{} [data-graph-edge-overlay='true']",
            state.container_id
        ))
        .ok()
        .flatten()
    else {
        return;
    };

    let container_rect = container.get_bounding_client_rect();
    let container_w = (container_rect.width() as f32).max(1.0);
    let container_h = (container_rect.height() as f32).max(1.0);

    let _ = svg.set_attribute("display", "block");
    let _ = svg.set_attribute("width", &format!("{container_w:.1}"));
    let _ = svg.set_attribute("height", &format!("{container_h:.1}"));
    let _ = svg.set_attribute(
        "viewBox",
        &format!("0 0 {container_w:.1} {container_h:.1}"),
    );

    let eye = state.camera.eye();
    let aspect = viewport_w / viewport_h.max(1.0);
    let proj = match state.projection {
        Projection::Perspective =>
            math::perspective(CAMERA_FOV, aspect, CAMERA_NEAR, CAMERA_FAR),
        Projection::Orthographic => {
            let half_h = state.camera.distance * (CAMERA_FOV * 0.5).tan();
            math::orthographic(half_h, aspect, CAMERA_NEAR, CAMERA_FAR)
        },
    };
    let view = math::look_at(eye, state.camera.target, [0.0, 1.0, 0.0]);
    let vp = math::mul(proj, view);
    let node_rects = collect_dom_node_rects(&doc, state, layout);
    let active_focus = edge_visual_state(state).active_focus(&layout.nodes);

    let Ok(edge_nodes) = doc.query_selector_all(&format!(
        "#{} [data-edge-idx]",
        state.container_id
    )) else {
        return;
    };

    for index in 0..edge_nodes.length() {
        let Some(line_node) = edge_nodes.item(index) else {
            continue;
        };
        let Ok(line) = line_node.dyn_into::<Element>() else {
            continue;
        };
        let Some(edge) = layout.edges.get(index as usize) else {
            let _ = line.set_attribute("display", "none");
            continue;
        };
        let Some(a) = layout.nodes.get(edge.from_idx) else {
            let _ = line.set_attribute("display", "none");
            continue;
        };
        let Some(b) = layout.nodes.get(edge.to_idx) else {
            let _ = line.set_attribute("display", "none");
            continue;
        };

        let screen_a =
            world_to_screen([a.x, a.y, a.z], &vp, viewport_w, viewport_h);
        let screen_b =
            world_to_screen([b.x, b.y, b.z], &vp, viewport_w, viewport_h);
        if (!screen_a.visible && !screen_b.visible)
            || (screen_a.x - screen_b.x).abs() < 1.0
                && (screen_a.y - screen_b.y).abs() < 1.0
        {
            let _ = line.set_attribute("display", "none");
            continue;
        }

        let rect_a = node_rects.get(edge.from_idx).copied().flatten();
        let rect_b = node_rects.get(edge.to_idx).copied().flatten();
        let center_a = rect_a
            .map(|rect| (rect.center_x, rect.center_y))
            .unwrap_or((viewport_x + screen_a.x, viewport_y + screen_a.y));
        let center_b = rect_b
            .map(|rect| (rect.center_x, rect.center_y))
            .unwrap_or((viewport_x + screen_b.x, viewport_y + screen_b.y));
        let (x1, y1) = rect_a
            .map(|rect| clip_edge_endpoint(rect, center_b, 0.0))
            .unwrap_or(center_a);
        let (x2, y2) = rect_b
            .map(|rect| clip_edge_endpoint(rect, center_a, 0.0))
            .unwrap_or(center_b);
        if (x1 - x2).abs() < 3.0 && (y1 - y2).abs() < 3.0 {
            let _ = line.set_attribute("display", "none");
            continue;
        }

        let (stroke_width, stroke_opacity) = edge_overlay_style(
            layout,
            active_focus,
            edge,
            a.id.as_str(),
            b.id.as_str(),
        );
        let (r, g, blue, _alpha) = edge_color(&edge.kind, &state.graph_theme);
        let stroke_alpha = (stroke_opacity
            * state.graph_theme.edge_overlay_opacity)
            .clamp(0.0, 1.0);
        let _ = line.set_attribute("display", "block");
        let _ = line.set_attribute("x1", &format!("{x1:.1}"));
        let _ = line.set_attribute("y1", &format!("{y1:.1}"));
        let _ = line.set_attribute("x2", &format!("{x2:.1}"));
        let _ = line.set_attribute("y2", &format!("{y2:.1}"));
        let _ = line.set_attribute(
            "stroke",
            &format!(
                "rgb({:.0} {:.0} {:.0} / {:.3})",
                r * 255.0,
                g * 255.0,
                blue * 255.0,
                stroke_alpha,
            ),
        );
        let _ = line.remove_attribute("stroke-opacity");
        let _ = line.remove_attribute("opacity");
        let _ =
            line.set_attribute("stroke-width", &format!("{stroke_width:.2}"));
        let _ = line.set_attribute("vector-effect", "non-scaling-stroke");
    }
}

fn edge_overlay_style(
    layout: &Layout3D,
    active_focus: Option<(&str, f32)>,
    edge: &EdgeRef3D,
    from_id: &str,
    to_id: &str,
) -> (f32, f32) {
    if let Some((focus_id, focus_flag)) = active_focus {
        if focus_id == from_id || focus_id == to_id {
            if focus_flag == EDGE_FLAG_SELECTED {
                return (3.2, 0.98);
            }
            return (2.8, 0.90);
        }
        return (1.3, 0.26);
    }

    let long_edge = if let (Some(a), Some(b)) = (
        layout.nodes.get(edge.from_idx),
        layout.nodes.get(edge.to_idx),
    ) {
        let dx = a.x - b.x;
        let dy = a.y - b.y;
        let dz = a.z - b.z;
        (dx * dx + dy * dy + dz * dz).sqrt() > 3.5
    } else {
        false
    };
    let (long_width, long_opacity, short_width, short_opacity) =
        match layout.node_card_profile {
            NodeCardProfile::Compact => (2.4, 0.82, 1.9, 0.64),
            NodeCardProfile::TicketWide => (2.6, 0.78, 2.1, 0.60),
        };
    if long_edge {
        (long_width, long_opacity)
    } else {
        (short_width, short_opacity)
    }
}

pub(crate) fn render_frame(
    state: &mut RenderState,
    frame: &crate::effects::FrameContext,
) {
    let dt = state
        .last_frame_time
        .map(|last| (frame.time_s - last).clamp(0.0, 0.1))
        .unwrap_or(1.0 / 60.0);
    state.last_frame_time = Some(frame.time_s);

    if let Some(goal) = state.camera_goal.clone() {
        if !animate_camera(&mut state.camera, &goal, dt, CAMERA_ANIMATION_SPEED)
        {
            state.camera_goal = None;
        }
    }
    if animate_layout_nodes(
        &mut state.layout,
        &state.target_layout,
        dt,
        LAYOUT_ANIMATION_SPEED,
    ) {
        state.dirty_layout = true;
    }

    // The overlay-driven loop already resized the canvas backing store and
    // hands us the current frame's swap-chain view. We only need to make
    // sure our depth texture matches the new size; CSS pixel size is
    // recomputed from `frame.canvas_w/h` divided by DPR for DOM positioning.
    let dpr = web_sys::window()
        .map(|w| w.device_pixel_ratio().clamp(1.0, 4.0))
        .unwrap_or(1.0) as f32;
    let css_w = (frame.canvas_w as f32) / dpr;
    let css_h = (frame.canvas_h as f32) / dpr;
    if frame.canvas_w != state.gpu.canvas_w
        || frame.canvas_h != state.gpu.canvas_h
    {
        state.gpu.depth_view = create_depth_view(
            &state.gpu.device,
            frame.canvas_w,
            frame.canvas_h,
        );
        state.gpu.canvas_w = frame.canvas_w;
        state.gpu.canvas_h = frame.canvas_h;
    }

    // Resolve the graph container's bounding rect so the camera and DOM
    // projection are both centred on the container, not the full canvas.
    let (cont_x_css, cont_y_css, cont_w_css, cont_h_css) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(&state.container_id))
        .map(|el| {
            let r = el.get_bounding_client_rect();
            (
                r.left() as f32,
                r.top() as f32,
                r.width() as f32,
                r.height() as f32,
            )
        })
        .unwrap_or((0.0, 0.0, css_w, css_h));
    let (viewport_x_css, viewport_y_css, viewport_w_css, viewport_h_css) =
        resolve_viewport_rect(cont_w_css, cont_h_css, state.viewport_insets);
    // Physical-pixel viewport coordinates for setViewport / setScissorRect.
    let vp_x = ((cont_x_css + viewport_x_css) * dpr).round() as u32;
    let vp_y = ((cont_y_css + viewport_y_css) * dpr).round() as u32;
    let vp_w = ((viewport_w_css * dpr).round() as u32).max(1);
    let vp_h = ((viewport_h_css * dpr).round() as u32).max(1);
    let render_layout = current_render_layout(state, viewport_w_css, viewport_h_css);
    let uses_dynamic_layout = render_layout.is_some();

    // Re-upload per-instance buffers if a node moved this frame.
    if uses_dynamic_layout || state.dirty_layout || state.dirty_edges {
        let render_layout = render_layout.as_ref().unwrap_or(&state.layout);
        let (edge_data, edge_count) = render_layout.build_gpu_edge_instances();
        if !edge_data.is_empty() {
            write_buffer(&state.gpu.device, &state.edge_buf, &edge_data);
        }
        state.edge_count = edge_count;
        state.dirty_edges = false;
    }

    if uses_dynamic_layout || state.dirty_layout {
        let render_layout = render_layout.as_ref().unwrap_or(&state.layout);
        let (node_data, node_count) = render_layout.build_node_quads();
        if !node_data.is_empty() {
            write_buffer(&state.gpu.device, &state.node_quad_buf, &node_data);
        }
        state.node_count = node_count;
        state.dirty_layout = false;
    }

    let gpu = &state.gpu;

    // Camera uniform — use container aspect ratio so the projection centres
    // the graph on the container, not the full canvas.
    let eye = state.camera.eye();
    let aspect = viewport_w_css / viewport_h_css.max(1.0);
    let proj = match state.projection {
        Projection::Perspective =>
            math::perspective(CAMERA_FOV, aspect, CAMERA_NEAR, CAMERA_FAR),
        Projection::Orthographic => {
            let half_h = state.camera.distance * (CAMERA_FOV * 0.5).tan();
            math::orthographic(half_h, aspect, CAMERA_NEAR, CAMERA_FAR)
        },
    };
    let view = math::look_at(eye, state.camera.target, [0.0, 1.0, 0.0]);
    let vp_mat = math::mul(proj, view);

    let time = web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now() as f32 / 1000.0)
        .unwrap_or(0.0);

    let mut cam_data = [0.0f32; CAM_UNIFORM_FLOATS];
    cam_data[..16].copy_from_slice(&vp_mat);
    cam_data[16] = eye[0];
    cam_data[17] = eye[1];
    cam_data[18] = eye[2];
    cam_data[19] = 1.0;
    cam_data[20] = time;
    cam_data[21] = vp_w as f32;
    cam_data[22] = vp_h as f32;
    write_buffer(&gpu.device, &gpu.cam_buf, &cam_data);

    // Use the swap-chain view supplied by the overlay's per-frame callback.
    // `loadOp: "load"` preserves whatever the overlay's smoke / particle
    // pass already drew underneath, so the graph composites on top.
    let tex_view = frame.frame_view.clone();

    // Render pass descriptor (colour LOADs the existing overlay frame, depth
    // is cleared because we own it exclusively).
    let color_att = obj();
    set(&color_att, "view", &tex_view);
    set(&color_att, "loadOp", &js_str("load"));
    set(&color_att, "storeOp", &js_str("store"));
    let color_atts = Array::new();
    color_atts.push(&JsValue::from(color_att));
    let rp_desc = obj();
    set(&rp_desc, "colorAttachments", &JsValue::from(color_atts));

    let ds_att = obj();
    set(&ds_att, "view", &gpu.depth_view);
    set(&ds_att, "depthClearValue", &js_f64(1.0));
    set(&ds_att, "depthLoadOp", &js_str("clear"));
    set(&ds_att, "depthStoreOp", &js_str("store"));
    set(&rp_desc, "depthStencilAttachment", &JsValue::from(ds_att));

    let encoder = gpu.device.create_command_encoder();
    let encoder_js: JsValue = encoder.into();
    let pass_desc =
        web_sys::GpuRenderPassDescriptor::from(JsValue::from(rp_desc));
    let enc_typed: web_sys::GpuCommandEncoder =
        encoder_js.clone().dyn_into().unwrap();
    let Ok(pass_enc) = enc_typed.begin_render_pass(&pass_desc) else {
        return;
    };
    let pass: JsValue = JsValue::from(pass_enc);

    // Restrict GPU rendering to the container region so edges and node quads
    // don't bleed into the content panel on the right.
    // setViewport(x, y, width, height, minDepth, maxDepth) — 6 args.
    if let Ok(f) =
        js_sys::Reflect::get(&pass, &super::interop::js_str("setViewport"))
            .and_then(|v| v.dyn_into::<js_sys::Function>())
    {
        let vp_args = Array::new();
        vp_args.push(&js_f64(vp_x as f64));
        vp_args.push(&js_f64(vp_y as f64));
        vp_args.push(&js_f64(vp_w as f64));
        vp_args.push(&js_f64(vp_h as f64));
        vp_args.push(&js_f64(0.0));
        vp_args.push(&js_f64(1.0));
        let _ = f.apply(&pass, &vp_args);
    }
    // setScissorRect(x, y, width, height) — 4 args.
    if let Ok(f) =
        js_sys::Reflect::get(&pass, &super::interop::js_str("setScissorRect"))
            .and_then(|v| v.dyn_into::<js_sys::Function>())
    {
        let sc_args = Array::new();
        sc_args.push(&js_f64(vp_x as f64));
        sc_args.push(&js_f64(vp_y as f64));
        sc_args.push(&js_f64(vp_w as f64));
        sc_args.push(&js_f64(vp_h as f64));
        let _ = f.apply(&pass, &sc_args);
    }

    // Node occluder quads are intentionally skipped: all nodes are on the
    // flat z=0 plane and DOM cards render on top of the GPU canvas anyway,
    // so writing depth causes more edge clipping than it prevents.

    // Edges: depth-test only.
    if state.edge_count > 0 {
        js_call(&pass, "setPipeline", &[&gpu.edge_pipeline.clone().into()]);
        js_call(&pass, "setBindGroup", &[&js_f64(0.0), &gpu.bind_group]);
        js_call(
            &pass,
            "setVertexBuffer",
            &[&js_f64(0.0), &gpu.quad_buf.clone().into()],
        );
        js_call(
            &pass,
            "setVertexBuffer",
            &[&js_f64(1.0), &state.edge_buf.clone().into()],
        );
        js_call(
            &pass,
            "draw",
            &[&js_f64(4.0), &js_f64(state.edge_count as f64)],
        );
    }

    js_call(&pass, "end", &[]);

    let cmd_buf: JsValue = Reflect::get(&encoder_js, &js_str("finish"))
        .and_then(|f| f.dyn_into::<Function>())
        .ok()
        .and_then(|f| f.call0(&encoder_js).ok())
        .unwrap_or(JsValue::UNDEFINED);
    let bufs = Array::new();
    bufs.push(&cmd_buf);
    js_call(frame.queue, "submit", &[&JsValue::from(bufs)]);

    let render_layout = render_layout.as_ref().unwrap_or(&state.layout);

    position_dom_nodes(
        state,
        render_layout,
        viewport_x_css,
        viewport_y_css,
        viewport_w_css,
        viewport_h_css,
    );
    position_dom_edges(
        state,
        render_layout,
        viewport_x_css,
        viewport_y_css,
        viewport_w_css,
        viewport_h_css,
    );
}

fn current_render_layout(
    state: &RenderState,
    viewport_width: f32,
    viewport_height: f32,
) -> Option<Layout3D> {
    if state.node_view_transform.is_active() {
        Some(apply_node_view_transform(
            &state.layout,
            &state.camera,
            viewport_width,
            viewport_height,
            state.node_view_transform,
        ))
    } else {
        None
    }
}

fn edge_visual_state(state: &RenderState) -> EdgeVisualState<'_> {
    EdgeVisualState {
        selected_node_id: state.selected_node_id.as_deref(),
        hovered_node_id: state.hovered_node_id.as_deref(),
    }
}
