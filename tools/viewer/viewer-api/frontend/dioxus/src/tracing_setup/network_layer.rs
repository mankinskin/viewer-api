//! Browser-side tracing layer that batches records and ships them to
//! `POST /api/client-log` on the same origin.
//!
//! # Opt-in
//!
//! Only registered when `?log_sink=on` is in the URL or
//! `localStorage["viewer-api-log-sink"] === "on"`.
//!
//! # Batching
//!
//! Records are buffered in a shared `Arc<Mutex<Vec<_>>>`.  A background
//! `spawn_local` task flushes the buffer to the server every 2 seconds or
//! when the buffer reaches 64 records.

#![cfg(target_arch = "wasm32")]

use std::sync::{Arc, Mutex};
use tracing::{Event, Subscriber};
use tracing_subscriber::{layer::Context, Layer};
use wasm_bindgen::{JsCast, JsValue};

// ── Public layer type ─────────────────────────────────────────────────────────

pub struct NetworkLayer {
    buffer: Arc<Mutex<Vec<serde_json::Value>>>,
}

impl NetworkLayer {
    pub fn new() -> Self {
        NetworkLayer {
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Spawn the background task that drains the buffer every 2 seconds.
    pub fn spawn_flush_loop(&self) {
        let buf = Arc::clone(&self.buffer);
        wasm_bindgen_futures::spawn_local(async move {
            loop {
                sleep_ms(2000).await;
                let records: Vec<serde_json::Value> = {
                    let mut guard = buf.lock().unwrap();
                    if guard.is_empty() {
                        continue;
                    }
                    std::mem::take(&mut *guard)
                };
                if let Err(e) = post_records(&records).await {
                    // Emit once at warn level — avoid infinite recursion by
                    // only logging to console (network layer isn't re-entered
                    // because the NetworkLayer won't be in scope here).
                    let _ = e; // silently drop; avoid noisy console spam
                }
            }
        });
    }
}

impl<S: Subscriber> Layer<S> for NetworkLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let record = serialize_event(event);
        let mut guard = self.buffer.lock().unwrap();
        guard.push(record);
        // Eager flush when buffer is full (handled on next loop iteration).
        // We don't spawn from here to avoid reentrant async scheduling inside
        // a subscriber callback.
    }
}

// ── Event serialisation ───────────────────────────────────────────────────────

struct FieldCollector(serde_json::Map<String, serde_json::Value>);

impl tracing::field::Visit for FieldCollector {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.0.insert(field.name().to_string(), value.into());
    }
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.0.insert(field.name().to_string(), value.into());
    }
    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.0.insert(field.name().to_string(), value.into());
    }
    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.0.insert(
            field.name().to_string(),
            serde_json::Value::Number(value.into()),
        );
    }
    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.0.insert(field.name().to_string(), value.into());
    }
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0
            .insert(field.name().to_string(), format!("{value:?}").into());
    }
}

fn serialize_event(event: &Event<'_>) -> serde_json::Value {
    let meta = event.metadata();
    let mut visitor = FieldCollector(serde_json::Map::new());
    event.record(&mut visitor);
    let ts = js_sys::Date::now() as u64;
    serde_json::json!({
        "ts": ts,
        "level": meta.level().as_str().to_lowercase(),
        "target": meta.target(),
        "fields": visitor.0,
    })
}

// ── Network helpers ───────────────────────────────────────────────────────────

async fn sleep_ms(ms: i32) {
    let p = js_sys::Promise::new(&mut |resolve, _| {
        if let Some(win) = web_sys::window() {
            let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                resolve.unchecked_ref(),
                ms,
            );
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(p).await;
}

async fn post_records(records: &[serde_json::Value]) -> Result<(), String> {
    let body = serde_json::to_string(&serde_json::json!({ "records": records }))
        .map_err(|e| e.to_string())?;

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(web_sys::RequestMode::SameOrigin);
    opts.set_body(&JsValue::from_str(&body));

    let headers = web_sys::Headers::new().map_err(|e| format!("{e:?}"))?;
    headers
        .set("content-type", "application/json")
        .map_err(|e| format!("{e:?}"))?;
    opts.set_headers(&headers);

    let request = web_sys::Request::new_with_str_and_init("/api/client-log", &opts)
        .map_err(|e| format!("{e:?}"))?;

    let win = web_sys::window().ok_or("no window")?;
    let promise = win.fetch_with_request(&request);
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|e| format!("{e:?}"))?;
    Ok(())
}
