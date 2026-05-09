use dioxus::prelude::*;

use crate::components::{
    ChevronRightIcon,
    CloseIcon,
    HamburgerIcon,
    ResizeDirection,
    ResizeEdge,
    ResizeHandle,
};

pub const SIDEBAR_MOBILE_BREAKPOINT_PX: f64 = 768.0;

pub fn is_mobile_sidebar_viewport() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::window;

        window()
            .and_then(|win| win.inner_width().ok())
            .and_then(|width| width.as_f64())
            .map(|width| width <= SIDEBAR_MOBILE_BREAKPOINT_PX)
            .unwrap_or(false)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

/// Collapsible, resizable sidebar with a full-screen drawer at ≤768 px.
#[component]
pub fn Sidebar(
    children: Element,
    #[props(default)] title: Option<String>,
    #[props(default)] badge: Option<String>,
    #[props(default = false)] collapsed: bool,
    #[props(default)] on_toggle: EventHandler<()>,
    #[props(default = true)] resizable: bool,
    #[props(default = 280.0)] initial_width: f64,
    #[props(default = 120.0)] min_width: f64,
    #[props(default)] class: String,
    #[props(default)] mobile_open: Option<bool>,
    #[props(default)] on_mobile_open_change: EventHandler<bool>,
) -> Element {
    let mut width = use_signal(|| initial_width);
    let mut drawer_open = use_signal(|| false);

    let is_open = mobile_open.unwrap_or_else(|| *drawer_open.read());

    let mut open_drawer = move || {
        if mobile_open.is_some() {
            on_mobile_open_change.call(true);
        } else {
            drawer_open.set(true);
            on_mobile_open_change.call(true);
        }
    };

    let mut close_drawer = move || {
        if mobile_open.is_some() {
            on_mobile_open_change.call(false);
        } else {
            drawer_open.set(false);
            on_mobile_open_change.call(false);
        }
    };

    #[cfg_attr(
        not(target_arch = "wasm32"),
        allow(unused_mut, unused_variables)
    )]
    let mut touch_start_x: Signal<f64> = use_signal(|| 0.0);

    let on_touch_start = move |evt: Event<TouchData>| {
        #[cfg(target_arch = "wasm32")]
        {
            use dioxus::web::WebEventExt as _;
            let x = evt
                .data()
                .try_as_web_event()
                .and_then(|e: web_sys::TouchEvent| e.touches().get(0))
                .map(|touch| touch.client_x() as f64)
                .unwrap_or(0.0);
            touch_start_x.set(x);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = evt;
        }
    };

    let on_touch_end = move |evt: Event<TouchData>| {
        #[cfg(target_arch = "wasm32")]
        {
            use dioxus::web::WebEventExt as _;
            let x_end = evt
                .data()
                .try_as_web_event()
                .and_then(|e: web_sys::TouchEvent| e.changed_touches().get(0))
                .map(|touch| touch.client_x() as f64)
                .unwrap_or(0.0);
            if x_end - *touch_start_x.read() > 60.0 {
                close_drawer();
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = evt;
        }
    };

    let sidebar_class = {
        let mut parts = vec!["sidebar"];
        if collapsed {
            parts.push("sidebar-collapsed");
        }
        if is_open {
            parts.push("sidebar-mobile-open");
        } else {
            parts.push("sidebar-mobile-closed");
        }
        let base = parts.join(" ");
        if class.is_empty() {
            base
        } else {
            format!("{base} {class}")
        }
    };

    let inline_style = if collapsed {
        "width: 0px; min-width: 0px; overflow: hidden;".to_string()
    } else {
        format!("width: {}px; min-width: {}px", *width.read(), min_width)
    };

    rsx! {
        div {
            class: if is_open { "sidebar-overlay visible" } else { "sidebar-overlay" },
            onclick: move |_| close_drawer(),
        }

        div {
            class: "{sidebar_class}",
            style: "{inline_style}",
            ontouchstart: on_touch_start,
            ontouchend: on_touch_end,

            div {
                class: "sidebar-header",
                if let Some(title) = &title {
                    h2 { "{title}" }
                }
                if let Some(badge) = &badge {
                    span { class: "sidebar-badge", "{badge}" }
                }
                button {
                    class: "sidebar-close-btn",
                    style: "min-width: 44px; min-height: 44px;",
                    aria_label: "Close sidebar",
                    onclick: move |_| close_drawer(),
                    CloseIcon {}
                }
                button {
                    class: "sidebar-collapse-btn",
                    aria_label: "Collapse sidebar",
                    onclick: move |_| on_toggle.call(()),
                    span {
                        style: if collapsed { "transform: rotate(0deg)" } else { "transform: rotate(180deg)" },
                        ChevronRightIcon {}
                    }
                }
            }

            div {
                class: "sidebar-content",
                {children}
            }

            if resizable && !collapsed {
                ResizeHandle {
                    edge: ResizeEdge::Right,
                    direction: ResizeDirection::Horizontal,
                    min_size: min_width,
                    on_resize: move |delta: f64| {
                        let new_width = (*width.read() + delta).max(min_width);
                        width.set(new_width);
                    },
                }
            }
        }

        if mobile_open.is_none() {
            button {
                class: "sidebar-hamburger",
                style: "min-width: 44px; min-height: 44px;",
                aria_label: "Open sidebar",
                onclick: move |_| open_drawer(),
                HamburgerIcon {}
            }
        }
    }
}
