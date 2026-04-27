//! Per-frame render orchestration: rAF loop, uniform packing, compute and
//! render passes.  Mirrors `gpu-render-loop.ts`.

#![cfg(target_arch = "wasm32")]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use dioxus::prelude::*;
use js_sys::{Array, Float32Array};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{HtmlCanvasElement, Window};

use super::element_scanner::scan_ui_rects;
use super::element_types::*;
use super::gpu_buffers::{mk_compute_bind_group, mk_render_bind_group, GpuBuffers};
use super::gpu_init::{init_gpu, GpuPipelines};
use super::settings::EffectSettings;
use super::webgpu::*;

// ── Per-frame GPU context ────────────────────────────────────────────────────

pub(super) struct GpuCtx {
    device:        JsValue,
    queue:         JsValue,
    /// `GPUCanvasContext` wrapping `#webgpu-canvas`.
    context:       JsValue,
    pipelines:     GpuPipelines,
    buffers:       GpuBuffers,
    compute_bg:    JsValue,
    render_bg:     JsValue,
    /// Depth texture (`depth24plus`) — recreated when canvas resizes.
    depth_tex:     JsValue,
    depth_view:    JsValue,
    depth_w:       u32,
    depth_h:       u32,
    /// CPU-side uniforms packed for `queue.writeBuffer`.
    uniforms_f32:  Float32Array,
    /// Timestamp of the first frame (ms from `performance.now`).
    start_time_ms: f64,
    /// Timestamp of the previous frame (ms).
    prev_time_ms:  f64,
    /// Effect settings loaded from localStorage on init.
    settings:      EffectSettings,
}

type SharedCtx = Rc<RefCell<Option<GpuCtx>>>;

// ── Hook entry point ─────────────────────────────────────────────────────────

/// Dioxus hook: bootstrap the WebGPU pipeline and start the rAF loop.
/// Called from `WgpuOverlay` on every render — `use_hook` ensures the
/// shared state is initialised exactly once per component lifetime.
pub fn mount_overlay() {
    let ctx: SharedCtx = use_hook(|| Rc::new(RefCell::new(None::<GpuCtx>)));
    let keep_running:   Rc<Cell<bool>> = use_hook(|| Rc::new(Cell::new(true)));
    let raf_id:         Rc<Cell<i32>>  = use_hook(|| Rc::new(Cell::new(0i32)));
    let raf_closure_jv: Rc<RefCell<Option<JsValue>>> =
        use_hook(|| Rc::new(RefCell::new(None::<JsValue>)));
    let initialized:    Rc<Cell<bool>> = use_hook(|| Rc::new(Cell::new(false)));

    // ── Cleanup on unmount ──────────────────────────────────────────────────
    {
        let kr  = Rc::clone(&keep_running);
        let ri  = Rc::clone(&raf_id);
        let rjv = Rc::clone(&raf_closure_jv);
        let ctx_drop = Rc::clone(&ctx);
        use_drop(move || {
            kr.set(false);
            let id = ri.get();
            if id != 0 {
                if let Some(w) = web_sys::window() { let _ = w.cancel_animation_frame(id); }
            }
            *rjv.borrow_mut()      = None;
            *ctx_drop.borrow_mut() = None;
        });
    }

    // ── One-time GPU bootstrap ──────────────────────────────────────────────
    {
        let init_flag = Rc::clone(&initialized);
        let ctx_ref   = Rc::clone(&ctx);
        let kr_ref    = Rc::clone(&keep_running);
        let ri_ref    = Rc::clone(&raf_id);
        let rjv_ref   = Rc::clone(&raf_closure_jv);

        use_effect(move || {
            if init_flag.get() { return; }
            init_flag.set(true);

            let ctx_e = Rc::clone(&ctx_ref);
            let kr_e  = Rc::clone(&kr_ref);
            let ri_e  = Rc::clone(&ri_ref);
            let rjv_e = Rc::clone(&rjv_ref);

            spawn(async move {
                match bootstrap_ctx().await {
                    Some(gpu_ctx) => {
                        *ctx_e.borrow_mut() = Some(gpu_ctx);
                        setup_raf_loop(
                            Rc::clone(&ctx_e),
                            Rc::clone(&kr_e),
                            Rc::clone(&ri_e),
                            Rc::clone(&rjv_e),
                        );
                    }
                    None => {
                        web_sys::console::warn_1(
                            &"[WgpuOverlay] WebGPU unavailable — overlay disabled".into(),
                        );
                    }
                }
            });
        });
    }
}

