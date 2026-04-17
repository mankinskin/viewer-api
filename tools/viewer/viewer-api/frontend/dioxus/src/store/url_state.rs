//! Hash-based URL state with `popstate` back/forward support.
//!
//! All operations go through `web_sys` — no inline JS dependencies.
//!
//! ## Hash format
//!
//! Parameters are stored as `#key=value&key2=value2`.  Keys and values are
//! percent-encoded with `js_sys::encode_uri_component` so that arbitrary
//! strings can be stored safely.
//!
//! ## Loop prevention
//!
//! [`set_hash_param`] / [`remove_hash_param`] use `location.hash` assignment,
//! which triggers a `hashchange` event but **not** a `popstate` event.
//! [`UrlStateManager::new`] registers a `popstate` listener, so the
//! `on_change` callback fires only on genuine browser back/forward navigation —
//! never on programmatic hash writes.

#[cfg(target_arch = "wasm32")]
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// ── RAII popstate listener ────────────────────────────────────────────────────

/// Removes a `popstate` listener when dropped.
#[cfg(target_arch = "wasm32")]
struct PopstateGuard {
    window: web_sys::Window,
    closure: Closure<dyn FnMut()>,
}

#[cfg(target_arch = "wasm32")]
impl Drop for PopstateGuard {
    fn drop(&mut self) {
        let _ = self.window.remove_event_listener_with_callback(
            "popstate",
            self.closure.as_ref().unchecked_ref(),
        );
    }
}

// ── UrlStateManager ───────────────────────────────────────────────────────────

/// Manages hash-fragment URL state across browser back/forward navigation.
///
/// Construct with [`UrlStateManager::new`], passing a callback that is invoked
/// whenever the browser navigates with the back or forward buttons.  Drop this
/// value to unregister the listener automatically.
///
/// Use the free functions [`get_hash_param`], [`set_hash_param`], and
/// [`remove_hash_param`] to read and write individual parameters.
pub struct UrlStateManager {
    #[cfg(target_arch = "wasm32")]
    _guard: Option<PopstateGuard>,
}

impl UrlStateManager {
    /// Creates the manager and registers a `popstate` listener.
    ///
    /// `on_change` is invoked each time the browser navigates back or forward.
    /// Dropping the returned value unregisters the listener.
    ///
    /// The callback is `FnMut` because Dioxus signal mutation methods require
    /// `&mut self`, making the closure `FnMut` rather than `Fn`.
    pub fn new<F>(on_change: F) -> Self
    where
        F: FnMut() + 'static,
    {
        #[cfg(target_arch = "wasm32")]
        let _guard = web_sys::window().and_then(|window| {
            let closure = Closure::wrap(Box::new(on_change) as Box<dyn FnMut()>);
            window
                .add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref())
                .ok()?;
            Some(PopstateGuard { window, closure })
        });

        #[cfg(not(target_arch = "wasm32"))]
        let _ = on_change;

        Self {
            #[cfg(target_arch = "wasm32")]
            _guard,
        }
    }
}

// ── Internal hash helpers ─────────────────────────────────────────────────────

/// Parse the current URL hash fragment into a `key → value` map.
///
/// Strips the leading `#` and splits on `&`. Keys and values are
/// percent-decoded with `js_sys::decode_uri_component`.
#[cfg(target_arch = "wasm32")]
fn parse_hash() -> HashMap<String, String> {
    let raw = web_sys::window()
        .map(|w| w.location().hash().unwrap_or_default())
        .unwrap_or_default();

    let fragment = raw.strip_prefix('#').unwrap_or(&raw);
    let mut map = HashMap::new();

    for pair in fragment.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            let k = js_sys::decode_uri_component(k)
                .ok()
                .and_then(|s| s.as_string())
                .unwrap_or_else(|| k.to_owned());
            let v = js_sys::decode_uri_component(v)
                .ok()
                .and_then(|s| s.as_string())
                .unwrap_or_else(|| v.to_owned());
            map.insert(k, v);
        }
    }

    map
}

/// Serialise a `key → value` map to a hash-fragment string (without `#`).
///
/// Keys and values are percent-encoded with `js_sys::encode_uri_component`.
#[cfg(target_arch = "wasm32")]
fn format_hash(map: &HashMap<String, String>) -> String {
    let pairs: Vec<String> = map
        .iter()
        .map(|(k, v)| {
            let ek = js_sys::encode_uri_component(k);
            let ev = js_sys::encode_uri_component(v);
            format!("{}={}", ek, ev)
        })
        .collect();
    pairs.join("&")
}

// ── Public hash-param API ─────────────────────────────────────────────────────

/// Returns the value of `key` in the URL hash fragment, or `None` if absent.
///
/// Reads `window.location.hash` on every call.
pub fn get_hash_param(key: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        parse_hash().remove(key)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = key;
        None
    }
}

/// Sets `key` to `value` in the URL hash fragment.
///
/// All other existing params are preserved.  The browser records a new history
/// entry so that back navigation restores the previous hash state.
pub fn set_hash_param(key: &str, value: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let mut map = parse_hash();
            map.insert(key.to_owned(), value.to_owned());
            let hash = format_hash(&map);
            let _ = window.location().set_hash(&hash);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (key, value);
    }
}

/// Removes `key` from the URL hash fragment.
///
/// All other existing params are preserved.
pub fn remove_hash_param(key: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let mut map = parse_hash();
            map.remove(key);
            let hash = format_hash(&map);
            let _ = window.location().set_hash(&hash);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = key;
    }
}
