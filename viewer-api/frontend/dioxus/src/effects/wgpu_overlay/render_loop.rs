//! Per-frame render orchestration: rAF loop, uniform packing, compute and
//! render passes.  Mirrors `gpu-render-loop.ts`.

#![cfg(target_arch = "wasm32")]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use dioxus::prelude::*;
use js_sys::Float32Array;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::MouseEvent;

use super::element_types::*;
use super::gpu_buffers::{mk_compute_bind_group, mk_render_bind_group, GpuBuffers};
use super::gpu_init::{init_gpu, GpuPipelines};
use super::settings::EffectSettings;
use super::webgpu::*;
use tracing::{info, warn};

mod frame;
mod uniforms;

use self::frame::render_frame;
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

            info!(target: "wgpu_overlay", "mount_overlay use_effect - spawning bootstrap");
            spawn(async move {
                info!(target: "wgpu_overlay", "bootstrap_ctx() starting");
                match bootstrap_ctx().await {
                    Some(gpu_ctx) => {
                        info!(target: "wgpu_overlay", "bootstrap_ctx() succeeded - starting rAF loop");
                        *ctx_e.borrow_mut() = Some(gpu_ctx);
                        setup_raf_loop(
                            Rc::clone(&ctx_e),
                            Rc::clone(&kr_e),
                            Rc::clone(&ri_e),
                            Rc::clone(&rjv_e),
                        );
                    }
                    None => {
                        warn!(target: "wgpu_overlay", "WebGPU unavailable - overlay disabled");
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

    // Seed the live effect-settings store from localStorage so previously
    // committed tweaks are restored on first paint.
    super::set_live_effects(EffectSettings::load());

    // Register a mousemove listener so the compute shader can use the
    // cursor position for spark spawning and hover detection.
    install_mouse_listener();

    let buffers    = GpuBuffers::new(&init.device, &init.queue)?;
    let compute_bg = mk_compute_bind_group(&init.device, &init.pipelines.compute_bgl, &buffers)?;
    let render_bg  = mk_render_bind_group (&init.device, &init.pipelines.render_bgl,  &buffers)?;
    let (depth_tex, depth_view) =
        create_depth_texture(&init.device, init.canvas_width, init.canvas_height)?;

    // Publish the GPU handles so secondary renderers (e.g. Graph3D) can
    // composite into the same swap-chain texture.
    super::set_shared_gpu(super::SharedGpu {
        device:  init.device.clone(),
        queue:   init.queue.clone(),
        context: init.context.clone(),
        format:  init.format.clone(),
    });

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
            let overlay_enabled = crate::effects::wgpu_overlay::is_overlay_enabled();
            if overlay_enabled {
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

// ── Mouse listener ───────────────────────────────────────────────────────────

/// Register a `mousemove` listener on `document` and store the JS closure so
/// it is never garbage-collected.  Safe to call multiple times — the second
/// call is a no-op because `store_mouse_listener` replaces the previous value.
fn install_mouse_listener() {
    let closure = Closure::<dyn FnMut(MouseEvent)>::new(|evt: MouseEvent| {
        super::set_mouse_pos(evt.client_x() as f32, evt.client_y() as f32);
    });
    if let Some(win) = web_sys::window() {
        if let Some(doc) = win.document() {
            let _ = doc.add_event_listener_with_callback(
                "mousemove",
                closure.as_ref().unchecked_ref(),
            );
        }
    }
    // Keep the closure alive — dropping it would silently unregister the listener.
    super::store_mouse_listener(closure.into_js_value());
}
