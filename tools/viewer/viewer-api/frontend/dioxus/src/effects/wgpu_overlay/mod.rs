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

// ── Canvas ownership arbitration ────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
thread_local! {
    /// `true` while another renderer (e.g. `Graph3D`) owns `#webgpu-canvas`.
    /// [`WgpuOverlay`] suspends its render loop while this is set.
    static GPU_CANVAS_OWNER: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// Master enable flag for the WgpuOverlay render loop. Defaults to `true`
    /// so first-load viewers render the full GPU experience immediately.
    static GPU_OVERLAY_ENABLED: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
    /// Live (preview-or-committed) effect settings read each frame by the
    /// render loop.  Mutated via [`set_live_effects`] from the Theme Settings
    /// UI for instant preview.
    static EFFECTS_LIVE: std::cell::RefCell<EffectSettings> =
        std::cell::RefCell::new(EffectSettings::default());
    /// Set whenever [`set_live_effects`] is called so the render loop knows
    /// to re-upload the palette uniform buffer on the next frame.
    static PALETTE_DIRTY: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static EFFECTS_LIVE: std::cell::RefCell<EffectSettings> =
        std::cell::RefCell::new(EffectSettings::default());
}

/// Claim (`taken = true`) or release (`taken = false`) exclusive ownership of
/// `#webgpu-canvas` on behalf of another renderer.
///
/// When claimed, [`WgpuOverlay`] suspends its render loop each animation
/// frame so the two GPU contexts do not fight over the same canvas.
///
/// Call with `true` before initialising a competing renderer (e.g. `Graph3D`)
/// and with `false` in that renderer's `use_drop` cleanup.  No-op on
/// non-WASM builds.
pub fn set_gpu_canvas_owner(taken: bool) {
    #[cfg(target_arch = "wasm32")]
    GPU_CANVAS_OWNER.with(|c| c.set(taken));
    #[cfg(not(target_arch = "wasm32"))]
    let _ = taken;
}

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
pub(crate) fn is_canvas_owned() -> bool {
    GPU_CANVAS_OWNER.with(|c| c.get())
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn is_overlay_enabled() -> bool {
    GPU_OVERLAY_ENABLED.with(|c| c.get())
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