// ── Bootstrap ────────────────────────────────────────────────────────────────

async fn bootstrap_ctx() -> Option<GpuCtx> {
    let init = init_gpu().await?;
    let win  = web_sys::window()?;
    let perf = win.performance()?;
    let now  = perf.now();

    let buffers    = GpuBuffers::new(&init.device, &init.queue)?;
    let compute_bg = mk_compute_bind_group(&init.device, &init.pipelines.compute_bgl, &buffers)?;
    let render_bg  = mk_render_bind_group (&init.device, &init.pipelines.render_bgl,  &buffers)?;
    let (depth_tex, depth_view) =
        create_depth_texture(&init.device, init.canvas_width, init.canvas_height)?;

    Some(GpuCtx {
        device:        init.device,
        queue:         init.queue,
        context:       init.context,
        pipelines:     init.pipelines,
        buffers,
        compute_bg,
        render_bg,
        depth_tex,
        depth_view,
        depth_w:       init.canvas_width,
        depth_h:       init.canvas_height,
        uniforms_f32:  Float32Array::new_with_length(UNIFORMS_F32_COUNT as u32),
        start_time_ms: now,
        prev_time_ms:  now,
        settings:      EffectSettings::load("default"),
    })
}

// ── rAF loop setup ───────────────────────────────────────────────────────────

/// Create one persistent `requestAnimationFrame` closure and kick off the
/// loop.  The closure self-re-schedules until `keep_running` is set to
/// `false` by `use_drop`.
fn setup_raf_loop(
    ctx:          SharedCtx,
    keep_running: Rc<Cell<bool>>,
    raf_id:       Rc<Cell<i32>>,
    raf_jv:       Rc<RefCell<Option<JsValue>>>,
) {
    let ctx_loop    = Rc::clone(&ctx);
    let kr_loop     = Rc::clone(&keep_running);
    let ri_loop     = Rc::clone(&raf_id);
    let raf_jv_loop = Rc::clone(&raf_jv);

    let closure = Closure::<dyn FnMut(f64)>::new(move |ts_ms: f64| {
        if !kr_loop.get() { return; }
        if let Some(win) = web_sys::window() {
            // Yield the canvas to other renderers (e.g. Graph3D) when claimed.
            let canvas_free     = !crate::effects::wgpu_overlay::is_canvas_owned();
            let overlay_enabled = crate::effects::wgpu_overlay::is_overlay_enabled();
            if canvas_free && overlay_enabled {
                if let Some(gpu) = ctx_loop.borrow_mut().as_mut() {
                    render_frame(gpu, ts_ms, &win);
                }
            }
            if let Some(ref jv) = *raf_jv_loop.borrow() {
                if let Ok(id) = win.request_animation_frame(jv.unchecked_ref()) {
                    ri_loop.set(id);
                }
            }
        }
    });

    // Transfer closure ownership to JS GC.
    let jv = closure.into_js_value();
    if let Some(win) = web_sys::window() {
        if let Ok(id) = win.request_animation_frame(jv.unchecked_ref()) {
            raf_id.set(id);
        }
    }
    *raf_jv.borrow_mut() = Some(jv);
}

// ── Per-frame render ─────────────────────────────────────────────────────────

