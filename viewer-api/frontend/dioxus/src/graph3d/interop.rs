//! Thin JS / WebGPU interop helpers used by the graph3d module.
//!
//! `web-sys` does not yet expose the full WebGPU API, so we go through
//! `js_sys::Reflect` for descriptors, pipeline creation, render-pass
//! encoding etc.

#![cfg(target_arch = "wasm32")]

use js_sys::{Array, Function, Object, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{GpuBuffer, GpuDevice};

pub(crate) const USAGE_VERTEX:   u32 = 0x0020;
pub(crate) const USAGE_UNIFORM:  u32 = 0x0040;
pub(crate) const USAGE_COPY_DST: u32 = 0x0008;

pub(crate) fn obj() -> Object { Object::new() }

pub(crate) fn set(o: &Object, key: &str, val: &JsValue) {
    Reflect::set(o, &JsValue::from_str(key), val).ok();
}

pub(crate) fn js_str(s: &str) -> JsValue { JsValue::from_str(s) }
pub(crate) fn js_f64(v: f64)  -> JsValue { JsValue::from_f64(v) }

pub(crate) fn preferred_format() -> String {
    let nav: JsValue = web_sys::window().unwrap().navigator().into();
    let gpu = Reflect::get(&nav, &js_str("gpu")).unwrap_or(JsValue::UNDEFINED);
    if gpu.is_undefined() { return "bgra8unorm".into(); }
    Reflect::get(&gpu, &js_str("getPreferredCanvasFormat"))
        .ok()
        .and_then(|f| f.dyn_into::<Function>().ok())
        .and_then(|f| f.call0(&gpu).ok())
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| "bgra8unorm".into())
}

/// Call a method on a JS object with 0–3 arguments via Reflect.
pub(crate) fn js_call(target: &JsValue, method: &str, args: &[&JsValue]) {
    let Ok(f_val) = Reflect::get(target, &js_str(method)) else { return };
    let Ok(f)     = f_val.dyn_into::<Function>()          else { return };
    match args.len() {
        0 => { f.call0(target).ok(); }
        1 => { f.call1(target, args[0]).ok(); }
        2 => { f.call2(target, args[0], args[1]).ok(); }
        3 => { f.call3(target, args[0], args[1], args[2]).ok(); }
        _ => {}
    }
}

pub(crate) fn write_buffer(device: &GpuDevice, buf: &GpuBuffer, data: &[f32]) {
    let bytes: &[u8] =
        unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4) };
    let uint8 = js_sys::Uint8Array::from(bytes);
    let _ = device
        .queue()
        .write_buffer_with_u32_and_u8_array(buf, 0, &uint8);
}

pub(crate) fn create_buf(device: &GpuDevice, size: usize, usage: u32) -> GpuBuffer {
    let desc = obj();
    set(&desc, "size",  &js_f64(size as f64));
    set(&desc, "usage", &js_f64(usage as f64));
    device
        .create_buffer(&web_sys::GpuBufferDescriptor::from(JsValue::from(desc)))
        .expect("create_buffer")
}

pub(crate) fn create_buf_init(device: &GpuDevice, data: &[f32], usage: u32) -> GpuBuffer {
    let buf = create_buf(device, data.len() * 4, usage | USAGE_COPY_DST);
    write_buffer(device, &buf, data);
    buf
}

/// Create a depth texture and return its default view.
pub(crate) fn create_depth_view(device: &GpuDevice, w: u32, h: u32) -> JsValue {
    let desc = obj();
    let size = Array::new();
    size.push(&js_f64(w as f64));
    size.push(&js_f64(h as f64));
    set(&desc, "size",   &JsValue::from(size));
    set(&desc, "format", &js_str("depth24plus"));
    set(&desc, "usage",  &js_f64(16.0)); // RENDER_ATTACHMENT
    let device_js: JsValue = device.clone().into();
    let tex = Reflect::get(&device_js, &js_str("createTexture"))
        .and_then(|f| f.dyn_into::<Function>())
        .expect("createTexture")
        .call1(&device_js, &JsValue::from(desc))
        .expect("createTexture call");
    Reflect::get(&tex, &js_str("createView"))
        .and_then(|f| f.dyn_into::<Function>())
        .expect("createView")
        .call0(&tex)
        .expect("createView call")
}

pub(crate) fn make_shader(device: &GpuDevice, code: &str) -> web_sys::GpuShaderModule {
    let desc = web_sys::GpuShaderModuleDescriptor::new(code);
    device.create_shader_module(&desc)
}
