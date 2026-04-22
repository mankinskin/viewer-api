//! GPU effects for viewer-api-dioxus.
//!
//! Currently provides the [`WgpuOverlay`] full-screen compositor component.
pub mod wgpu_overlay;
pub use wgpu_overlay::WgpuOverlay;
pub use wgpu_overlay::set_gpu_canvas_owner;
pub use wgpu_overlay::set_gpu_overlay_enabled;