fn render_frame(gpu: &mut GpuCtx, ts_ms: f64, win: &Window) {
    let dt_s   = ((ts_ms - gpu.prev_time_ms) / 1000.0).min(0.1) as f32;
    gpu.prev_time_ms = ts_ms;
    let time_s = ((ts_ms - gpu.start_time_ms) / 1000.0) as f32;

    // ── Resize canvas to device pixels ──────────────────────────────────────
    let Some(doc) = win.document() else { return; };
    let Some(canvas) = doc
        .get_element_by_id("webgpu-canvas")
        .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
    else { return; };
    let dpr = win.device_pixel_ratio();
    let cw  = ((canvas.client_width()  as f64 * dpr) as u32).max(1);
    let ch  = ((canvas.client_height() as f64 * dpr) as u32).max(1);
    if cw != canvas.width()  { canvas.set_width(cw); }
    if ch != canvas.height() { canvas.set_height(ch); }

    // Recreate depth texture if canvas was resized.
    if cw != gpu.depth_w || ch != gpu.depth_h {
        if let Some((dt, dv)) = create_depth_texture(&gpu.device, cw, ch) {
            gpu.depth_tex  = dt;
            gpu.depth_view = dv;
            gpu.depth_w    = cw;
            gpu.depth_h    = ch;
        }
    }

    // ── DOM element scan ────────────────────────────────────────────────────
    let (elem_data, elem_count) = scan_ui_rects(&doc);

    // Grow the element buffer if scanned count exceeded capacity.
    if gpu.buffers.ensure_elem_capacity(&gpu.device, elem_count) {
        if let Some(cb) = mk_compute_bind_group(&gpu.device, &gpu.pipelines.compute_bgl, &gpu.buffers) {
            gpu.compute_bg = cb;
        }
        if let Some(rb) = mk_render_bind_group(&gpu.device, &gpu.pipelines.render_bgl, &gpu.buffers) {
            gpu.render_bg = rb;
        }
    }

    // Upload element rects.
    if !elem_data.is_empty() {
        // SAFETY: `elem_data` lives for the duration of this call; the view
        // is consumed before this function returns.
        let fa = unsafe { Float32Array::view(&elem_data) };
        queue_write_f32(&gpu.queue, &gpu.buffers.elem_buf, 0, &fa);
    }

    // ── Pack uniforms ───────────────────────────────────────────────────────
    pack_uniforms(gpu, time_s, dt_s, cw, ch, elem_count);
    queue_write_f32(&gpu.queue, &gpu.buffers.uniform_buf, 0, &gpu.uniforms_f32);

    // ── Get current frame texture ───────────────────────────────────────────
    let Some(frame_tex) = get_fn(&gpu.context, "getCurrentTexture")
        .and_then(|f| f.call0(&gpu.context).ok()) else { return; };
    let Some(frame_view) = create_tex_view(&frame_tex) else { return; };

    // ── Command encoder ─────────────────────────────────────────────────────
    let Some(encoder) = get_fn(&gpu.device, "createCommandEncoder")
        .and_then(|f| f.call0(&gpu.device).ok()) else { return; };

    // ── Compute pass (particle physics) ─────────────────────────────────────
    if gpu.settings.particles_enabled {
        if let Some(pass) = get_fn(&encoder, "beginComputePass")
            .and_then(|f| f.call0(&encoder).ok())
        {
            call_set_pipeline(&pass, &gpu.pipelines.compute_pipeline);
            call_set_bind_group(&pass, 0, &gpu.compute_bg);
            let wg = ((NUM_PARTICLES + COMPUTE_WORKGROUP - 1) / COMPUTE_WORKGROUP) as u32;
            call_dispatch(&pass, wg);
            call_end(&pass);
        }
    }

    // ── Render pass ─────────────────────────────────────────────────────────
    {
        let rp_desc = build_render_pass_desc(&frame_view, &gpu.depth_view);
        if let Some(pass) = get_fn(&encoder, "beginRenderPass")
            .and_then(|f| f.call1(&encoder, &rp_desc).ok())
        {
            // a) Background full-screen quad: smoke, element rects, CRT, grain, vignette.
            call_set_pipeline(&pass, &gpu.pipelines.bg_pipeline);
            call_set_bind_group(&pass, 0, &gpu.render_bg);
            call_draw(&pass, 6, 1);

            // b) Particle quads (additive blend — sparks, embers, beams, glitter).
            if gpu.settings.particles_enabled {
                call_set_pipeline(&pass, &gpu.pipelines.particle_pipeline);
                call_set_bind_group(&pass, 0, &gpu.render_bg);
                call_draw(&pass, 6, NUM_PARTICLES as u32);
            }

            call_end(&pass);
        }
    }

    // ── Submit ──────────────────────────────────────────────────────────────
    if let Some(finish) = get_fn(&encoder, "finish")
        .and_then(|f| f.call0(&encoder).ok())
    {
        if let Some(submit) = get_fn(&gpu.queue, "submit") {
            let cmds = Array::new();
            cmds.push(&finish);
            let _ = submit.call1(&gpu.queue, &cmds);
        }
    }

    // Suppress unused suggestions for fields used only for resource ownership.
    let _ = &gpu.depth_tex;
}

// ── Uniform packing ──────────────────────────────────────────────────────────

