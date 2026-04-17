//! WgpuOverlay — full-screen GPU compositor component.
//!
//! Mounts as a `use_effect` side-effect in Dioxus web (WASM) builds.
//! On mount the overlay:
//!
//! 1. Acquires the `#webgpu-canvas` element already present in [`ViewerShell`].
//! 2. Scans `#ui-root` children via `document.querySelectorAll` collecting
//!    their `getBoundingClientRect` values.
//! 3. Uploads those rects into a GPU storage buffer (element buffer).
//! 4. Renders a full-screen WebGPU 2-D pass applying glass/blur effects on
//!    the captured element regions with background smoke, CRT, and vignette
//!    post-processing.
//! 5. Runs a GPU compute particle simulation (sparks, embers, angelic beams,
//!    glitter) every animation frame via `requestAnimationFrame`.
//!
//! Effect settings are persisted per-theme name to `localStorage`.
//!
//! Non-WASM builds compile to an empty no-op component.
use dioxus::prelude::*;

// ── Embedded WGSL shaders ─────────────────────────────────────────────────────
// Concatenation order must match the TypeScript reference (gpu-init.ts):
//   palette → types → noise → [particle_shading] → pipeline-specific
//
// Files live in src/effects/shaders/ and are copied verbatim from the
// TypeScript reference at tools/viewer/log-viewer/frontend/src/{effects,
// components/WgpuOverlay}/.
const PALETTE_WGSL:          &str = include_str!("shaders/palette.wgsl");
const TYPES_WGSL:            &str = include_str!("shaders/types.wgsl");
const NOISE_WGSL:            &str = include_str!("shaders/noise.wgsl");
const PARTICLE_SHADING_WGSL: &str = include_str!("shaders/particle_shading.wgsl");
const BACKGROUND_WGSL:       &str = include_str!("shaders/background.wgsl");
const PARTICLES_WGSL:        &str = include_str!("shaders/particles.wgsl");
const COMPUTE_WGSL:          &str = include_str!("shaders/compute.wgsl");

// ── Constants (mirror element-types.ts) ──────────────────────────────────────

/// `f32` values per element rect in the storage buffer: `[x, y, w, h, hue, kind, depth, _pad]`.
const ELEM_FLOATS: usize  = 8;
const ELEM_BYTES: usize   = ELEM_FLOATS * 4; // 32 bytes — 16-byte aligned

/// Total number of particles simulated by the compute shader.
const NUM_PARTICLES: usize = 640;

/// `f32` values per particle (48 bytes, vec3f alignment-padded).
const PARTICLE_FLOATS: usize  = 12;
const PARTICLE_BUF_SIZE: usize = NUM_PARTICLES * PARTICLE_FLOATS * 4;

/// Compute shader workgroup size (must match `compute.wgsl`).
const COMPUTE_WORKGROUP: usize = 64;

/// Palette uniform: 24 × `vec4f` = 384 bytes.
const PALETTE_VEC4_COUNT: usize = 24;
const PALETTE_BYTE_SIZE: usize  = PALETTE_VEC4_COUNT * 16;

/// Uniforms buffer: 88 × `f32` = 352 bytes (matches `types.wgsl` `Uniforms` struct).
const UNIFORMS_F32_COUNT: usize = 88;
const UNIFORMS_BYTE_SIZE: usize = UNIFORMS_F32_COUNT * 4;

/// Initial element buffer capacity (doubles dynamically on overflow).
const INITIAL_ELEM_CAP: usize = 128;

// ── WebGPU buffer usage flags (match GPUBufferUsage JS constants) ─────────────
const USAGE_UNIFORM:  u32 = 0x0040;
const USAGE_STORAGE:  u32 = 0x0080;
const USAGE_COPY_DST: u32 = 0x0008;

// ── CSS selectors and shader kind codes ──────────────────────────────────────
/// Each entry is `(selector, kind)`.  The hue is `idx / len` so shaders get
/// a stable per-type tint without requiring a palette lookup per element.
const UI_SELECTORS: &[(&str, u32)] = &[
    // Structural regions → kind 0
    (".header",              0),
    (".sidebar",             0),
    (".tab-bar",             0),
    (".filter-panel",        0),
    (".view-container",      0),
    (".log-list",            0),
    (".code-viewer",         0),
    // Per-severity log entries → kinds 1-4
    (".log-entry.level-error", 1),
    (".log-entry.level-warn",  2),
    (".log-entry.level-info",  3),
    (".log-entry.level-debug", 4),
    (".log-entry.level-trace", 4),
    // Interactive states → kinds 5-7
    (".log-entry.span-highlighted", 5),
    (".log-entry.selected",         6),
    (".log-entry.panic-entry",      7),
];

/// localStorage key under which effect settings are persisted per theme.
const STORAGE_KEY_PREFIX: &str = "viewer-api-effects-";

// ═════════════════════════════════════════════════════════════════════════════
// Public Dioxus component
// ═════════════════════════════════════════════════════════════════════════════

/// Full-screen WebGPU compositor component.
///
/// Mount this **inside** [`ViewerShell`] so it acquires `#webgpu-canvas` and
/// renders behind the `#ui-root` HTML overlay.  The component itself renders
/// nothing — it is purely a [`use_effect`] side-effect that spawns a GPU
/// render loop.
///
/// ```rust
/// rsx! {
///     ViewerShell {
///         WgpuOverlay {}
///         MyAppContent {}
///     }
/// }
/// ```
///
/// Non-WASM builds (native, test, docs) compile to an empty no-op.
#[component]
pub fn WgpuOverlay() -> Element {
    #[cfg(target_arch = "wasm32")]
    wasm_impl::mount_overlay();

    rsx! {}
}

// ═════════════════════════════════════════════════════════════════════════════
// WASM-only implementation
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(target_arch = "wasm32")]
mod wasm_impl {
    use super::*;
    use std::{cell::{Cell, RefCell}, rc::Rc};
    use js_sys::{Array, Float32Array, Function, Object, Promise, Reflect};
    use wasm_bindgen::{closure::Closure, JsCast, JsValue};
    use wasm_bindgen_futures::{JsFuture, spawn_local};
    use web_sys::{Document, Element, HtmlCanvasElement, NodeList, Window};

    // ── GPU state ─────────────────────────────────────────────────────────────

    struct GpuBuffers {
        uniform_buf:   JsValue,
        elem_buf:      JsValue,
        particle_buf:  JsValue,
        palette_buf:   JsValue,
        elem_capacity: usize,
    }

    struct GpuPipelines {
        bg_pipeline:       JsValue,
        particle_pipeline: JsValue,
        compute_pipeline:  JsValue,
        compute_bgl:       JsValue,
        render_bgl:        JsValue,
    }

    struct GpuCtx {
        device:        JsValue,
        queue:         JsValue,
        /// `GPUCanvasContext` wrapping the `#webgpu-canvas` element.
        context:       JsValue,
        /// Preferred swap-chain format string (e.g. `"bgra8unorm"`).
        format:        JsValue,
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
        /// Timestamp of the first frame (milliseconds from `performance.now`).
        start_time_ms: f64,
        /// Timestamp of the previous frame (milliseconds).
        prev_time_ms:  f64,
        /// Effect settings loaded from localStorage on init.
        settings:      EffectSettings,

