//! GPU device, pipeline, and shader-module factory.
//!
//! Mirrors `gpu-init.ts`. Creates the WebGPU adapter/device, concatenates
//! WGSL shader sources, and builds the three pipelines:
//!   - **background** — full-screen quad: smoke, element rects, CRT, grain
//!   - **particle**   — instanced quads: sparks, embers, beams, glitter
//!   - **compute**    — particle physics simulation
//!
//! The canvas is configured with `alphaMode: "opaque"` and sits behind
//! `#ui-root` (z-index 1 vs 3). DOM elements compose over it via the normal
//! browser stacking context — no DOM rasterisation pipeline is required.

#![cfg(target_arch = "wasm32")]

use js_sys::{Array, Function, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::HtmlCanvasElement;

use super::webgpu::*;

// ── Embedded WGSL shaders ────────────────────────────────────────────────────
// Concatenation order matches the TypeScript reference:
//   palette → types → noise → [particle_shading] → pipeline-specific
const PALETTE_WGSL:          &str = include_str!("../shaders/palette.wgsl");
const TYPES_WGSL:            &str = include_str!("../shaders/types.wgsl");
const NOISE_WGSL:            &str = include_str!("../shaders/noise.wgsl");
const PARTICLE_SHADING_WGSL: &str = include_str!("../shaders/particle_shading.wgsl");
const BACKGROUND_WGSL:       &str = include_str!("../shaders/background.wgsl");
const PARTICLES_WGSL:        &str = include_str!("../shaders/particles.wgsl");
const COMPUTE_WGSL:          &str = include_str!("../shaders/compute.wgsl");

pub(super) struct GpuPipelines {
    pub bg_pipeline:       JsValue,
    pub particle_pipeline: JsValue,
    pub compute_pipeline:  JsValue,
    pub compute_bgl:       JsValue,
    pub render_bgl:        JsValue,
}

pub(super) struct InitOutput {
    pub device:        JsValue,
    pub queue:         JsValue,
    pub context:       JsValue,
    pub pipelines:     GpuPipelines,
    pub canvas_width:  u32,
    pub canvas_height: u32,
}

/// Acquire `#webgpu-canvas`, request adapter+device, configure the canvas
/// context, build shader modules and pipelines.  Returns `None` when WebGPU
/// is unavailable or any step fails.
pub(super) async fn init_gpu() -> Option<InitOutput> {
    let win = web_sys::window()?;
    let doc = win.document()?;

    // ── Acquire canvas ──────────────────────────────────────────────────────
    let canvas = doc
        .get_element_by_id("webgpu-canvas")?
        .dyn_into::<HtmlCanvasElement>()
        .ok()?;
    let dpr = win.device_pixel_ratio();
    let init_w = ((canvas.client_width()  as f64 * dpr) as u32).max(1);
    let init_h = ((canvas.client_height() as f64 * dpr) as u32).max(1);
    canvas.set_width(init_w);
    canvas.set_height(init_h);

    // ── navigator.gpu ───────────────────────────────────────────────────────
    let navigator = win.navigator();
    let gpu_js = Reflect::get(&navigator, &"gpu".into()).ok()?;
    if gpu_js.is_undefined() || gpu_js.is_null() {
        web_sys::console::warn_1(&"[WgpuOverlay] navigator.gpu unavailable".into());
        return None;
    }

    // ── requestAdapter() ────────────────────────────────────────────────────
    let request_adapter: Function = get_fn(&gpu_js, "requestAdapter")?;
    let adapter_promise: Promise = request_adapter.call0(&gpu_js).ok()?.dyn_into().ok()?;
    let adapter = JsFuture::from(adapter_promise).await.ok()?;
    if adapter.is_null() || adapter.is_undefined() { return None; }

    // ── requestDevice() ─────────────────────────────────────────────────────
    let request_device: Function = get_fn(&adapter, "requestDevice")?;
    let device_promise: Promise = request_device.call0(&adapter).ok()?.dyn_into().ok()?;
    let device = JsFuture::from(device_promise).await.ok()?;
    if device.is_null() || device.is_undefined() { return None; }

    let queue = Reflect::get(&device, &"queue".into()).ok()?;

    // ── Canvas WebGPU context ───────────────────────────────────────────────
    let context: JsValue = canvas.get_context("webgpu").ok()??.into();
    let format: JsValue = get_fn(&gpu_js, "getPreferredCanvasFormat")?.call0(&gpu_js).ok()?;

    // Configure the canvas context. `alphaMode: "opaque"` matches the
    // TypeScript reference — the canvas paints a full opaque background and
    // the DOM stacks above it via normal z-index ordering.
    let cfg = Object::new();
    set_prop(&cfg, "device",    &device);
    set_prop(&cfg, "format",    &format);
    set_prop(&cfg, "alphaMode", &"opaque".into());
    get_fn(&context, "configure")?.call1(&context, &cfg.into()).ok()?;

    // ── Shader modules ──────────────────────────────────────────────────────
    let shared_code   = format!("{}\n{}\n{}\n", PALETTE_WGSL, TYPES_WGSL, NOISE_WGSL);
    let render_shared = format!("{}{}\n", shared_code, PARTICLE_SHADING_WGSL);

    let bg_shader       = create_shader(&device, "background", &format!("{}{}", render_shared, BACKGROUND_WGSL))?;
    let particle_shader = create_shader(&device, "particles",  &format!("{}{}", render_shared, PARTICLES_WGSL))?;
    let compute_shader  = create_shader(&device, "compute",    &format!("{}{}", shared_code,   COMPUTE_WGSL))?;

    // ── Bind-group layouts ──────────────────────────────────────────────────
    //
    // Compute BGL (matches compute.wgsl):
    //   0: uniform                       — Uniforms
    //   1: read-only-storage             — array<ElemRect>
    //   2: storage (read_write)          — array<Particle>
    //   3: uniform                       — ThemePalette
    let compute_bgl = {
        let entries = Array::new();
        entries.push(&bgl_buf(0, 4, "uniform"));
        entries.push(&bgl_buf(1, 4, "read-only-storage"));
        entries.push(&bgl_buf(2, 4, "storage"));
        entries.push(&bgl_buf(3, 4, "uniform"));
        create_bgl(&device, &entries)?
    };

    // Render BGL (matches background.wgsl + particles.wgsl):
    //   0: uniform                       — Uniforms          (VERTEX|FRAGMENT)
    //   1: read-only-storage             — array<ElemRect>   (VERTEX|FRAGMENT)
    //   2: read-only-storage             — array<Particle>   (VERTEX|FRAGMENT)
    //   3: uniform                       — ThemePalette      (FRAGMENT)
    let render_bgl = {
        let entries = Array::new();
        entries.push(&bgl_buf(0, 3, "uniform"));
        entries.push(&bgl_buf(1, 3, "read-only-storage"));
        entries.push(&bgl_buf(2, 3, "read-only-storage"));
        entries.push(&bgl_buf(3, 2, "uniform"));
        create_bgl(&device, &entries)?
    };

    // ── Pipeline layouts ────────────────────────────────────────────────────
    let compute_layout = create_pipeline_layout(&device, &[&compute_bgl])?;
    let render_layout  = create_pipeline_layout(&device, &[&render_bgl])?;

    // ── Pipelines ───────────────────────────────────────────────────────────
    let compute_pipeline  = create_compute_pipeline(&device, &compute_layout, &compute_shader)?;
    let bg_pipeline       = create_render_pipeline(
        &device, &render_layout, &bg_shader,       &bg_shader,       &format, /*additive=*/ false)?;
    let particle_pipeline = create_render_pipeline(
        &device, &render_layout, &particle_shader, &particle_shader, &format, /*additive=*/ true)?;

    Some(InitOutput {
        device,
        queue,
        context,
        pipelines: GpuPipelines {
            bg_pipeline,
            particle_pipeline,
            compute_pipeline,
            compute_bgl,
            render_bgl,
        },
        canvas_width:  init_w,
        canvas_height: init_h,
    })
}