fn pack_uniforms(gpu: &GpuCtx, time_s: f32, dt_s: f32, cw: u32, ch: u32, elem_count: usize) {
    let u  = &gpu.uniforms_f32;
    let s  = &gpu.settings;
    let vp = ortho_vp(cw as f32, ch as f32);
    let iv = ortho_inv_vp(cw as f32, ch as f32);

    // Scalars [0..55]
    u.set_index(0,  time_s);
    u.set_index(1,  cw as f32);
    u.set_index(2,  ch as f32);
    u.set_index(3,  elem_count as f32);
    u.set_index(4,  -9999.0); // mouse_x
    u.set_index(5,  -9999.0); // mouse_y
    u.set_index(6,  dt_s);
    u.set_index(7,  -1.0);    // hover_elem
    u.set_index(8,  0.0);     // hover_start_time
    u.set_index(9,  -1.0);    // selected_elem
    u.set_index(10, s.crt_scanlines_h);
    u.set_index(11, 0.0);     // crt_scanlines_v
    u.set_index(12, s.crt_edge_shadow);
    u.set_index(13, 0.08);    // crt_flicker
    u.set_index(14, 0.3);     // crt_line_width
    u.set_index(15, s.smoke_intensity);
    u.set_index(16, 1.0);     // smoke_speed
    u.set_index(17, 1.0);     // smoke_warm_scale
    u.set_index(18, 1.0);     // smoke_cool_scale
    u.set_index(19, 1.0);     // smoke_moss_scale
    u.set_index(20, s.grain_intensity);
    u.set_index(21, 0.5);     // grain_coarseness
    u.set_index(22, 0.3);     // grain_size
    u.set_index(23, s.vignette_str);
    u.set_index(24, 0.2);     // underglow_str
    u.set_index(25, 1.0);     // spark_speed
    u.set_index(26, 1.0);     // ember_speed
    u.set_index(27, 1.0);     // beam_speed
    u.set_index(28, 1.0);     // glitter_speed
    u.set_index(29, 35.0);    // beam_height
    u.set_index(30, 0.0);     // beam_count
    u.set_index(31, 1.0);     // beam_drift
    u.set_index(32, 0.0);     // scroll_dx
    u.set_index(33, 0.0);     // scroll_dy
    u.set_index(34, if s.particles_enabled { 1.0 } else { 0.0 }); // spark_count
    u.set_index(35, 1.0);     // spark_size
    u.set_index(36, if s.particles_enabled { 1.0 } else { 0.0 }); // ember_count
    u.set_index(37, 1.0);     // ember_size
    u.set_index(38, if s.particles_enabled { 1.0 } else { 0.0 }); // glitter_count
    u.set_index(39, 1.0);     // glitter_size
    u.set_index(40, 1.0);     // cinder_size
    u.set_index(41, 0.0);     // ref_depth
    u.set_index(42, 1.0);     // world_scale
    u.set_index(43, 0.0);     // vp_x
    u.set_index(44, 0.0);     // vp_y
    u.set_index(45, cw as f32);
    u.set_index(46, ch as f32);
    u.set_index(47, 0.0);     // current_view (0 = logs)
    u.set_index(48, 0.9);     // crt_color_r
    u.set_index(49, 0.7);     // crt_color_g
    u.set_index(50, 0.4);     // crt_color_b
    u.set_index(51, 0.0);     // _crt_pad
    u.set_index(52, 0.0);     // cam x
    u.set_index(53, 0.0);     // cam y
    u.set_index(54, 0.0);     // cam z
    u.set_index(55, 0.0);     // _cam_pad
    for (i, &v) in vp.iter().enumerate() { u.set_index(56 + i as u32, v); }
    for (i, &v) in iv.iter().enumerate() { u.set_index(72 + i as u32, v); }
}

// ── Math helpers ─────────────────────────────────────────────────────────────

/// Column-major orthographic matrix: screen pixels → WebGPU NDC.
/// Maps x ∈ [0, w] → [-1, +1] and y ∈ [0, h] → [+1, -1] (Y-flip).
fn ortho_vp(w: f32, h: f32) -> [f32; 16] {
    [
         2.0 / w,   0.0,      0.0, 0.0,
         0.0,      -2.0 / h,  0.0, 0.0,
         0.0,       0.0,      1.0, 0.0,
        -1.0,       1.0,      0.0, 1.0,
    ]
}

/// Column-major inverse of [`ortho_vp`]: NDC → screen pixels.
fn ortho_inv_vp(w: f32, h: f32) -> [f32; 16] {
    [
        w / 2.0,  0.0,       0.0, 0.0,
        0.0,     -h / 2.0,   0.0, 0.0,
        0.0,      0.0,       1.0, 0.0,
        w / 2.0,  h / 2.0,   0.0, 1.0,
    ]
}
