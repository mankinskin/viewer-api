//! GPU effects for viewer-api-dioxus.
//!
//! Currently provides the [`WgpuOverlay`] full-screen compositor component.
pub mod wgpu_overlay;
#[allow(deprecated)]
pub use wgpu_overlay::set_gpu_canvas_owner;
#[cfg(target_arch = "wasm32")]
pub use wgpu_overlay::{
    register_frame_callback,
    shared_gpu,
    FrameCallbackHandle,
    FrameContext,
    SharedGpu,
};
pub use wgpu_overlay::{
    set_gpu_overlay_enabled,
    WgpuOverlay,
};
