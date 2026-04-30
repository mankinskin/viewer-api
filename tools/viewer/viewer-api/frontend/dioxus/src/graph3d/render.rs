//! Per-frame render: GPU pass + CSS-3D DOM node positioning.

#![cfg(target_arch = "wasm32")]

use js_sys::{Array, Function, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use super::camera::{Camera, CAMERA_FAR, CAMERA_FOV, CAMERA_NEAR, CAM_UNIFORM_FLOATS};
use super::data::Layout3D;
use super::gpu::GpuResources;
use super::interop::*;
use super::math;

pub(crate) struct RenderState {
    pub gpu:           GpuResources,
    pub layout:        Layout3D,
    pub camera:        Camera,
    pub edge_buf:      web_sys::GpuBuffer,
    pub edge_count:    u32,
    pub node_quad_buf: web_sys::GpuBuffer,
    pub node_count:    u32,
    /// CSS id of the DOM container that hosts the node cards. Used to
    /// translate world-space projections into container-local pixels.
    pub container_id:  String,
    /// Set by the drag interaction when a node has moved this frame; the
    /// renderer will rewrite `edge_buf` + `node_quad_buf` from `layout`
    /// before drawing.
    pub dirty_layout:  bool,
}

struct ScreenPos { x: f32, y: f32, z: f32, visible: bool }

fn world_to_screen(pos: [f32; 3], vp: &[f32; 16], vw: f32, vh: f32) -> ScreenPos {
    let x = vp[0]*pos[0] + vp[4]*pos[1] + vp[ 8]*pos[2] + vp[12];
    let y = vp[1]*pos[0] + vp[5]*pos[1] + vp[ 9]*pos[2] + vp[13];
    let z = vp[2]*pos[0] + vp[6]*pos[1] + vp[10]*pos[2] + vp[14];
    let w = vp[3]*pos[0] + vp[7]*pos[1] + vp[11]*pos[2] + vp[15];
    if w <= 0.001 {
        return ScreenPos { x: 0.0, y: 0.0, z: 0.0, visible: false };
    }
    let ndc_x = x / w;
    let ndc_y = y / w;
    let ndc_z = z / w;
    let sx = (ndc_x + 1.0) * 0.5 * vw;
    let sy = (1.0 - ndc_y) * 0.5 * vh;
    ScreenPos { x: sx, y: sy, z: ndc_z, visible: ndc_z >= 0.0 && ndc_z <= 1.0 }
}

fn position_dom_nodes(state: &RenderState, vw: f32, vh: f32) {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else { return };

    let (container_left, container_top) = doc
        .get_element_by_id(&state.container_id)
        .map(|el| {
            let rect = el.get_bounding_client_rect();
            (rect.left() as f32, rect.top() as f32)
        })
        .unwrap_or((0.0, 0.0));

    let eye    = state.camera.eye();
    let aspect = vw / vh.max(1.0);
    let proj   = math::perspective(CAMERA_FOV, aspect, CAMERA_NEAR, CAMERA_FAR);
    let view   = math::look_at(eye, state.camera.target, [0.0, 1.0, 0.0]);
    let vp     = math::mul(proj, view);

    let Ok(node_list) = doc.query_selector_all(&format!("#{} [data-node-idx]", state.container_id))
    else { return };
    for i in 0..node_list.length() {
        let Some(el)        = node_list.item(i)              else { continue };
        let Ok(html_el)     = el.dyn_into::<HtmlElement>()    else { continue };
        let Some(idx_str)   = html_el.get_attribute("data-node-idx") else { continue };
        let Ok(idx)         = idx_str.parse::<usize>()        else { continue };
        let Some(node)      = state.layout.nodes.get(idx)     else { continue };

        let screen = world_to_screen([node.x, node.y, node.z], &vp, vw, vh);

        let dx = eye[0] - node.x;
        let dy = eye[1] - node.y;
        let dz = eye[2] - node.z;
        let dist = (dx*dx + dy*dy + dz*dz).sqrt().max(0.1);
        let pixel_scale = (15.0 / dist).clamp(0.15, 2.5);

        let margin = 300.0;
        if !screen.visible
            || screen.x < -margin || screen.x > vw + margin
            || screen.y < -margin || screen.y > vh + margin
            || pixel_scale < 0.08
        {
            let _ = html_el.style().set_property("display", "none");
            continue;
        }

        let local_x = screen.x - container_left;
        let local_y = screen.y - container_top;

        // Use explicit "block" instead of "" — when callers style cards via
        // CSS classes (e.g. `.graph-node-card { display: none }`), removing
        // the inline override falls back to the CSS rule and the card stays
        // hidden. "block" wins as an inline override regardless.
        let _ = html_el.style().set_property("display", "block");
        let z_idx = ((1.0 - screen.z) * 10000.0) as i32;
        let _ = html_el.style().set_property("z-index", &z_idx.to_string());

        let transform = format!(
            "translate(-50%, -50%) translate({:.1}px, {:.1}px) scale({:.3})",
            local_x, local_y, pixel_scale,
        );
        let _ = html_el.style().set_property("transform", &transform);
    }
}

pub(crate) fn render_frame(state: &mut RenderState, frame: &crate::effects::FrameContext) {
    // The overlay-driven loop already resized the canvas backing store and
    // hands us the current frame's swap-chain view. We only need to make
    // sure our depth texture matches the new size; CSS pixel size is
    // recomputed from `frame.canvas_w/h` divided by DPR for DOM positioning.
    let dpr = web_sys::window()
        .map(|w| w.device_pixel_ratio().clamp(1.0, 4.0))
        .unwrap_or(1.0) as f32;
    let css_w = (frame.canvas_w as f32) / dpr;
    let css_h = (frame.canvas_h as f32) / dpr;
    if frame.canvas_w != state.gpu.canvas_w || frame.canvas_h != state.gpu.canvas_h {
        state.gpu.depth_view =
            create_depth_view(&state.gpu.device, frame.canvas_w, frame.canvas_h);
        state.gpu.canvas_w = frame.canvas_w;
        state.gpu.canvas_h = frame.canvas_h;
    }
    // Re-upload per-instance buffers if a node moved this frame.
    if state.dirty_layout {
        let (edge_data, edge_count) = state.layout.build_edge_instances();
        if !edge_data.is_empty() {
            write_buffer(&state.gpu.device, &state.edge_buf, &edge_data);
        }
        state.edge_count = edge_count;
        let (node_data, node_count) = state.layout.build_node_quads();
        if !node_data.is_empty() {
            write_buffer(&state.gpu.device, &state.node_quad_buf, &node_data);
        }
        state.node_count = node_count;
        state.dirty_layout = false;
    }

    let gpu = &state.gpu;
    let w = gpu.canvas_w;
    let h = gpu.canvas_h;

    // Camera uniform.
    let eye    = state.camera.eye();
    let aspect = w as f32 / h.max(1) as f32;
    let proj   = math::perspective(CAMERA_FOV, aspect, CAMERA_NEAR, CAMERA_FAR);
    let view   = math::look_at(eye, state.camera.target, [0.0, 1.0, 0.0]);
    let vp     = math::mul(proj, view);

    let time = web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now() as f32 / 1000.0)
        .unwrap_or(0.0);

    let mut cam_data = [0.0f32; CAM_UNIFORM_FLOATS];
    cam_data[..16].copy_from_slice(&vp);
    cam_data[16] = eye[0]; cam_data[17] = eye[1]; cam_data[18] = eye[2]; cam_data[19] = 1.0;
    cam_data[20] = time;   cam_data[21] = w as f32; cam_data[22] = h as f32;
    write_buffer(&gpu.device, &gpu.cam_buf, &cam_data);

    // Use the swap-chain view supplied by the overlay's per-frame callback.
    // `loadOp: "load"` preserves whatever the overlay's smoke / particle
    // pass already drew underneath, so the graph composites on top.
    let tex_view = frame.frame_view.clone();

    // Render pass descriptor (colour LOADs the existing overlay frame, depth
    // is cleared because we own it exclusively).
    let color_att = obj();
    set(&color_att, "view",    &tex_view);
    set(&color_att, "loadOp",  &js_str("load"));
    set(&color_att, "storeOp", &js_str("store"));
    let color_atts = Array::new(); color_atts.push(&JsValue::from(color_att));
    let rp_desc = obj();
    set(&rp_desc, "colorAttachments", &JsValue::from(color_atts));

    let ds_att = obj();
    set(&ds_att, "view",            &gpu.depth_view);
    set(&ds_att, "depthClearValue", &js_f64(1.0));
    set(&ds_att, "depthLoadOp",     &js_str("clear"));
    set(&ds_att, "depthStoreOp",    &js_str("store"));
    set(&rp_desc, "depthStencilAttachment", &JsValue::from(ds_att));

    let encoder = gpu.device.create_command_encoder();
    let encoder_js: JsValue = encoder.into();
    let pass_desc = web_sys::GpuRenderPassDescriptor::from(JsValue::from(rp_desc));
    let enc_typed: web_sys::GpuCommandEncoder = encoder_js.clone().dyn_into().unwrap();
    let Ok(pass_enc) = enc_typed.begin_render_pass(&pass_desc) else { return };
    let pass: JsValue = JsValue::from(pass_enc);

    // Node occluder quads first (write depth).
    if state.node_count > 0 {
        js_call(&pass, "setPipeline",     &[&gpu.node_quad_pipeline.clone().into()]);
        js_call(&pass, "setBindGroup",    &[&js_f64(0.0), &gpu.bind_group]);
        js_call(&pass, "setVertexBuffer", &[&js_f64(0.0), &gpu.quad_buf.clone().into()]);
        js_call(&pass, "setVertexBuffer", &[&js_f64(1.0), &state.node_quad_buf.clone().into()]);
        js_call(&pass, "draw",            &[&js_f64(4.0), &js_f64(state.node_count as f64)]);
    }

    // Edges: depth-test only.
    if state.edge_count > 0 {
        js_call(&pass, "setPipeline",     &[&gpu.edge_pipeline.clone().into()]);
        js_call(&pass, "setBindGroup",    &[&js_f64(0.0), &gpu.bind_group]);
        js_call(&pass, "setVertexBuffer", &[&js_f64(0.0), &gpu.quad_buf.clone().into()]);
        js_call(&pass, "setVertexBuffer", &[&js_f64(1.0), &state.edge_buf.clone().into()]);
        js_call(&pass, "draw",            &[&js_f64(4.0), &js_f64(state.edge_count as f64)]);
    }

    js_call(&pass, "end", &[]);

    let cmd_buf: JsValue = Reflect::get(&encoder_js, &js_str("finish"))
        .and_then(|f| f.dyn_into::<Function>())
        .ok()
        .and_then(|f| f.call0(&encoder_js).ok())
        .unwrap_or(JsValue::UNDEFINED);
    let bufs = Array::new(); bufs.push(&cmd_buf);
    js_call(frame.queue, "submit", &[&JsValue::from(bufs)]);

    position_dom_nodes(state, css_w, css_h);
}
