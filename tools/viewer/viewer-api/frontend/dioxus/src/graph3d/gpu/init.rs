//! WebGPU initialisation for the graph3d view: device, pipelines, bind
//! group, depth texture.

#![cfg(target_arch = "wasm32")]

use js_sys::{Array, Function, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    GpuBuffer, GpuCanvasContext, GpuDevice, GpuRenderPipeline, HtmlCanvasElement,
};

use super::super::camera::{CAM_UNIFORM_FLOATS, PALETTE_FLOATS};
use super::super::interop::*;

/// Bundle of GPU resources the render loop needs each frame.
pub(crate) struct GpuResources {
    pub device:             GpuDevice,
    pub ctx:                GpuCanvasContext,
    pub edge_pipeline:      GpuRenderPipeline,
    pub node_quad_pipeline: GpuRenderPipeline,
    pub bind_group:         JsValue,
    pub cam_buf:            GpuBuffer,
    pub quad_buf:           GpuBuffer,
    pub depth_view:         JsValue,
    pub canvas_w:           u32,
    pub canvas_h:           u32,
}

const EDGE_SHADER:      &str = include_str!("../shaders/edge.wgsl");
const NODE_QUAD_SHADER: &str = include_str!("../shaders/node_quad.wgsl");

pub(crate) async fn init_gpu(canvas: HtmlCanvasElement) -> Result<GpuResources, String> {
    let nav: JsValue = web_sys::window().unwrap().navigator().into();
    let gpu = Reflect::get(&nav, &js_str("gpu")).map_err(|_| "No navigator.gpu")?;
    if gpu.is_undefined() { return Err("WebGPU not supported".into()); }

    // adapter
    let adapter_promise = Reflect::get(&gpu, &js_str("requestAdapter"))
        .and_then(|f| f.dyn_into::<Function>())
        .map_err(|_| "requestAdapter missing")?
        .call0(&gpu).map_err(|_| "requestAdapter call failed")?;
    let adapter = JsFuture::from(js_sys::Promise::from(adapter_promise))
        .await.map_err(|_| "adapter request failed")?;
    if adapter.is_null() || adapter.is_undefined() { return Err("No GPU adapter".into()); }

    // device
    let device_promise = Reflect::get(&adapter, &js_str("requestDevice"))
        .and_then(|f| f.dyn_into::<Function>())
        .map_err(|_| "requestDevice missing")?
        .call0(&adapter).map_err(|_| "requestDevice call failed")?;
    let device_js = JsFuture::from(js_sys::Promise::from(device_promise))
        .await.map_err(|_| "device request failed")?;
    let device: GpuDevice = device_js.dyn_into().map_err(|_| "device cast failed")?;

    // canvas context
    let format = preferred_format();
    let canvas_js: JsValue = canvas.clone().into();
    let ctx_js = Reflect::get(&canvas_js, &js_str("getContext"))
        .and_then(|f| f.dyn_into::<Function>())
        .map_err(|_| "getContext missing")?
        .call1(&canvas_js, &js_str("webgpu"))
        .map_err(|_| "getContext call failed")?;
    let ctx: GpuCanvasContext = ctx_js.dyn_into().map_err(|_| "ctx cast failed")?;
    let cfg = obj();
    set(&cfg, "device",    &device.clone().into());
    set(&cfg, "format",    &js_str(&format));
    set(&cfg, "alphaMode", &js_str("opaque"));
    js_call(&ctx.clone().into(), "configure", &[&JsValue::from(cfg)]);

    let canvas_w = canvas.width();
    let canvas_h = canvas.height();

    // ── Bind group layout: camera(0) + palette(1) ──
    let bgl_entry0 = obj();
    set(&bgl_entry0, "binding",    &js_f64(0.0));
    set(&bgl_entry0, "visibility", &js_f64(3.0)); // VERTEX|FRAGMENT
    let bt0 = obj(); set(&bt0, "type", &js_str("uniform"));
    set(&bgl_entry0, "buffer", &JsValue::from(bt0));

    let bgl_entry1 = obj();
    set(&bgl_entry1, "binding",    &js_f64(1.0));
    set(&bgl_entry1, "visibility", &js_f64(2.0));
    let bt1 = obj(); set(&bt1, "type", &js_str("uniform"));
    set(&bgl_entry1, "buffer", &JsValue::from(bt1));

    let bgl_entries = Array::new();
    bgl_entries.push(&JsValue::from(bgl_entry0));
    bgl_entries.push(&JsValue::from(bgl_entry1));
    let bgl_desc = obj();
    set(&bgl_desc, "entries", &JsValue::from(bgl_entries));
    let bgl_js = Reflect::get(&device.clone().into(), &js_str("createBindGroupLayout"))
        .and_then(|f| f.dyn_into::<Function>())
        .map_err(|_| "createBindGroupLayout")?
        .call1(&device.clone().into(), &JsValue::from(bgl_desc))
        .map_err(|_| "bgl call")?;

    // pipeline layout
    let pl_bgls = Array::new();
    pl_bgls.push(&bgl_js);
    let pl_desc = obj();
    set(&pl_desc, "bindGroupLayouts", &JsValue::from(pl_bgls));
    let pipeline_layout = Reflect::get(&device.clone().into(), &js_str("createPipelineLayout"))
        .and_then(|f| f.dyn_into::<Function>())
        .map_err(|_| "createPipelineLayout")?
        .call1(&device.clone().into(), &JsValue::from(pl_desc))
        .map_err(|_| "pl call")?;

    let edge_pipeline      = build_edge_pipeline(&device, &pipeline_layout, &format)?;
    let node_quad_pipeline = build_node_quad_pipeline(&device, &pipeline_layout, &format)?;

    // uniform buffers
    let cam_buf     = create_buf(&device, CAM_UNIFORM_FLOATS * 4, USAGE_UNIFORM | USAGE_COPY_DST);
    let palette_buf = create_buf(&device, PALETTE_FLOATS * 4,     USAGE_UNIFORM | USAGE_COPY_DST);
    write_buffer(&device, &palette_buf, &vec![0.0f32; PALETTE_FLOATS]);

    // bind group
    let bg_entry0 = obj();
    set(&bg_entry0, "binding", &js_f64(0.0));
    let res0 = obj();  set(&res0, "buffer", &cam_buf.clone().into());
    set(&bg_entry0, "resource", &JsValue::from(res0));

    let bg_entry1 = obj();
    set(&bg_entry1, "binding", &js_f64(1.0));
    let res1 = obj();  set(&res1, "buffer", &palette_buf.clone().into());
    set(&bg_entry1, "resource", &JsValue::from(res1));

    let bg_entries = Array::new();
    bg_entries.push(&JsValue::from(bg_entry0));
    bg_entries.push(&JsValue::from(bg_entry1));
    let bg_desc = obj();
    set(&bg_desc, "layout",  &bgl_js);
    set(&bg_desc, "entries", &JsValue::from(bg_entries));
    let bind_group = Reflect::get(&device.clone().into(), &js_str("createBindGroup"))
        .and_then(|f| f.dyn_into::<Function>())
        .map_err(|_| "createBindGroup")?
        .call1(&device.clone().into(), &JsValue::from(bg_desc))
        .map_err(|_| "bg call")?;

    // shared full-screen quad (4 verts, triangle-strip)
    let quad_data: [f32; 8] = [-1.0, -1.0,  1.0, -1.0, -1.0, 1.0,  1.0, 1.0];
    let quad_buf = create_buf_init(&device, &quad_data, USAGE_VERTEX);

    let depth_view = create_depth_view(&device, canvas_w, canvas_h);

    Ok(GpuResources {
        device, ctx, edge_pipeline, node_quad_pipeline, bind_group,
        cam_buf, quad_buf, depth_view, canvas_w, canvas_h,
    })
}

