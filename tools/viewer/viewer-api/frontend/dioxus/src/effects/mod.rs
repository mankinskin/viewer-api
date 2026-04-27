//! GPU effects for viewer-api-dioxus.
//!
//! Currently provides the [`WgpuOverlay`] full-screen compositor component.
pub mod wgpu_overlay;
pub use wgpu_overlay::WgpuOverlay;
#[allow(deprecated)]
pub use wgpu_overlay::set_gpu_canvas_owner;
pub use wgpu_overlay::set_gpu_overlay_enabled;
#[cfg(target_arch = "wasm32")]
pub use wgpu_overlay::{
    register_frame_callback, shared_gpu, FrameCallbackHandle, FrameContext, SharedGpu,
};
