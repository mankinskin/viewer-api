//! SSE helper utilities for building typed server-sent events.
//!
//! Wraps `axum::response::sse::Event` with a helper that serialises a typed
//! payload to JSON, assigns a monotonic `id`, and sets the `event` name field.

use axum::response::sse::Event;
use serde::Serialize;

/// Build an SSE `Event` from a typed payload.
///
/// - `event_name` maps to the SSE `event:` field (e.g., `"ticket.upsert"`)
/// - `id` is a monotonically increasing u64 counter (converted to string)
/// - `payload` is serialised to JSON and set as the SSE `data:` field
///
/// Returns `Err` only if `serde_json::to_string` fails.
pub fn sse_event<T: Serialize>(
    event_name: &str,
    id: u64,
    payload: &T,
) -> Result<Event, serde_json::Error> {
    let data = serde_json::to_string(payload)?;
    Ok(Event::default()
        .id(id.to_string())
        .event(event_name)
        .data(data))
}

/// Helper to produce SSE keep-alive events.
pub fn sse_keep_alive() -> Event {
    Event::default().comment("keep-alive")
}