fn build_edge_pipeline(
    device: &GpuDevice,
    pipeline_layout: &JsValue,
    format: &str,
) -> Result<GpuRenderPipeline, String> {
    let shader = make_shader(device, EDGE_SHADER);

    let quad_attr = obj();
    set(&quad_attr, "format", &js_str("float32x2"));
    set(&quad_attr, "offset", &js_f64(0.0));
    set(&quad_attr, "shaderLocation", &js_f64(0.0));
    let quad_attrs = Array::new(); quad_attrs.push(&JsValue::from(quad_attr));
    let quad_layout = obj();
    set(&quad_layout, "arrayStride", &js_f64(8.0));
    set(&quad_layout, "stepMode",    &js_str("vertex"));
    set(&quad_layout, "attributes",  &JsValue::from(quad_attrs));

    let inst_attrs = Array::new();
    let locs: &[(u32, &str, f64)] = &[
        (6,  "float32x3",  0.0),
        (7,  "float32x3", 12.0),
        (8,  "float32x4", 24.0),
        (9,  "float32",   40.0),
        (10, "float32",   44.0),
    ];
    for &(loc, fmt, offset) in locs {
        let a = obj();
        set(&a, "format", &js_str(fmt));
        set(&a, "offset", &js_f64(offset));
        set(&a, "shaderLocation", &js_f64(loc as f64));
        inst_attrs.push(&JsValue::from(a));
    }
    let inst_layout = obj();
    set(&inst_layout, "arrayStride", &js_f64(48.0));
    set(&inst_layout, "stepMode",    &js_str("instance"));
    set(&inst_layout, "attributes",  &JsValue::from(inst_attrs));

    let vert_bufs = Array::new();
    vert_bufs.push(&JsValue::from(quad_layout));
    vert_bufs.push(&JsValue::from(inst_layout));

    let vertex_state = obj();
    set(&vertex_state, "module",     &shader.clone().into());
    set(&vertex_state, "entryPoint", &js_str("vs_edge"));
    set(&vertex_state, "buffers",    &JsValue::from(vert_bufs));

    // premultiplied alpha blend
    let blend_comp = obj();
    set(&blend_comp, "srcFactor", &js_str("one"));
    set(&blend_comp, "dstFactor", &js_str("one-minus-src-alpha"));
    let blend = obj();
    set(&blend, "color", &JsValue::from(blend_comp.clone()));
    set(&blend, "alpha", &JsValue::from(blend_comp));
    let target0 = obj();
    set(&target0, "format", &js_str(format));
    set(&target0, "blend",  &JsValue::from(blend));
    let targets = Array::new(); targets.push(&JsValue::from(target0));

    let frag_state = obj();
    set(&frag_state, "module",     &shader.into());
    set(&frag_state, "entryPoint", &js_str("fs_edge"));
    set(&frag_state, "targets",    &JsValue::from(targets));

    let primitive = obj();
    set(&primitive, "topology", &js_str("triangle-strip"));

    let ds = obj();
    set(&ds, "format",             &js_str("depth24plus"));
    set(&ds, "depthWriteEnabled",  &JsValue::FALSE);
    set(&ds, "depthCompare",       &js_str("less-equal"));

    let pipe_desc = obj();
    set(&pipe_desc, "layout",       pipeline_layout);
    set(&pipe_desc, "vertex",       &JsValue::from(vertex_state));
    set(&pipe_desc, "fragment",     &JsValue::from(frag_state));
    set(&pipe_desc, "primitive",    &JsValue::from(primitive));
    set(&pipe_desc, "depthStencil", &JsValue::from(ds));

    Reflect::get(&device.clone().into(), &js_str("createRenderPipeline"))
        .and_then(|f| f.dyn_into::<Function>())
        .map_err(|_| "createRenderPipeline")?
        .call1(&device.clone().into(), &JsValue::from(pipe_desc))
        .map_err(|_| "edge pipeline call".to_string())?
        .dyn_into::<GpuRenderPipeline>()
        .map_err(|_| "edge pipeline cast".to_string())
}

