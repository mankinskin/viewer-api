//! `WgpuOverlay` — full-screen WebGPU compositor component for Dioxus web.
//!
//! Mirrors the canonical TypeScript implementation at
//! `tools/viewer/log-viewer/frontend/src/components/WgpuOverlay/`.
//!
//! ## Architecture
//!
//! - The opaque WebGPU canvas (`#webgpu-canvas`, z-index 1) sits **behind**
//!   the DOM (`#ui-root`, z-index 3) and renders smoke / particles / CRT
//!   straight onto its own surface.
//! - DOM elements compose over the canvas via the normal browser stacking
//!   context.  No DOM rasterisation pipeline is required.
//! - On mount the overlay scans `#ui-root` children matching
//!   [`element_types::UI_SELECTORS`] and uploads their bounding rects to a
//!   GPU storage buffer so shaders can render glow / underlay effects on
//!   them.
//!
//! ## Module layout
//!
//! - [`element_types`] — selectors, kind constants, packed-buffer sizes.
//! - [`webgpu`]        — JS interop helpers (Reflect-based wrappers).
//! - [`settings`]      — per-theme `EffectSettings` persisted to localStorage.
//! - [`gpu_init`]      — adapter/device/pipeline/shader factory.
//! - [`gpu_buffers`]   — buffer manager + bind-group factories.
//! - [`element_scanner`] — DOM → packed `Float32` rects.
//! - [`render_loop`]   — rAF loop, uniform packing, compute / render passes.
//!
//! Non-WASM builds compile to an empty no-op component.

use dioxus::prelude::*;

mod element_types;
pub mod settings;

#[cfg(target_arch = "wasm32")] mod webgpu;
#[cfg(target_arch = "wasm32")] mod gpu_init;
#[cfg(target_arch = "wasm32")] mod gpu_buffers;
#[cfg(target_arch = "wasm32")] mod element_scanner;
#[cfg(target_arch = "wasm32")] mod render_loop;

pub use settings::{
    hex_to_rgba, rgba_to_hex, EffectSettings, PaletteColor, PALETTE_LABELS, PALETTE_LEN,
};

// ── Shared GPU handles + frame-callback registry ────────────────────────────
//
// The `#webgpu-canvas` is configured with the overlay's `GPUDevice` exactly
// once. Other renderers (e.g. `Graph3D`) compose into the same swap-chain
// texture by registering a [`FrameCallback`] which the overlay invokes after
// its own pass each frame, sharing the same device, queue and frame view.
// This avoids cross-device validation errors and keeps the overlay running
// uninterrupted when secondary renderers come and go.

#[cfg(target_arch = "wasm32")]
use std::cell::{Cell, RefCell};
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

/// Shared GPU handles published by the overlay once its canvas has been
/// configured. Secondary renderers consume these instead of requesting their
/// own adapter/device.
#[cfg(target_arch = "wasm32")]
pub struct SharedGpu {
    pub device:  JsValue,
    pub queue:   JsValue,
    pub context: JsValue,
    /// Preferred canvas format string (e.g. `"bgra8unorm"`).
    pub format:  String,
}

/// Per-frame context handed to each registered [`FrameCallback`].
#[cfg(target_arch = "wasm32")]
pub struct FrameContext<'a> {
    pub device:    &'a JsValue,
    pub queue:     &'a JsValue,
    /// View of the swap-chain texture for this frame. Use `loadOp: "load"`
    /// when binding it as a colour attachment so the overlay's render is
    /// preserved underneath.
    pub frame_view: &'a JsValue,
    /// Canvas backing-store size in physical pixels.
    pub canvas_w:  u32,
    pub canvas_h:  u32,
    pub time_s:    f32,
}

#[cfg(target_arch = "wasm32")]
pub type FrameCallback = Box<dyn FnMut(&FrameContext)>;

#[cfg(target_arch = "wasm32")]
thread_local! {
    /// Master enable flag for the WgpuOverlay render loop. Defaults to `true`
    /// so first-load viewers render the full GPU experience immediately.
    static GPU_OVERLAY_ENABLED: Cell<bool> = const { Cell::new(true) };
    /// Live (preview-or-committed) effect settings read each frame by the
    /// render loop.  Mutated via [`set_live_effects`] from the Theme Settings
    /// UI for instant preview.
    static EFFECTS_LIVE: RefCell<EffectSettings> =
        RefCell::new(EffectSettings::default());
    /// Set whenever [`set_live_effects`] is called so the render loop knows
    /// to re-upload the palette uniform buffer on the next frame.
    static PALETTE_DIRTY: Cell<bool> = const { Cell::new(true) };

    /// Shared GPU handles, populated after the overlay's GPU bootstrap.
    static SHARED_GPU: RefCell<Option<Rc<SharedGpu>>> = const { RefCell::new(None) };
    /// Registered per-frame callbacks invoked after the overlay's own pass.
    static FRAME_CALLBACKS: RefCell<Vec<(u64, Rc<RefCell<FrameCallback>>)>> =
        const { RefCell::new(Vec::new()) };
    static NEXT_CB_ID: Cell<u64> = const { Cell::new(1) };
}

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static EFFECTS_LIVE: std::cell::RefCell<EffectSettings> =
        std::cell::RefCell::new(EffectSettings::default());
}