        // ── DOM capture texture ───────────────────────────────────────────────
        // Each frame an async task rasterises `#ui-root` via SVG foreignObject
        // and uploads the result here so shaders can sample it for glass/blur
        // compositing effects.
        dom_tex:      JsValue,  // GPUTexture (rgba8unorm)
        dom_tex_view: JsValue,  // GPUTextureView
        dom_sam:      JsValue,  // GPUSampler (linear filtering)
        dom_tex_w:    u32,
        dom_tex_h:    u32,
        /// JS closure: `(uiRoot, width, height) -> Promise<ImageBitmap|null>`.
        capture_fn:   JsValue,
        /// JS closure: `(queue, tex, bitmap, w, h)` — wraps copyExternalImageToTexture.
        copy_fn:      JsValue,
        /// Completed bitmap from the most recent async capture task.
        pending_dom_bitmap: Rc<RefCell<Option<JsValue>>>,
        /// True while an async capture task is outstanding.
        dom_capture_busy:   Rc<Cell<bool>>,
    }

    /// Per-theme effect settings persisted to `localStorage`.
    #[derive(Clone)]
    struct EffectSettings {
        smoke_intensity:  f32,
        crt_scanlines_h:  f32,
        crt_edge_shadow:  f32,
        grain_intensity:  f32,
        vignette_str:     f32,
        particles_enabled: bool,
    }

    impl Default for EffectSettings {
        fn default() -> Self {
            Self {
                smoke_intensity:  0.6,
                crt_scanlines_h:  0.15,
                crt_edge_shadow:  0.4,
                grain_intensity:  0.15,
                vignette_str:     0.5,
                particles_enabled: true,
            }
        }
    }

    impl EffectSettings {
        /// Load from `localStorage` for the active theme, falling back to defaults.
        fn load(theme_key: &str) -> Self {
            let key = format!("{}{}", STORAGE_KEY_PREFIX, theme_key);
            let json = web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
                .and_then(|s| s.get_item(&key).ok().flatten());
            if let Some(j) = json {
                // Simple hand-rolled parse — avoids serde dependency.
                let mut s = Self::default();
                for pair in j.trim_matches(['{', '}'].as_slice()).split(',') {
                    let mut parts = pair.splitn(2, ':');
                    let k = parts.next().unwrap_or("").trim().trim_matches('"');
                    let v = parts.next().unwrap_or("").trim();
                    match k {
                        "smoke_intensity"  => { if let Ok(f) = v.parse() { s.smoke_intensity  = f; } }
                        "crt_scanlines_h"  => { if let Ok(f) = v.parse() { s.crt_scanlines_h  = f; } }
                        "crt_edge_shadow"  => { if let Ok(f) = v.parse() { s.crt_edge_shadow  = f; } }
                        "grain_intensity"  => { if let Ok(f) = v.parse() { s.grain_intensity  = f; } }
                        "vignette_str"     => { if let Ok(f) = v.parse() { s.vignette_str     = f; } }
                        "particles_enabled" => { s.particles_enabled = v == "true"; }
                        _ => {}
                    }
                }
                s
            } else {
                Self::default()
            }
        }