fn build_node_quad_pipeline(
    device: &GpuDevice,
    pipeline_layout: &JsValue,
    format: &str,
) -> Result<GpuRenderPipeline, String> {
    let shader = make_shader(device, NODE_QUAD_SHADER);

    let q_attr = obj();
    set(&q_attr, "format", &js_str("float32x2"));
    set(&q_attr, "offset", &js_f64(0.0));
    set(&q_attr, "shaderLocation", &js_f64(0.0));
    let q_attrs = Array::new(); q_attrs.push(&JsValue::from(q_attr));
    let q_layout = obj();
    set(&q_layout, "arrayStride", &js_f64(8.0));
    set(&q_layout, "stepMode",    &js_str("vertex"));
    set(&q_layout, "attributes",  &JsValue::from(q_attrs));

    let i_attr = obj();
    set(&i_attr, "format", &js_str("float32x3"));
    set(&i_attr, "offset", &js_f64(0.0));
    set(&i_attr, "shaderLocation", &js_f64(1.0));
    let i_attrs = Array::new(); i_attrs.push(&JsValue::from(i_attr));
    let i_layout = obj();
    set(&i_layout, "arrayStride", &js_f64(16.0));
    set(&i_layout, "stepMode",    &js_str("instance"));
    set(&i_layout, "attributes",  &JsValue::from(i_attrs));

    let vbufs = Array::new();
    vbufs.push(&JsValue::from(q_layout));
    vbufs.push(&JsValue::from(i_layout));

    let vertex_state = obj();
    set(&vertex_state, "module",     &shader.clone().into());
    set(&vertex_state, "entryPoint", &js_str("vs_node_quad"));
    set(&vertex_state, "buffers",    &JsValue::from(vbufs));

    let target = obj();
    set(&target, "format", &js_str(format));
    let targets = Array::new(); targets.push(&JsValue::from(target));

    let frag_state = obj();
    set(&frag_state, "module",     &shader.into());
    set(&frag_state, "entryPoint", &js_str("fs_node_quad"));
    set(&frag_state, "targets",    &JsValue::from(targets));

    let primitive = obj();
    set(&primitive, "topology", &js_str("triangle-strip"));

    let ds = obj();
    set(&ds, "format",            &js_str("depth24plus"));
    set(&ds, "depthWriteEnabled", &JsValue::TRUE);
    set(&ds, "depthCompare",      &js_str("less-equal"));

    let pipe_desc = obj();
    set(&pipe_desc, "layout",       pipeline_layout);
    set(&pipe_desc, "vertex",       &JsValue::from(vertex_state));
    set(&pipe_desc, "fragment",     &JsValue::from(frag_state));
    set(&pipe_desc, "primitive",    &JsValue::from(primitive));
    set(&pipe_desc, "depthStencil", &JsValue::from(ds));

    Reflect::get(&device.clone().into(), &js_str("createRenderPipeline"))
        .and_then(|f| f.dyn_into::<Function>())
        .map_err(|_| "createRenderPipeline node")?
        .call1(&device.clone().into(), &JsValue::from(pipe_desc))
        .map_err(|_| "node quad pipeline call".to_string())?
        .dyn_into::<GpuRenderPipeline>()
        .map_err(|_| "node quad pipeline cast".to_string())
}
