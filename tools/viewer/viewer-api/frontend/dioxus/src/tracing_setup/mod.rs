//! Tracing bootstrap for the Dioxus WASM frontend.
//!
//! Call [`install`] once at the top of `main()`, before any other code logs
//! anything.  It is idempotent — safe to call multiple times.
//!
//! # Log level / filter
//!
//! Resolution order (first match wins):
//! 1. URL query-string parameter `?log=<env-filter-spec>`
//!    e.g. `?log=info,wgpu_overlay=debug`
//! 2. `localStorage["viewer-api-log-filter"]`
//! 3. Default: `"info"`
//!
//! # Network sink (ticket 8f349d96)
//!
//! When `?log_sink=on` is present in the URL, or
//! `localStorage["viewer-api-log-sink"] === "on"`, a [`NetworkLayer`] is
//! added to the subscriber that batches records and POSTs them to
//! `POST /api/client-log` every 2 seconds.

#![cfg(target_arch = "wasm32")]

mod network_layer;

use std::sync::OnceLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

static INSTALLED: OnceLock<()> = OnceLock::new();

/// Install the global tracing subscriber.  Idempotent.
pub fn install() {
    INSTALLED.get_or_init(|| {
        let filter_str = resolve_filter();
        let env_filter = EnvFilter::try_new(&filter_str)
            .unwrap_or_else(|_| EnvFilter::new("info"));

        let console_layer = tracing_wasm::WASMLayer::new(tracing_wasm::WASMLayerConfig::default());

        if is_sink_enabled() {
            let net = network_layer::NetworkLayer::new();
            net.spawn_flush_loop();
            tracing_subscriber::registry()
                .with(env_filter)
                .with(console_layer)
                .with(net)
                .init();
        } else {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(console_layer)
                .init();
        }
    });
}

// ── Filter resolution ─────────────────────────────────────────────────────────

fn resolve_filter() -> String {
    if let Some(f) = url_query_param("log") {
        if !f.is_empty() {
            return f;
        }
    }
    if let Some(f) = local_storage_get("viewer-api-log-filter") {
        if !f.is_empty() {
            return f;
        }
    }
    "info".to_string()
}

fn is_sink_enabled() -> bool {
    if let Some(v) = url_query_param("log_sink") {
        if v == "on" {
            return true;
        }
    }
    local_storage_get("viewer-api-log-sink")
        .map(|v| v == "on")
        .unwrap_or(false)
}

// ── DOM helpers ───────────────────────────────────────────────────────────────

/// Read a single URL query-string parameter by name.
fn url_query_param(key: &str) -> Option<String> {
    let href = web_sys::window()?.location().href().ok()?;
    // Parse the query string portion.
    let qs = href.split('?').nth(1).unwrap_or("").split('#').next()?;
    for pair in qs.split('&') {
        let mut parts = pair.splitn(2, '=');
        let k = parts.next().unwrap_or("");
        let v = parts.next().unwrap_or("");
        if k == key {
            // URL-decode '+' → ' '  and percent-encoded chars (best-effort).
            return Some(
                js_sys::decode_uri_component(v)
                    .unwrap_or_else(|_| js_sys::JsString::from(v))
                    .as_string()
                    .unwrap_or_default(),
            );
        }
    }
    None
}

/// Read a value from localStorage.  Returns `None` if storage is unavailable.
fn local_storage_get(key: &str) -> Option<String> {
    let storage = web_sys::window()?.local_storage().ok()??;
    storage.get_item(key).ok()?
}
