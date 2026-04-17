pub mod components;
pub mod effects;
pub mod store;
pub use components::*;
pub use effects::WgpuOverlay;
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
            // Dark base colour — visible through transparent GPU overlay regions
            // (smoke, atmospheric effects). The GPU canvas does NOT paint a solid
            // background any more; CSS owns the dark background so DOM elements
            // can show through the transparent WebGPU overlay canvas.
            style: "width: 100vw; height: 100vh; margin: 0; padding: 0; overflow: hidden; position: relative; background: #0a0a0c;",

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