        fn save(&self, theme_key: &str) {
            let key  = format!("{}{}", STORAGE_KEY_PREFIX, theme_key);
            let json = format!(
                r#"{{"smoke_intensity":{},\
"crt_scanlines_h":{},\
"crt_edge_shadow":{},\
"grain_intensity":{},\
"vignette_str":{},\
"particles_enabled":{}}}"#,
                self.smoke_intensity,
                self.crt_scanlines_h,
                self.crt_edge_shadow,
                self.grain_intensity,
                self.vignette_str,
                self.particles_enabled,
            );
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
            {
                let _ = storage.set_item(&key, &json);
            }
        }
    }

    type SharedCtx = Rc<RefCell<Option<GpuCtx>>>;

    // ── Hook entrypoint ───────────────────────────────────────────────────────

    /// Called from [`WgpuOverlay`] on every render (Dioxus calls component
    /// functions on every re-render, but the hooks below are initialised only
    /// once per component lifetime).
    pub(super) fn mount_overlay() {
        let ctx: SharedCtx = use_hook(|| Rc::new(RefCell::new(None::<GpuCtx>)));
        // Flag set to `false` by `use_drop` to stop the RAF loop.
        let keep_running:   Rc<Cell<bool>> = use_hook(|| Rc::new(Cell::new(true)));
        // Pending RAF ID — cancelled in `use_drop`.
        let raf_id:         Rc<Cell<i32>>  = use_hook(|| Rc::new(Cell::new(0i32)));
        // Stores the RAF closure JsValue (keeps it alive in the JS GC).
        let raf_closure_jv: Rc<RefCell<Option<JsValue>>> =
            use_hook(|| Rc::new(RefCell::new(None::<JsValue>)));
        // Guard: GPU init runs once; prevents re-spawn on Dioxus re-renders.
        let initialized: Rc<Cell<bool>> = use_hook(|| Rc::new(Cell::new(false)));

        // ── Cleanup when component unmounts ──────────────────────────────────
        {
            let kr  = Rc::clone(&keep_running);
            let ri  = Rc::clone(&raf_id);
            let rjv = Rc::clone(&raf_closure_jv);
            let ctx_drop = Rc::clone(&ctx);
            use_drop(move || {
                kr.set(false);
                let id = ri.get();
                if id != 0 {
                    if let Some(w) = web_sys::window() {
                        let _ = w.cancel_animation_frame(id);
                    }
                }
                // Drop JsValue → JS GC can free the closure function object.
                *rjv.borrow_mut() = None;
                // Release GPU resources.
                *ctx_drop.borrow_mut() = None;
            });
        }

        // ── One-time GPU bootstrap ────────────────────────────────────────────
        {
            let init_flag   = Rc::clone(&initialized);
            let ctx_ref     = Rc::clone(&ctx);
            let kr_ref      = Rc::clone(&keep_running);
            let ri_ref      = Rc::clone(&raf_id);
            let rjv_ref     = Rc::clone(&raf_closure_jv);

            use_effect(move || {
                if init_flag.get() { return; }
                init_flag.set(true);

                // Clone again — the async block inside `spawn` can only capture
                // by move once, so we need fresh `Rc` clones here.
                let ctx_e  = Rc::clone(&ctx_ref);
                let kr_e   = Rc::clone(&kr_ref);
                let ri_e   = Rc::clone(&ri_ref);
                let rjv_e  = Rc::clone(&rjv_ref);

                spawn(async move {
                    match init_gpu().await {
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

    // ── RAF loop setup ────────────────────────────────────────────────────────

    /// Create a single persistent `requestAnimationFrame` closure and kick
    /// off the render loop.  The closure self-re-schedules until `keep_running`
    /// is set to `false` by `use_drop`.
    fn setup_raf_loop(
        ctx:         SharedCtx,
        keep_running: Rc<Cell<bool>>,
        raf_id:      Rc<Cell<i32>>,
        raf_jv:      Rc<RefCell<Option<JsValue>>>,
    ) {
        let ctx_loop        = Rc::clone(&ctx);
        let kr_loop         = Rc::clone(&keep_running);
        let ri_loop         = Rc::clone(&raf_id);
        let raf_jv_loop     = Rc::clone(&raf_jv);

        // The closure captures `raf_jv_loop` to obtain its own JsValue for
        // re-scheduling.  Ownership is transferred via `into_js_value()` so
        // the JS GC manages the function lifetime.
        let closure = Closure::<dyn FnMut(f64)>::new(move |ts_ms: f64| {
            if !kr_loop.get() { return; }
            if let Some(win) = web_sys::window() {
                if let Some(gpu) = ctx_loop.borrow_mut().as_mut() {
                    render_frame(gpu, ts_ms, &win);
                }
                // Re-schedule using the stored JsValue reference.
                if let Some(ref jv) = *raf_jv_loop.borrow() {
                    match win.request_animation_frame(jv.unchecked_ref()) {
                        Ok(id) => ri_loop.set(id),
                        Err(_) => {}
                    }
                }
            }
        });

        // Transfer closure ownership to JS GC and store the JsValue handle.
        let jv = closure.into_js_value();
        if let Some(win) = web_sys::window() {
            match win.request_animation_frame(jv.unchecked_ref()) {
                Ok(id) => raf_id.set(id),
                Err(_) => {}
            }
        }
        *raf_jv.borrow_mut() = Some(jv);
    }

    // ── GPU initialisation ────────────────────────────────────────────────────

    async fn init_gpu() -> Option<GpuCtx> {
        let win  = web_sys::window()?;
        let doc  = win.document()?;
        let perf = win.performance()?;

        // ── Acquire canvas ────────────────────────────────────────────────────
        let canvas = doc
            .get_element_by_id("webgpu-canvas")?
            .dyn_into::<HtmlCanvasElement>()
            .ok()?;

        // Size the backing buffer to the physical pixel size.
        let dpr = win.device_pixel_ratio();
        let init_w = ((canvas.client_width()  as f64 * dpr) as u32).max(1);
        let init_h = ((canvas.client_height() as f64 * dpr) as u32).max(1);
        canvas.set_width(init_w);
        canvas.set_height(init_h);

        // ── navigator.gpu ─────────────────────────────────────────────────────
        let navigator = win.navigator();
        let gpu_js = Reflect::get(&navigator, &"gpu".into()).ok()?;
        if gpu_js.is_undefined() || gpu_js.is_null() {
            web_sys::console::warn_1(&"[WgpuOverlay] navigator.gpu unavailable".into());
            return None;
        }

        // ── requestAdapter() ─────────────────────────────────────────────────
        let request_adapter: Function =
            get_fn(&gpu_js, "requestAdapter")?;
        let adapter_promise: Promise =
            request_adapter.call0(&gpu_js).ok()?.dyn_into().ok()?;
        let adapter = JsFuture::from(adapter_promise).await.ok()?;
        if adapter.is_null() || adapter.is_undefined() { return None; }

        // ── requestDevice() ──────────────────────────────────────────────────
        let request_device: Function = get_fn(&adapter, "requestDevice")?;
        let device_promise: Promise =
            request_device.call0(&adapter).ok()?.dyn_into().ok()?;
        let device = JsFuture::from(device_promise).await.ok()?;
        if device.is_null() || device.is_undefined() { return None; }

        // ── device.queue ─────────────────────────────────────────────────────
        let queue = Reflect::get(&device, &"queue".into()).ok()?;

        // ── Canvas WebGPU context ─────────────────────────────────────────────
        let context: JsValue = canvas
            .get_context("webgpu")
            .ok()??
            .into();

        let format: JsValue = get_fn(&gpu_js, "getPreferredCanvasFormat")?
            .call0(&gpu_js)
            .ok()?;

        // Configure the canvas context.
        let cfg = Object::new();
        set_prop(&cfg, "device", &device);
        set_prop(&cfg, "format", &format);
        // "premultiplied" tells the browser compositor that transparent canvas
        // pixels reveal the DOM elements below — required for GPU overlay mode.
        set_prop(&cfg, "alphaMode", &"premultiplied".into());
        get_fn(&context, "configure")?.call1(&context, &cfg).ok()?;

        // ── Shaders ───────────────────────────────────────────────────────────
        let shared_code  = format!("{}\n{}\n{}\n", PALETTE_WGSL, TYPES_WGSL, NOISE_WGSL);
        let render_shared = format!("{}{}\n", shared_code, PARTICLE_SHADING_WGSL);

        let bg_shader       = create_shader(&device, "background", &format!("{}{}", render_shared, BACKGROUND_WGSL))?;
        let particle_shader = create_shader(&device, "particles",  &format!("{}{}", render_shared, PARTICLES_WGSL))?;
        let compute_shader  = create_shader(&device, "compute",    &format!("{}{}", shared_code,   COMPUTE_WGSL))?;

        // ── Bind group layouts ────────────────────────────────────────────────
        //
        // Compute BGL — bindings match compute.wgsl + types.wgsl:
        //   0: uniform  (Uniforms)
        //   1: read-only-storage  (array<ElemRect>)
        //   2: storage            (array<Particle>, read_write)
        //   3: uniform  (ThemePalette)
        let compute_bgl = {
            let entries = Array::new();
            entries.push(&bgl_buf(0, 4, "uniform"));           // Compute only
            entries.push(&bgl_buf(1, 4, "read-only-storage")); // Compute only
            entries.push(&bgl_buf(2, 4, "storage"));           // Compute only (r/w)
            entries.push(&bgl_buf(3, 4, "uniform"));           // Compute only (palette)
            create_bgl(&device, &entries)?
        };

        // Render BGL — bindings match background.wgsl + particles.wgsl:
        //   0: uniform  (Uniforms)            — vertex + fragment
        //   1: read-only-storage (ElemRect[]) — vertex + fragment
        //   2: read-only-storage (Particle[]) — vertex + fragment
        //   3: uniform  (ThemePalette)        — fragment
        //   4: sampler  (dom_sam)             — fragment
        //   5: texture_2d (dom_tex)           — fragment
        let render_bgl = {
            let entries = Array::new();
            entries.push(&bgl_buf(0, 3, "uniform"));           // VERTEX|FRAGMENT
            entries.push(&bgl_buf(1, 3, "read-only-storage")); // VERTEX|FRAGMENT
            entries.push(&bgl_buf(2, 3, "read-only-storage")); // VERTEX|FRAGMENT
            entries.push(&bgl_buf(3, 2, "uniform"));           // FRAGMENT
            entries.push(&bgl_sampler(4, 2));                  // FRAGMENT — DOM sampler
            entries.push(&bgl_texture(5, 2));                  // FRAGMENT — DOM texture
            create_bgl(&device, &entries)?
        };

        // ── Pipeline layouts ──────────────────────────────────────────────────
        let compute_layout = create_pipeline_layout(&device, &[&compute_bgl])?;
        let render_layout  = create_pipeline_layout(&device, &[&render_bgl])?;

        // ── Pipelines ─────────────────────────────────────────────────────────
        let compute_pipeline  = create_compute_pipeline(&device, &compute_layout, &compute_shader)?;
        let bg_pipeline       = create_render_pipeline(
            &device, &render_layout, &bg_shader,       &bg_shader,       &format, false)?;
        let particle_pipeline = create_render_pipeline(
            &device, &render_layout, &particle_shader, &particle_shader, &format, true)?;

        // ── Buffers ───────────────────────────────────────────────────────────
        let uniform_buf  = gpu_buffer(&device, UNIFORMS_BYTE_SIZE as u32,
                                      USAGE_UNIFORM  | USAGE_COPY_DST)?;
        let elem_buf     = gpu_buffer(&device, (INITIAL_ELEM_CAP * ELEM_BYTES) as u32,
                                      USAGE_STORAGE  | USAGE_COPY_DST)?;
        let particle_buf = gpu_buffer(&device, PARTICLE_BUF_SIZE as u32,
                                      USAGE_STORAGE  | USAGE_COPY_DST)?;
        let palette_buf  = gpu_buffer(&device, PALETTE_BYTE_SIZE as u32,
                                      USAGE_UNIFORM  | USAGE_COPY_DST)?;

        // Zero-initialise the particle buffer so all particles start dead.
        {
            let zeros = Float32Array::new_with_length((NUM_PARTICLES * PARTICLE_FLOATS) as u32);
            queue_write_f32(&queue, &particle_buf, 0, &zeros);
        }

        // Upload a default dark-theme palette.
        queue_write_f32(&queue, &palette_buf, 0, &default_palette_f32());

        let buffers = GpuBuffers {
            uniform_buf,
            elem_buf,
            particle_buf,
            palette_buf,
            elem_capacity: INITIAL_ELEM_CAP,
        };

        // ── Bind groups ───────────────────────────────────────────────────────
        // DOM capture texture — created before bind groups so bind groups can
        // include the texture view from the start.
        let dom_sam                   = create_sampler(&device)?;
        let (dom_tex, dom_tex_view)   = create_texture_2d_with_view(&device, init_w, init_h)?;

        // JS helper: rasterise #ui-root to an ImageBitmap via SVG foreignObject.
        // Returns a Promise that resolves to an ImageBitmap or null.
        let capture_fn = js_sys::Function::new_with_args(
            "uiRoot, width, height",
            r#"
if (!uiRoot) return Promise.resolve(null);
try {
    var html = uiRoot.outerHTML;
    var svgStr = '<svg xmlns="http://www.w3.org/2000/svg" width="' + width + '" height="' + height + '">'
        + '<foreignObject width="100%" height="100%">'
        + '<div xmlns="http://www.w3.org/1999/xhtml">' + html + '</div>'
        + '</foreignObject></svg>';
    var blob = new Blob([svgStr], {type: 'image/svg+xml;charset=utf-8'});
    var url = URL.createObjectURL(blob);
    return new Promise(function(resolve) {
        var img = new Image();
        img.onload = function() {
            URL.revokeObjectURL(url);
            try {
                var oc = new OffscreenCanvas(width, height);
                var ctx2d = oc.getContext('2d');
                ctx2d.drawImage(img, 0, 0);
                createImageBitmap(oc)
                    .then(function(bm) { resolve(bm); })
                    .catch(function() { resolve(null); });
            } catch(e) { resolve(null); }
        };
        img.onerror = function() { URL.revokeObjectURL(url); resolve(null); };
        img.src = url;
    });
} catch(e) { return Promise.resolve(null); }
            "#,
        );
        // JS helper: upload an ImageBitmap into a GPUTexture.
        let copy_fn = js_sys::Function::new_with_args(
            "queue, tex, bitmap, w, h",
            "try { queue.copyExternalImageToTexture(\
                {source:bitmap,flipY:false},{texture:tex},[w,h,1]); } catch(e) {}",
        );

        let compute_bg = mk_bind_group(&device, &compute_bgl, &buffers)?;
        let render_bg  = mk_render_bind_group(&device, &render_bgl, &buffers, &dom_sam, &dom_tex_view)?;

        // ── Depth texture ─────────────────────────────────────────────────────
        let (depth_tex, depth_view) = create_depth_texture(&device, init_w, init_h)?;

        let settings   = EffectSettings::load("default");
        let now        = perf.now();
        let uniforms   = Float32Array::new_with_length(UNIFORMS_F32_COUNT as u32);

        Some(GpuCtx {
            device,
            queue,
            context,
            format,
            pipelines: GpuPipelines {
                bg_pipeline,
                particle_pipeline,
                compute_pipeline,
                compute_bgl,
                render_bgl,
            },
            buffers,
            compute_bg,
            render_bg,
            depth_tex,
            depth_view,
            depth_w: init_w,
            depth_h: init_h,
            uniforms_f32: uniforms,
            start_time_ms: now,
            prev_time_ms:  now,
            settings,
            dom_tex,
            dom_tex_view,
            dom_sam,
            dom_tex_w: init_w,
            dom_tex_h: init_h,
            capture_fn: capture_fn.into(),
            copy_fn: copy_fn.into(),
            pending_dom_bitmap: Rc::new(RefCell::new(None)),
            dom_capture_busy:   Rc::new(Cell::new(false)),
        })
    }

    // ── Per-frame render ──────────────────────────────────────────────────────

    fn render_frame(gpu: &mut GpuCtx, ts_ms: f64, win: &Window) {
        let dt_s   = ((ts_ms - gpu.prev_time_ms) / 1000.0).min(0.1) as f32;
        gpu.prev_time_ms = ts_ms;
        let time_s = ((ts_ms - gpu.start_time_ms) / 1000.0) as f32;

        // ── Resize canvas to device pixels ────────────────────────────────────
        let doc = match win.document() { Some(d) => d, None => return };
        let canvas = match doc
            .get_element_by_id("webgpu-canvas")
            .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
        {
            Some(c) => c,
            None    => return,
        };
        let dpr  = win.device_pixel_ratio();
        let cw   = ((canvas.client_width()  as f64 * dpr) as u32).max(1);
        let ch   = ((canvas.client_height() as f64 * dpr) as u32).max(1);
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

        // ── DOM capture texture resize + pending bitmap upload ─────────────────
        // If the canvas was resized, recreate the dom_tex to match and rebuild
        // the render bind group (texture view pointer changed).
        if cw != gpu.dom_tex_w || ch != gpu.dom_tex_h {
            if let Some((nt, nv)) = create_texture_2d_with_view(&gpu.device, cw, ch) {
                gpu.dom_tex      = nt;
                gpu.dom_tex_view = nv;
                gpu.dom_tex_w    = cw;
                gpu.dom_tex_h    = ch;
                if let Some(rb) = mk_render_bind_group(
                    &gpu.device, &gpu.pipelines.render_bgl, &gpu.buffers,
                    &gpu.dom_sam, &gpu.dom_tex_view,
                ) {
                    gpu.render_bg = rb;
                }
            }
        }

        // If an async DOM capture just completed, upload the bitmap.
        if let Some(bitmap) = gpu.pending_dom_bitmap.borrow_mut().take() {
            if cw == gpu.dom_tex_w && ch == gpu.dom_tex_h {
                if let Some(copy_fn) = gpu.copy_fn.dyn_ref::<Function>() {
                    let args = Array::new();
                    args.push(&gpu.queue);
                    args.push(&gpu.dom_tex);
                    args.push(&bitmap);
                    args.push(&(cw as f64).into());
                    args.push(&(ch as f64).into());
                    let _ = copy_fn.apply(&JsValue::NULL, &args);
                }
            }
            // else: canvas resized between capture start and completion — discard;
            // the next frame will trigger a new capture at the updated size.
        }

        // Schedule the next async DOM capture if none is already in flight.
        // The capture is fire-and-forget; the result lands in `pending_dom_bitmap`
        // and is uploaded at the start of the NEXT render frame.
        if !gpu.dom_capture_busy.get() {
            if let Some(ui_root_el) = doc.get_element_by_id("ui-root") {
                gpu.dom_capture_busy.set(true);
                let pb      = Rc::clone(&gpu.pending_dom_bitmap);
                let cb      = Rc::clone(&gpu.dom_capture_busy);
                let cap_fn: Function = gpu.capture_fn.clone().unchecked_into();
                let ui_root_js = JsValue::from(ui_root_el);
                let promise_result = cap_fn.call3(
                    &JsValue::NULL,
                    &ui_root_js,
                    &(cw as f64).into(),
                    &(ch as f64).into(),
                );
                match promise_result.ok().and_then(|v| v.dyn_into::<Promise>().ok()) {
                    Some(promise) => {
                        let fut = JsFuture::from(promise);
                        spawn_local(async move {
                            if let Ok(bm_val) = fut.await {
                                if !bm_val.is_null() && !bm_val.is_undefined() {
                                    *pb.borrow_mut() = Some(bm_val);
                                }
                            }
                            cb.set(false);
                        });
                    }
                    None => { gpu.dom_capture_busy.set(false); }
                }
            }
        }

        // ── DOM element scan ──────────────────────────────────────────────────
        let (elem_data, elem_count) = scan_ui_rects(&doc);

        // Grow element buffer if scanned count exceeded capacity.
        if elem_count > gpu.buffers.elem_capacity {
            let new_cap = (elem_count * 2).max(INITIAL_ELEM_CAP);
            if let Some(new_buf) = gpu_buffer(
                &gpu.device,
                (new_cap * ELEM_BYTES) as u32,
                USAGE_STORAGE | USAGE_COPY_DST,
            ) {
                gpu.buffers.elem_buf      = new_buf;
                gpu.buffers.elem_capacity = new_cap;
                // Rebuild bind groups — buffer pointer changed.
                if let Some(cb) = mk_bind_group(&gpu.device, &gpu.pipelines.compute_bgl, &gpu.buffers) {
                    gpu.compute_bg = cb;
                }
                if let Some(rb) = mk_render_bind_group(
                    &gpu.device, &gpu.pipelines.render_bgl, &gpu.buffers,
                    &gpu.dom_sam, &gpu.dom_tex_view,
                ) {
                    gpu.render_bg = rb;
                }
            }
        }

        // Upload element rects.
        if !elem_data.is_empty() {
            // SAFETY: `elem_data` lives for the duration of this call; the
            // view is consumed before this function returns.
            let fa = unsafe { Float32Array::view(&elem_data) };
            queue_write_f32(&gpu.queue, &gpu.buffers.elem_buf, 0, &fa);
        }

        // ── Pack uniforms ─────────────────────────────────────────────────────
        {
            let u  = &gpu.uniforms_f32;
            let s  = &gpu.settings;
            let vp = ortho_vp(cw as f32, ch as f32);
            let iv = ortho_inv_vp(cw as f32, ch as f32);

            // Scalars [0..55]
            u.set_index(0,  time_s);
            u.set_index(1,  cw as f32);
            u.set_index(2,  ch as f32);
            u.set_index(3,  elem_count as f32);
            u.set_index(4,  -9999.0); // mouse_x (not tracked yet)
            u.set_index(5,  -9999.0); // mouse_y
            u.set_index(6,  dt_s);
            u.set_index(7,  -1.0);  // hover_elem
            u.set_index(8,  0.0);   // hover_start_time
            u.set_index(9,  -1.0);  // selected_elem
            u.set_index(10, s.crt_scanlines_h);
            u.set_index(11, 0.0);   // crt_scanlines_v
            u.set_index(12, s.crt_edge_shadow);
            u.set_index(13, 0.08);  // crt_flicker
            u.set_index(14, 0.3);   // crt_line_width
            u.set_index(15, s.smoke_intensity);
            u.set_index(16, 1.0);   // smoke_speed
            u.set_index(17, 1.0);   // smoke_warm_scale
            u.set_index(18, 1.0);   // smoke_cool_scale
            u.set_index(19, 1.0);   // smoke_moss_scale
            u.set_index(20, s.grain_intensity);
            u.set_index(21, 0.5);   // grain_coarseness
            u.set_index(22, 0.3);   // grain_size
            u.set_index(23, s.vignette_str);
            u.set_index(24, 0.2);   // underglow_str
            u.set_index(25, 1.0);   // spark_speed
            u.set_index(26, 1.0);   // ember_speed
            u.set_index(27, 1.0);   // beam_speed
            u.set_index(28, 1.0);   // glitter_speed
            u.set_index(29, 35.0);  // beam_height
            u.set_index(30, 0.0);   // beam_count (0 = all slots)
            u.set_index(31, 1.0);   // beam_drift
            u.set_index(32, 0.0);   // scroll_dx
            u.set_index(33, 0.0);   // scroll_dy
            u.set_index(34, if s.particles_enabled { 1.0 } else { 0.0 }); // spark_count
            u.set_index(35, 1.0);   // spark_size
            u.set_index(36, if s.particles_enabled { 1.0 } else { 0.0 }); // ember_count
            u.set_index(37, 1.0);   // ember_size
            u.set_index(38, if s.particles_enabled { 1.0 } else { 0.0 }); // glitter_count
            u.set_index(39, 1.0);   // glitter_size
            u.set_index(40, 1.0);   // cinder_size
            u.set_index(41, 0.0);   // ref_depth
            u.set_index(42, 1.0);   // world_scale
            u.set_index(43, 0.0);   // vp_x
            u.set_index(44, 0.0);   // vp_y
            u.set_index(45, cw as f32); // vp_w
            u.set_index(46, ch as f32); // vp_h
            u.set_index(47, 0.0);   // current_view (0 = logs)
            // CRT scanline tint (warm amber)
            u.set_index(48, 0.9);   // crt_color_r
            u.set_index(49, 0.7);   // crt_color_g
            u.set_index(50, 0.4);   // crt_color_b
            u.set_index(51, 0.0);   // _crt_pad
            // Camera position (2D — unused)
            u.set_index(52, 0.0);
            u.set_index(53, 0.0);
            u.set_index(54, 0.0);
            u.set_index(55, 0.0);   // _cam_pad
            // particle_vp — orthographic mat4x4 [56..71]
            for (i, &v) in vp.iter().enumerate() { u.set_index(56 + i as u32, v); }
            // particle_inv_vp [72..87]
            for (i, &v) in iv.iter().enumerate() { u.set_index(72 + i as u32, v); }

            queue_write_f32(&gpu.queue, &gpu.buffers.uniform_buf, 0, u);
        }

        // ── Get current frame texture ─────────────────────────────────────────
        let frame_tex  = match get_fn(&gpu.context, "getCurrentTexture")
            .and_then(|f| f.call0(&gpu.context).ok())
        {
            Some(t) => t,
            None    => return,
        };
        let frame_view = match create_tex_view(&frame_tex) {
            Some(v) => v,
            None    => return,
        };

        // ── Command encoder ───────────────────────────────────────────────────
        let encoder = match get_fn(&gpu.device, "createCommandEncoder")
            .and_then(|f| f.call0(&gpu.device).ok())
        {
            Some(e) => e,
            None    => return,
        };

        // ── Compute pass (particle physics) ───────────────────────────────────
        if gpu.settings.particles_enabled {
            if let Some(pass) = get_fn(&encoder, "beginComputePass")
                .and_then(|f| f.call0(&encoder).ok())
            {
                call_set_pipeline(&pass, "setPipeline", &gpu.pipelines.compute_pipeline);
                call_set_bind_group(&pass, 0, &gpu.compute_bg);
                let wg = ((NUM_PARTICLES + COMPUTE_WORKGROUP - 1) / COMPUTE_WORKGROUP) as u32;
                call_dispatch(&pass, wg);
                call_end(&pass);
            }
        }

        // ── Render pass ───────────────────────────────────────────────────────
        {
            let rp_desc = build_render_pass_desc(&frame_view, &gpu.depth_view);
            if let Some(pass) = get_fn(&encoder, "beginRenderPass")
                .and_then(|f| f.call1(&encoder, &rp_desc).ok())
            {
                // a) Background full-screen quad (smoke, element rects, CRT, grain, vignette)
                call_set_pipeline(&pass, "setPipeline", &gpu.pipelines.bg_pipeline);
                call_set_bind_group(&pass, 0, &gpu.render_bg);
                call_draw(&pass, 6, 1);

                // b) Particle quads (additive blending — sparks, embers, beams, glitter)
                if gpu.settings.particles_enabled {
                    call_set_pipeline(&pass, "setPipeline", &gpu.pipelines.particle_pipeline);
                    call_set_bind_group(&pass, 0, &gpu.render_bg);
                    call_draw(&pass, 6, NUM_PARTICLES as u32);
                }

                call_end(&pass);
            }
        }

        // ── Submit ────────────────────────────────────────────────────────────
        if let Some(finish) = get_fn(&encoder, "finish")
            .and_then(|f| f.call0(&encoder).ok())
        {
            if let Some(submit) = get_fn(&gpu.queue, "submit") {
                let cmds = Array::new();
                cmds.push(&finish);
                let _ = submit.call1(&gpu.queue, &cmds);
            }
        }
    }

    // ── DOM scanning ──────────────────────────────────────────────────────────

    /// Scan `#ui-root` children matching [`UI_SELECTORS`] via
    /// `querySelectorAll` and pack their `getBoundingClientRect` values into a
    /// flat `Vec<f32>` suitable for direct `writeBuffer` upload.
    ///
    /// Layout per element: `[x, y, w, h, hue, kind, depth=0, _pad=0]`
    fn scan_ui_rects(doc: &Document) -> (Vec<f32>, usize) {
        let total = UI_SELECTORS.len() as f32;
        let mut data: Vec<f32> = Vec::with_capacity(64 * ELEM_FLOATS);

        for (idx, &(selector, kind)) in UI_SELECTORS.iter().enumerate() {
            let hue: f32 = idx as f32 / total;

            let node_list: NodeList = match doc.query_selector_all(selector) {
                Ok(nl) => nl,
                Err(_) => continue,
            };

            for j in 0..node_list.length() {
                let node = match node_list.get(j) { Some(n) => n, None => continue };
                let el: Element = match node.dyn_into::<Element>() {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                // Call el.getBoundingClientRect() via Reflect to avoid needing
                // the DomRect web-sys feature.
                let rect_val = match get_fn(&el, "getBoundingClientRect")
                    .and_then(|f| f.call0(&el).ok())
                {
                    Some(r) => r,
                    None    => continue,
                };

                let x = prop_f32(&rect_val, "x");
                let y = prop_f32(&rect_val, "y");
                let w = prop_f32(&rect_val, "width");
                let h = prop_f32(&rect_val, "height");

                // Skip zero-size or off-screen elements.
                if w < 1.0 || h < 1.0 { continue; }

                data.push(x);
                data.push(y);
                data.push(w);
                data.push(h);
                data.push(hue);
                data.push(kind as f32);
                data.push(0.0); // depth — flat screen-space
                data.push(0.0); // _pad
            }
        }

        let count = data.len() / ELEM_FLOATS;
        (data, count)
    }

    // ── Default palette (dark theme) ──────────────────────────────────────────

    /// Build a default dark-theme palette `Float32Array` for the palette
    /// uniform buffer (384 bytes = 24 × vec4f).
    ///
    /// Uses hard-coded defaults matching the `DARK` preset from `theme.rs`.
    /// When theme integration is added, this can be replaced with a live lookup.
    fn default_palette_f32() -> Float32Array {
        // 24 vec4f entries = 96 f32 values.
        let buf = Float32Array::new_with_length(96);
        let mut i = 0u32;

        // Helper: write one vec4f entry.
        let mut w = |r: f32, g: f32, b: f32, a: f32| {
            buf.set_index(i,     r);
            buf.set_index(i + 1, g);
            buf.set_index(i + 2, b);
            buf.set_index(i + 3, a);
            i += 4;
        };

        // [0] spark_core  — hot white-yellow
        w(1.0,  0.97, 0.85, 1.0);
        // [1] spark_ember — outer ember glow
        w(1.0,  0.4,  0.05, 1.0);
        // [2] spark_steel — metallic highlight
        w(0.7,  0.75, 0.85, 1.0);
        // [3] ember_hot   — bright hot center
        w(1.0,  0.6,  0.1,  1.0);
        // [4] beam_center — golden-white core
        w(1.0,  0.98, 0.88, 1.0);
        // [5] beam_edge   — warm gold edge
        w(1.0,  0.78, 0.2,  1.0);
        // [6] glitter_warm — golden-white base
        w(1.0,  0.95, 0.7,  1.0);
        // [7] glitter_cool — blue-white variation
        w(0.7,  0.85, 1.0,  1.0);
        // [8]  cinder_ember — deep orange-red
        w(0.7,  0.15, 0.02, 1.0);
        // [9]  cinder_gold  — tarnished gold
        w(0.6,  0.45, 0.05, 1.0);
        // [10] cinder_ash   — cool grey
        w(0.35, 0.33, 0.32, 1.0);
        // [11] cinder_vine  — deep green
        w(0.05, 0.22, 0.05, 1.0);
        // [12] smoke_cool   — blue-grey
        w(0.08, 0.1,  0.15, 1.0);
        // [13] smoke_warm   — brown-amber
        w(0.12, 0.08, 0.03, 1.0);
        // [14] smoke_moss   — mossy mid-tone
        w(0.06, 0.09, 0.04, 1.0);
        // [15..22] kind glow colors (structural, error, warn, info, debug, span, selected, panic)
        w(0.18, 0.16, 0.14, 1.0); // kind_structural — dark stone
        w(0.97, 0.47, 0.55, 1.0); // kind_error      — red
        w(0.88, 0.68, 0.41, 1.0); // kind_warn       — amber
        w(0.48, 0.81, 0.64, 1.0); // kind_info       — green
        w(0.48, 0.60, 0.97, 1.0); // kind_debug      — blue
        w(0.61, 0.80, 0.41, 1.0); // kind_span       — bright green
        w(1.0,  0.62, 0.39, 1.0); // kind_selected   — orange
        w(0.97, 0.47, 0.55, 1.0); // kind_panic      — same base as error
        // [23] _pad
        w(0.0,  0.0,  0.0,  0.0);

        buf
    }

    // ── Math helpers ──────────────────────────────────────────────────────────

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

    // ── WebGPU wrapper helpers ─────────────────────────────────────────────────

    /// Get a `Function` property from `obj` by name, returning `None` if
    /// the property doesn't exist or is not callable.
    fn get_fn(obj: &JsValue, name: &str) -> Option<Function> {
        Reflect::get(obj, &name.into()).ok()?.dyn_into::<Function>().ok()
    }

    /// Set a property `key` on `obj` to `value` via `Reflect.set`.
    fn set_prop(obj: &Object, key: &str, value: &JsValue) {
        let _ = Reflect::set(obj, &key.into(), value);
    }

    /// Read a numeric property from `obj` and cast to `f32`.
    fn prop_f32(obj: &JsValue, key: &str) -> f32 {
        Reflect::get(obj, &key.into())
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32
    }

    fn create_shader(device: &JsValue, label: &str, code: &str) -> Option<JsValue> {
        let desc = Object::new();
        set_prop(&desc, "label", &label.into());
        set_prop(&desc, "code",  &code.into());
        get_fn(device, "createShaderModule")?.call1(device, &desc.into()).ok()
    }

    /// Build a `GPUBindGroupLayoutEntry` for a buffer binding.
    /// `visibility` is a bit-mask: 1=VERTEX, 2=FRAGMENT, 4=COMPUTE.
    fn bgl_buf(binding: u32, visibility: u32, ty: &str) -> JsValue {
        let entry = Object::new();
        set_prop(&entry, "binding",    &binding.into());
        set_prop(&entry, "visibility", &visibility.into());
        let buf = Object::new();
        set_prop(&buf, "type", &ty.into());
        set_prop(&entry, "buffer", &buf.into());
        entry.into()
    }

    fn create_bgl(device: &JsValue, entries: &Array) -> Option<JsValue> {
        let desc = Object::new();
        set_prop(&desc, "entries", &entries.into());
        get_fn(device, "createBindGroupLayout")?.call1(device, &desc.into()).ok()
    }

    fn create_pipeline_layout(device: &JsValue, bgls: &[&JsValue]) -> Option<JsValue> {
        let arr = Array::new();
        for &bgl in bgls { arr.push(bgl); }
        let desc = Object::new();
        set_prop(&desc, "bindGroupLayouts", &arr.into());
        get_fn(device, "createPipelineLayout")?.call1(device, &desc.into()).ok()
    }

    fn create_compute_pipeline(
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

    /// Build a render pipeline.  When `additive` is true the fragment stage
    /// uses `ONE + ONE` blending (particle pass); otherwise no blend state is
    /// set (opaque background pass).
    fn create_render_pipeline(
        device: &JsValue, layout: &JsValue,
        vs: &JsValue, fs: &JsValue,
        format: &JsValue, additive: bool,
    ) -> Option<JsValue> {
        let vs_state = Object::new();
        set_prop(&vs_state, "module",     vs);
        let vs_ep: JsValue = if additive { "vs_particle".into() } else { "vs_main".into() };
        set_prop(&vs_state, "entryPoint", &vs_ep);

        let color_target = Object::new();
        set_prop(&color_target, "format", format);
        if additive {
            // Additive blend: ONE + ONE (particle pass — sparks, embers, glitter).
            let bc = Object::new();
            set_prop(&bc, "srcFactor", &"one".into());
            set_prop(&bc, "dstFactor", &"one".into());
            set_prop(&bc, "operation", &"add".into());
            let blend = Object::new();
            set_prop(&blend, "color", &bc.clone().into());
            set_prop(&blend, "alpha", &bc.into());
            set_prop(&color_target, "blend", &blend.into());
        } else {
            // Alpha-over blend: standard src-alpha / one-minus-src-alpha for
            // the background pass so the GPU overlay composites correctly on
            // top of the DOM. The alpha channel uses `one / one-minus-src-alpha`
            // to produce premultiplied-compatible output for the compositor.
            let bc = Object::new();
            set_prop(&bc, "srcFactor", &"src-alpha".into());
            set_prop(&bc, "dstFactor", &"one-minus-src-alpha".into());
            set_prop(&bc, "operation", &"add".into());
            let ba = Object::new();
            set_prop(&ba, "srcFactor", &"one".into());
            set_prop(&ba, "dstFactor", &"one-minus-src-alpha".into());
            set_prop(&ba, "operation", &"add".into());
            let blend = Object::new();
            set_prop(&blend, "color", &bc.into());
            set_prop(&blend, "alpha", &ba.into());
            set_prop(&color_target, "blend", &blend.into());
        }
        let targets = Array::new();
        targets.push(&color_target.into());

        let fs_state = Object::new();
        set_prop(&fs_state, "module",  fs);
        let fs_ep: JsValue = if additive { "fs_particle".into() } else { "fs_main".into() };
        set_prop(&fs_state, "entryPoint", &fs_ep);
        set_prop(&fs_state, "targets",    &targets.into());

        let prim = Object::new();
        set_prop(&prim, "topology", &"triangle-list".into());

        let depth = Object::new();
        set_prop(&depth, "format",             &"depth24plus".into());
        set_prop(&depth, "depthWriteEnabled",   &JsValue::FALSE);
        set_prop(&depth, "depthCompare",        &"always".into());

        let desc = Object::new();
        set_prop(&desc, "layout",       layout);
        set_prop(&desc, "vertex",       &vs_state.into());
        set_prop(&desc, "fragment",     &fs_state.into());
        set_prop(&desc, "primitive",    &prim.into());
        set_prop(&desc, "depthStencil", &depth.into());

        get_fn(device, "createRenderPipeline")?.call1(device, &desc.into()).ok()
    }

    fn gpu_buffer(device: &JsValue, size: u32, usage: u32) -> Option<JsValue> {
        let desc = Object::new();
        set_prop(&desc, "size",  &(size as f64).into());
        set_prop(&desc, "usage", &usage.into());
        get_fn(device, "createBuffer")?.call1(device, &desc.into()).ok()
    }

    fn queue_write_f32(queue: &JsValue, buf: &JsValue, offset_bytes: u32, data: &Float32Array) {
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

    fn bg_binding_entry(binding: u32, resource: &JsValue) -> JsValue {
        let entry = Object::new();
        set_prop(&entry, "binding",  &binding.into());
        set_prop(&entry, "resource", resource);
        entry.into()
    }

    fn buf_resource(buf: &JsValue) -> JsValue {
        let b = Object::new();
        set_prop(&b, "buffer", buf);
        b.into()
    }

    fn mk_bind_group(device: &JsValue, layout: &JsValue, buffers: &GpuBuffers) -> Option<JsValue> {
        let entries = Array::new();
        entries.push(&bg_binding_entry(0, &buf_resource(&buffers.uniform_buf)));
        entries.push(&bg_binding_entry(1, &buf_resource(&buffers.elem_buf)));
        entries.push(&bg_binding_entry(2, &buf_resource(&buffers.particle_buf)));
        entries.push(&bg_binding_entry(3, &buf_resource(&buffers.palette_buf)));
        let desc = Object::new();
        set_prop(&desc, "layout",  layout);
        set_prop(&desc, "entries", &entries.into());
        get_fn(device, "createBindGroup")?.call1(device, &desc.into()).ok()
    }

    /// Same as [`mk_bind_group`] but adds DOM sampler (binding 4) and DOM
    /// texture view (binding 5) required by the render BGL.
    fn mk_render_bind_group(
        device: &JsValue, layout: &JsValue, buffers: &GpuBuffers,
        dom_sam: &JsValue, dom_tex_view: &JsValue,
    ) -> Option<JsValue> {
        let entries = Array::new();
        entries.push(&bg_binding_entry(0, &buf_resource(&buffers.uniform_buf)));
        entries.push(&bg_binding_entry(1, &buf_resource(&buffers.elem_buf)));
        entries.push(&bg_binding_entry(2, &buf_resource(&buffers.particle_buf)));
        entries.push(&bg_binding_entry(3, &buf_resource(&buffers.palette_buf)));
        // Binding 4: sampler resource = the sampler object itself.
        entries.push(&bg_binding_entry(4, dom_sam));
        // Binding 5: texture resource = the GPUTextureView.
        entries.push(&bg_binding_entry(5, dom_tex_view));
        let desc = Object::new();
        set_prop(&desc, "layout",  layout);
        set_prop(&desc, "entries", &entries.into());
        get_fn(device, "createBindGroup")?.call1(device, &desc.into()).ok()
    }

    fn create_tex_view(texture: &JsValue) -> Option<JsValue> {
        get_fn(texture, "createView")?.call0(texture).ok()
    }

    /// Build a `GPUBindGroupLayoutEntry` for a sampler binding.
    fn bgl_sampler(binding: u32, visibility: u32) -> JsValue {
        let entry = Object::new();
        set_prop(&entry, "binding",    &binding.into());
        set_prop(&entry, "visibility", &visibility.into());
        let sam = Object::new();
        set_prop(&sam, "type", &"filtering".into());
        set_prop(&entry, "sampler", &sam.into());
        entry.into()
    }

    /// Build a `GPUBindGroupLayoutEntry` for a 2-D float texture binding.
    fn bgl_texture(binding: u32, visibility: u32) -> JsValue {
        let entry = Object::new();
        set_prop(&entry, "binding",    &binding.into());
        set_prop(&entry, "visibility", &visibility.into());
        let tex = Object::new();
        set_prop(&tex, "sampleType",    &"float".into());
        set_prop(&tex, "viewDimension", &"2d".into());
        set_prop(&tex, "multisampled",  &JsValue::FALSE);
        set_prop(&entry, "texture", &tex.into());
        entry.into()
    }

    /// Create a linear-filter `GPUSampler`.
    fn create_sampler(device: &JsValue) -> Option<JsValue> {
        let desc = Object::new();
        set_prop(&desc, "magFilter", &"linear".into());
        set_prop(&desc, "minFilter", &"linear".into());
        get_fn(device, "createSampler")?.call1(device, &desc.into()).ok()
    }

    /// Create an `rgba8unorm` 2-D `GPUTexture` with TEXTURE_BINDING + COPY_DST
    /// usage, and return it together with a default view.
    fn create_texture_2d_with_view(device: &JsValue, w: u32, h: u32) -> Option<(JsValue, JsValue)> {
        let size = Object::new();
        set_prop(&size, "width",  &(w as f64).into());
        set_prop(&size, "height", &(h as f64).into());
        let desc = Object::new();
        set_prop(&desc, "size",   &size.into());
        set_prop(&desc, "format", &"rgba8unorm".into());
        // GPUTextureUsage.TEXTURE_BINDING = 0x04, COPY_DST = 0x08
        set_prop(&desc, "usage", &(0x04u32 | 0x08u32).into());
        let tex  = get_fn(device, "createTexture")?.call1(device, &desc.into()).ok()?;
        let view = create_tex_view(&tex)?;
        Some((tex, view))
    }

    fn create_depth_texture(device: &JsValue, w: u32, h: u32) -> Option<(JsValue, JsValue)> {
        let size = Object::new();
        set_prop(&size, "width",  &(w as f64).into());
        set_prop(&size, "height", &(h as f64).into());
        let desc = Object::new();
        set_prop(&desc, "size",   &size.into());
        set_prop(&desc, "format", &"depth24plus".into());
        // GPUTextureUsage.RENDER_ATTACHMENT = 0x10
        set_prop(&desc, "usage",  &0x10u32.into());
        let tex = get_fn(device, "createTexture")?.call1(device, &desc.into()).ok()?;
        let view = create_tex_view(&tex)?;
        Some((tex, view))
    }

    fn build_render_pass_desc(color_view: &JsValue, depth_view: &JsValue) -> JsValue {
        // Color attachment
        let ca = Object::new();
        set_prop(&ca, "view",       color_view);
        set_prop(&ca, "loadOp",     &"clear".into());
        set_prop(&ca, "storeOp",    &"store".into());
        let clear = Object::new();
        set_prop(&clear, "r", &0.0f64.into());
        set_prop(&clear, "g", &0.0f64.into());
        set_prop(&clear, "b", &0.0f64.into());
        set_prop(&clear, "a", &0.0f64.into()); // transparent — DOM shows through
        set_prop(&ca, "clearValue", &clear.into());

        // Depth attachment
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

    fn call_set_pipeline(pass: &JsValue, method: &str, pipeline: &JsValue) {
        if let Some(f) = get_fn(pass, method) { let _ = f.call1(pass, pipeline); }
    }

    fn call_set_bind_group(pass: &JsValue, index: u32, bg: &JsValue) {
        if let Some(f) = get_fn(pass, "setBindGroup") {
            let _ = f.call2(pass, &index.into(), bg);
        }
    }

    fn call_dispatch(pass: &JsValue, x: u32) {
        if let Some(f) = get_fn(pass, "dispatchWorkgroups") {
            let _ = f.call1(pass, &x.into());
        }
    }

    fn call_draw(pass: &JsValue, vertices: u32, instances: u32) {
        if let Some(f) = get_fn(pass, "draw") {
            let _ = f.call2(pass, &vertices.into(), &instances.into());
        }
    }

    fn call_end(pass: &JsValue) {
        if let Some(f) = get_fn(pass, "end") { let _ = f.call0(pass); }
    }
}
