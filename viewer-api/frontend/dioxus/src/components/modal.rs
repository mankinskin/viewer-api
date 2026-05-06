//! Overlay/Modal — full-screen backdrop hosting a centered panel.
//!
//! Renders a backdrop layer that dismisses the overlay when clicked
//! outside the panel.  Press `Escape` to close while the overlay is open.
//!
//! ## Usage
//!
//! ```ignore
//! Overlay {
//!     open: show.read().clone(),
//!     on_close: move |_| show.set(false),
//!     // Optional: extra CSS class on the panel.
//!     panel_class: "my-panel",
//!     ThemeSettings { on_close: move |_| show.set(false) }
//! }
//! ```
//!
//! CSS lives in `viewer-api/public/css/modal.css`.
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
use {
    gloo_events::EventListener,
    wasm_bindgen::JsCast,
    web_sys::KeyboardEvent,
};

/// Full-screen overlay with backdrop dismiss + Escape-to-close.
///
/// When `open == false` the component renders nothing.
///
/// The backdrop captures pointer events; clicking the backdrop fires
/// `on_close`.  Clicks inside `.modal-panel` are stopped from
/// propagating so they don't trigger dismiss.
#[component]
pub fn Overlay(
    open: bool,
    on_close: EventHandler<()>,
    /// Optional extra CSS classes on the inner `.modal-panel` element.
    #[props(default)]
    panel_class: String,
    /// Optional ARIA label for screen readers.
    #[props(default = "Dialog".to_string())]
    aria_label: String,
    children: Element,
) -> Element {
    // Escape-key listener: held in a signal so it's dropped (and
    // unregistered) when `open` flips back to false or the component
    // unmounts.  We keep at most one listener alive at a time.
    #[cfg(target_arch = "wasm32")]
    {
        let mut listener_slot: Signal<Option<EventListener>> = use_signal(|| None);
        let on_close_for_effect = on_close;
        use_effect(move || {
            if !open {
                listener_slot.set(None);
                return;
            }
            let window = match web_sys::window() {
                Some(w) => w,
                None => return,
            };
            let handler = on_close_for_effect;
            let listener = EventListener::new(&window, "keydown", move |evt| {
                if let Some(kev) = evt.dyn_ref::<KeyboardEvent>() {
                    if kev.key() == "Escape" {
                        handler.call(());
                    }
                }
            });
            listener_slot.set(Some(listener));
        });
    }

    if !open {
        return rsx! {};
    }

    let panel_combined = if panel_class.is_empty() {
        "modal-panel".to_string()
    } else {
        format!("modal-panel {panel_class}")
    };

    rsx! {
        div {
            class: "modal-backdrop",
            role: "dialog",
            aria_modal: "true",
            aria_label: "{aria_label}",
            onclick: move |evt| {
                // Only fire on direct backdrop clicks, not bubbled clicks
                // from inside the panel.
                evt.stop_propagation();
                on_close.call(());
            },

            div {
                class: "{panel_combined}",
                // Stop click propagation so clicks inside the panel
                // don't bubble up and dismiss the overlay.
                onclick: move |evt| evt.stop_propagation(),

                {children}
            }
        }
    }
}
