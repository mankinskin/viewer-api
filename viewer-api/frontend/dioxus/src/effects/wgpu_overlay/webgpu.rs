//! Low-level WebGPU JS-interop helpers.
//!
//! All WebGPU calls go through `js_sys::Reflect`/`Function::call*` because
//! `web_sys` lacks the `webgpu` bindings stable.  These helpers wrap the
//! repetitive boilerplate (build descriptor `Object`, set props, invoke
//! method) into ergonomic functions used by the GPU init / buffers / render
//! modules.

#![cfg(target_arch = "wasm32")]

use js_sys::{Array, Float32Array, Function, Object, Reflect};
use wasm_bindgen::{JsCast, JsValue};

// ── Generic JS reflection ────────────────────────────────────────────────────

/// Get a callable `Function` property from `obj`, or `None` if absent.
pub(super) fn get_fn(obj: &JsValue, name: &str) -> Option<Function> {
    Reflect::get(obj, &name.into()).ok()?.dyn_into::<Function>().ok()
}

/// Set a property on `obj` via `Reflect.set`.
pub(super) fn set_prop(obj: &Object, key: &str, value: &JsValue) {
    let _ = Reflect::set(obj, &key.into(), value);
}

/// Read a numeric property and cast to `f32`.
pub(super) fn prop_f32(obj: &JsValue, key: &str) -> f32 {
    Reflect::get(obj, &key.into())
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as f32
}

// ── Shader / pipeline factories ──────────────────────────────────────────────

pub(super) fn create_shader(device: &JsValue, label: &str, code: &str) -> Option<JsValue> {
    let desc = Object::new();
    set_prop(&desc, "label", &label.into());
    set_prop(&desc, "code",  &code.into());
    get_fn(device, "createShaderModule")?.call1(device, &desc.into()).ok()
}

/// Build a `GPUBindGroupLayoutEntry` for a buffer binding.
/// `visibility` bitmask: `1=VERTEX`, `2=FRAGMENT`, `4=COMPUTE`.
pub(super) fn bgl_buf(binding: u32, visibility: u32, ty: &str) -> JsValue {
    let entry = Object::new();
    set_prop(&entry, "binding",    &binding.into());
    set_prop(&entry, "visibility", &visibility.into());
    let buf = Object::new();
    set_prop(&buf, "type", &ty.into());
    set_prop(&entry, "buffer", &buf.into());
    entry.into()
}

pub(super) fn create_bgl(device: &JsValue, entries: &Array) -> Option<JsValue> {
    let desc = Object::new();
    set_prop(&desc, "entries", entries);
    get_fn(device, "createBindGroupLayout")?.call1(device, &desc.into()).ok()
}

pub(super) fn create_pipeline_layout(device: &JsValue, bgls: &[&JsValue]) -> Option<JsValue> {
    let arr = Array::new();
    for &bgl in bgls { arr.push(bgl); }
    let desc = Object::new();
    set_prop(&desc, "bindGroupLayouts", &arr.into());
    get_fn(device, "createPipelineLayout")?.call1(device, &desc.into()).ok()
}

pub(super) fn create_compute_pipeline(
    device: &JsValue, layout: &JsValue, shader: &JsValue,
) -> Option<JsValue> {
    let compute = Object::new();
    set_prop(&compute, "module",     shader);
    set_prop(&compute, "entryPoint", &"cs_main".into());
    let desc = Object::new();
    set_prop(&desc, "layout",  layout);
    set_prop(&desc, "compute", &compute.into());
    get_fn(device, "createComputePipeline")?.call1(device, &desc.into()).ok()
}

/// Build a render pipeline.
///
/// When `additive` is true the fragment stage uses additive blending and
/// the `vs_particle` / `fs_particle` entry points (particle pass).
/// Otherwise it uses opaque output with `vs_main` / `fs_main` (background
/// pass — the canvas is opaque so no alpha blending is required).
pub(super) fn create_render_pipeline(
    device: &JsValue, layout: &JsValue,
    vs: &JsValue, fs: &JsValue,
    format: &JsValue, additive: bool,
) -> Option<JsValue> {
    let vs_state = Object::new();
    set_prop(&vs_state, "module", vs);
    let vs_ep: JsValue = if additive { "vs_particle".into() } else { "vs_main".into() };
    set_prop(&vs_state, "entryPoint", &vs_ep);

    let color_target = Object::new();
    set_prop(&color_target, "format", format);
    if additive {
        // Additive blend: ONE + ONE (sparks, embers, glitter on opaque canvas).
        let bc = Object::new();
        set_prop(&bc, "srcFactor", &"one".into());
        set_prop(&bc, "dstFactor", &"one".into());
        set_prop(&bc, "operation", &"add".into());
        let blend = Object::new();
        set_prop(&blend, "color", &bc.clone().into());
        set_prop(&blend, "alpha", &bc.into());
        set_prop(&color_target, "blend", &blend.into());
    }
    // Opaque background pass: omit the `blend` field entirely.

    let targets = Array::new();
    targets.push(&color_target.into());

    let fs_state = Object::new();
    set_prop(&fs_state, "module", fs);
    let fs_ep: JsValue = if additive { "fs_particle".into() } else { "fs_main".into() };
    set_prop(&fs_state, "entryPoint", &fs_ep);
    set_prop(&fs_state, "targets",    &targets.into());

    let prim = Object::new();
    set_prop(&prim, "topology", &"triangle-list".into());

    let depth = Object::new();
    set_prop(&depth, "format",            &"depth24plus".into());
    set_prop(&depth, "depthWriteEnabled", &JsValue::FALSE);
    set_prop(&depth, "depthCompare",      &"always".into());

    let desc = Object::new();
    set_prop(&desc, "layout",       layout);
    set_prop(&desc, "vertex",       &vs_state.into());
    set_prop(&desc, "fragment",     &fs_state.into());
    set_prop(&desc, "primitive",    &prim.into());
    set_prop(&desc, "depthStencil", &depth.into());

    get_fn(device, "createRenderPipeline")?.call1(device, &desc.into()).ok()
}

