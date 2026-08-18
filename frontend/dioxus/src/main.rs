use dioxus::prelude::*;
use viewer_api_dioxus::store::ThemeProvider;

#[path = "main/demo.rs"]
mod demo;
#[path = "main/session_demo.rs"]
mod session_demo;
#[path = "main/url_state_demo.rs"]
mod url_state_demo;

use demo::Demo;

fn main() {
    #[cfg(target_arch = "wasm32")]
    viewer_api_dioxus::tracing_setup::install();
    dioxus_web::launch::launch_cfg(App, dioxus_web::Config::default());
}

#[component]
fn App() -> Element {
    rsx! {
        ThemeProvider {
            Demo {}
        }
    }
}