/// Deprecated no-op kept for source-level compatibility.
///
/// The canvas is always owned by [`WgpuOverlay`]; secondary renderers
/// composite into the same frame via [`register_frame_callback`].
#[deprecated(note = "secondary renderers should use register_frame_callback instead")]
pub fn set_gpu_canvas_owner(_taken: bool) {}

/// Enable or disable the WebGPU overlay master switch.
///
/// When disabled, the render loop still schedules animation frames but skips
/// all GPU work — the canvas remains untouched.  Use this from the Theme
/// Settings UI; defaults to `true` (on).  No-op on non-WASM builds.
pub fn set_gpu_overlay_enabled(enabled: bool) {
    #[cfg(target_arch = "wasm32")]
    GPU_OVERLAY_ENABLED.with(|c| c.set(enabled));
    #[cfg(not(target_arch = "wasm32"))]
    let _ = enabled;
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn is_overlay_enabled() -> bool {
    GPU_OVERLAY_ENABLED.with(|c| c.get())
}

// ── Shared GPU access ───────────────────────────────────────────────────────

/// Publish the overlay's GPU handles. Called once by the overlay's bootstrap
/// after `context.configure(...)` succeeds.
#[cfg(target_arch = "wasm32")]
pub(crate) fn set_shared_gpu(g: SharedGpu) {
    SHARED_GPU.with(|c| *c.borrow_mut() = Some(Rc::new(g)));
}

/// Return the shared GPU handles if the overlay has finished its bootstrap.
/// Secondary renderers should poll this until `Some` is returned.
#[cfg(target_arch = "wasm32")]
pub fn shared_gpu() -> Option<Rc<SharedGpu>> {
    SHARED_GPU.with(|c| c.borrow().clone())
}

// ── Frame-callback registry ─────────────────────────────────────────────────

/// Handle returned by [`register_frame_callback`]; dropping it unregisters
/// the callback from the overlay loop.
#[cfg(target_arch = "wasm32")]
#[must_use = "the callback is unregistered when this handle is dropped"]
pub struct FrameCallbackHandle(u64);

#[cfg(target_arch = "wasm32")]
impl Drop for FrameCallbackHandle {
    fn drop(&mut self) {
        let id = self.0;
        FRAME_CALLBACKS.with(|c| c.borrow_mut().retain(|(i, _)| *i != id));
    }
}

/// Register a callback invoked once per frame after the overlay's own pass.
/// The callback runs on the same `GPUDevice` and renders into the same swap-
/// chain texture, so it can composite freely on top of the overlay's smoke
/// and particles using `loadOp: "load"`.
#[cfg(target_arch = "wasm32")]
pub fn register_frame_callback<F>(cb: F) -> FrameCallbackHandle
where
    F: FnMut(&FrameContext) + 'static,
{
    let id = NEXT_CB_ID.with(|n| { let v = n.get(); n.set(v + 1); v });
    let boxed: Rc<RefCell<FrameCallback>> = Rc::new(RefCell::new(Box::new(cb)));
    FRAME_CALLBACKS.with(|c| c.borrow_mut().push((id, boxed)));
    FrameCallbackHandle(id)
}

/// Invoke all registered frame callbacks. Called by the overlay's render loop
/// after submitting its own command buffer.
#[cfg(target_arch = "wasm32")]
pub(crate) fn invoke_frame_callbacks(ctx: &FrameContext) {
    // Snapshot the list so callbacks may freely register/unregister.
    let cbs: Vec<Rc<RefCell<FrameCallback>>> = FRAME_CALLBACKS
        .with(|c| c.borrow().iter().map(|(_, cb)| cb.clone()).collect());
    for cb in cbs {
        if let Ok(mut f) = cb.try_borrow_mut() {
            (f)(ctx);
        }
    }
}

// ── Live effect-settings access ─────────────────────────────────────────────

/// Replace the live effect settings consumed by the [`WgpuOverlay`] render
/// loop.  Always marks the palette as dirty so the colour buffer is
/// re-uploaded on the next frame.
///
/// Use this for **live preview** in the Theme Settings UI: each draft change
/// pushes a new snapshot here for an immediate visual update without
/// touching `localStorage`.  Persistence is handled separately via
/// [`EffectSettings::save`].
pub fn set_live_effects(s: EffectSettings) {
    EFFECTS_LIVE.with(|c| *c.borrow_mut() = s);
    #[cfg(target_arch = "wasm32")]
    PALETTE_DIRTY.with(|c| c.set(true));
}

/// Snapshot the currently live effect settings.
pub fn live_effects() -> EffectSettings {
    EFFECTS_LIVE.with(|c| c.borrow().clone())
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn take_palette_dirty() -> bool {
    PALETTE_DIRTY.with(|c| {
        let was = c.get();
        c.set(false);
        was
    })
}

// ═════════════════════════════════════════════════════════════════════════════
// Public Dioxus component
// ═════════════════════════════════════════════════════════════════════════════

/// Full-screen WebGPU compositor component.
///
/// Mount this **inside** [`crate::ViewerShell`] so it acquires
/// `#webgpu-canvas` and renders behind the `#ui-root` HTML overlay.  The
/// component itself renders nothing — it is purely a [`use_effect`]
/// side-effect that spawns a GPU render loop.
///
/// ```rust,ignore
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
    render_loop::mount_overlay();

    rsx! {}
}