// ── Buffer helpers ───────────────────────────────────────────────────────────

pub(super) fn gpu_buffer(device: &JsValue, size: u32, usage: u32) -> Option<JsValue> {
    let desc = Object::new();
    set_prop(&desc, "size",  &(size as f64).into());
    set_prop(&desc, "usage", &usage.into());
    get_fn(device, "createBuffer")?.call1(device, &desc.into()).ok()
}

pub(super) fn queue_write_f32(
    queue: &JsValue, buf: &JsValue, offset_bytes: u32, data: &Float32Array,
) {
    if let Some(f) = get_fn(queue, "writeBuffer") {
        let args = Array::new();
        args.push(buf);
        args.push(&(offset_bytes as f64).into());
        args.push(&data.buffer().into());
        args.push(&(data.byte_offset() as f64).into());
        args.push(&(data.byte_length() as f64).into());
        let _ = f.apply(queue, &args);
    }
}

// ── Bind-group entry helpers ─────────────────────────────────────────────────

pub(super) fn bg_binding_entry(binding: u32, resource: &JsValue) -> JsValue {
    let entry = Object::new();
    set_prop(&entry, "binding",  &binding.into());
    set_prop(&entry, "resource", resource);
    entry.into()
}

pub(super) fn buf_resource(buf: &JsValue) -> JsValue {
    let b = Object::new();
    set_prop(&b, "buffer", buf);
    b.into()
}

// ── Texture / view helpers ───────────────────────────────────────────────────

pub(super) fn create_tex_view(texture: &JsValue) -> Option<JsValue> {
    get_fn(texture, "createView")?.call0(texture).ok()
}

pub(super) fn create_depth_texture(device: &JsValue, w: u32, h: u32) -> Option<(JsValue, JsValue)> {
    let size = Object::new();
    set_prop(&size, "width",  &(w as f64).into());
    set_prop(&size, "height", &(h as f64).into());
    let desc = Object::new();
    set_prop(&desc, "size",   &size.into());
    set_prop(&desc, "format", &"depth24plus".into());
    // GPUTextureUsage.RENDER_ATTACHMENT = 0x10
    set_prop(&desc, "usage",  &0x10u32.into());
    let tex  = get_fn(device, "createTexture")?.call1(device, &desc.into()).ok()?;
    let view = create_tex_view(&tex)?;
    Some((tex, view))
}

// ── Render-pass encoding ─────────────────────────────────────────────────────

pub(super) fn build_render_pass_desc(color_view: &JsValue, depth_view: &JsValue) -> JsValue {
    let ca = Object::new();
    set_prop(&ca, "view",    color_view);
    set_prop(&ca, "loadOp",  &"clear".into());
    set_prop(&ca, "storeOp", &"store".into());
    let clear = Object::new();
    set_prop(&clear, "r", &0.0f64.into());
    set_prop(&clear, "g", &0.0f64.into());
    set_prop(&clear, "b", &0.0f64.into());
    set_prop(&clear, "a", &1.0f64.into()); // opaque canvas
    set_prop(&ca, "clearValue", &clear.into());

    let da = Object::new();
    set_prop(&da, "view",            depth_view);
    set_prop(&da, "depthLoadOp",     &"clear".into());
    set_prop(&da, "depthStoreOp",    &"discard".into());
    set_prop(&da, "depthClearValue", &1.0f64.into());

    let color_attachments = Array::new();
    color_attachments.push(&ca.into());

    let desc = Object::new();
    set_prop(&desc, "colorAttachments",       &color_attachments.into());
    set_prop(&desc, "depthStencilAttachment", &da.into());
    desc.into()
}

pub(super) fn call_set_pipeline(pass: &JsValue, pipeline: &JsValue) {
    if let Some(f) = get_fn(pass, "setPipeline") { let _ = f.call1(pass, pipeline); }
}

pub(super) fn call_set_bind_group(pass: &JsValue, index: u32, bg: &JsValue) {
    if let Some(f) = get_fn(pass, "setBindGroup") {
        let _ = f.call2(pass, &index.into(), bg);
    }
}

pub(super) fn call_dispatch(pass: &JsValue, x: u32) {
    if let Some(f) = get_fn(pass, "dispatchWorkgroups") {
        let _ = f.call1(pass, &x.into());
    }
}

pub(super) fn call_draw(pass: &JsValue, vertices: u32, instances: u32) {
    if let Some(f) = get_fn(pass, "draw") {
        let _ = f.call2(pass, &vertices.into(), &instances.into());
    }
}

pub(super) fn call_end(pass: &JsValue) {
    if let Some(f) = get_fn(pass, "end") { let _ = f.call0(pass); }
}
