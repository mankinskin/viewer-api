pub mod components;
pub mod effects;
pub mod graph3d;
pub mod store;
#[cfg(target_arch = "wasm32")]
pub mod tracing_setup;

/// Join CSS class names, filtering out empty strings and `None` values.
///
/// # Usage
/// ```rust
/// let cls = classes!("btn", is_active.then_some("btn-active"), extra_class);
/// ```
#[macro_export]
macro_rules! classes {
    ($($part:expr),* $(,)?) => {{
        let mut parts: Vec<&str> = Vec::new();
        $(
            // Accept &str, String, Option<&str>, Option<String>
            let s: Option<&str> = $crate::__classes_as_opt_str!($part);
            if let Some(s) = s {
                if !s.is_empty() { parts.push(s); }
            }
        )*
        parts.join(" ")
    }};
}

/// Internal helper — converts an expression to `Option<&str>`.
/// Not part of the public API.
#[macro_export]
#[doc(hidden)]
macro_rules! __classes_as_opt_str {
    ($e:expr) => {{
        use $crate::__ClassesConvert;
        (&$e).into_opt_str()
    }};
}

/// Sealed trait that lets [`classes!`] accept `&str`, `String`,
/// `Option<&str>`, and `Option<String>` uniformly.
pub trait __ClassesConvert {
    fn into_opt_str(&self) -> Option<&str>;
}
impl __ClassesConvert for &str {
    fn into_opt_str(&self) -> Option<&str> { Some(self) }
}
impl __ClassesConvert for str {
    fn into_opt_str(&self) -> Option<&str> { Some(self) }
}
impl __ClassesConvert for String {
    fn into_opt_str(&self) -> Option<&str> { Some(self.as_str()) }
}
impl __ClassesConvert for Option<&str> {
    fn into_opt_str(&self) -> Option<&str> { *self }
}
impl __ClassesConvert for Option<String> {
    fn into_opt_str(&self) -> Option<&str> { self.as_deref() }
}

pub use graph3d::{
    can_use_webgpu as can_use_webgpu_graph3d, CameraCommand, EdgeRef3D, Graph3D, Layout3D, Node3D,
    DEFAULT_CONTAINER_ID as GRAPH3D_DEFAULT_CONTAINER_ID,
};
pub use components::*;
pub use effects::WgpuOverlay;
#[allow(deprecated)]
pub use effects::set_gpu_canvas_owner;
pub use effects::set_gpu_overlay_enabled;
pub use store::*;
// Explicit re-exports so downstream crates can import without glob.
pub use store::session::{clear_session, get_session_id, with_session};
pub use store::url_state::{get_hash_param, remove_hash_param, set_hash_param, UrlStateManager};

use dioxus::prelude::*;

/// Full-screen root shell used by all viewer applications built on this
/// platform. Renders an absolute-positioned WebGPU canvas underneath a
/// pointer-transparent UI overlay root.
///
/// Downstream crates should mount their own content *inside* `#ui-root` by
/// nesting it as children, or by relying on Dioxus Router to inject route
/// components into the overlay.
#[component]
pub fn ViewerShell(children: Element) -> Element {
    rsx! {
        div {
            // Dark base colour shows through transparent regions of the WGPU
            // canvas (e.g. when smoke is disabled) so the UI never flashes
            // white. When the smoke shader is active it fully covers this
            // surface; panels above use their own translucent backgrounds
            // (`--panel-bg`) so the smoke bleeds through every UI layer.
            style: "width: 100vw; height: 100vh; margin: 0; padding: 0; overflow: hidden; position: relative; background: #050608;",

            // UI root — graph nodes and all other DOM content render here.
            // z-index 3 keeps it above the GPU overlay canvas at z-index 1,
            // so opaque UI panels (sidebars, headers) occlude the canvas.
            // Only the transparent content area lets the GPU canvas show through.
            div {
                id: "ui-root",
                style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; z-index: 3;",
                {children}
            }

            // WebGPU overlay canvas — renders GPU effects (graph edges, particles,
            // atmospheric glows) BEHIND DOM content.
            //
            // Key properties:
            //   z-index: 1          — sits below #ui-root in the stacking context
            //   pointer-events:none — mouse/touch events pass through to the DOM
            //   alphaMode set to "premultiplied" by WgpuOverlay at runtime so
            //   transparent canvas regions reveal the CSS background below.
            //
            // WgpuOverlay acquires this element by id. Keep the id stable —
            // downstream tickets reference it.
            canvas {
                id: "webgpu-canvas",
                style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; display: block; z-index: 1; pointer-events: none;",
            }
        }
    }
}
