//! Session ID utilities backed by `sessionStorage`.
//!
//! All operations go through `web_sys` — no inline JS dependencies.
//!
//! ## Session lifetime
//!
//! The session ID is persisted in `sessionStorage` under the key
//! `viewer-api-session-id`.  It survives page refreshes within the same
//! browser tab but is cleared when the tab is closed.  Call [`clear_session`]
//! to reset it programmatically (the next call to [`get_session_id`] generates
//! a new ID).
//!
//! ## API integration
//!
//! Pass a header list through [`with_session`] to inject the
//! `X-Session-Id` header before dispatching HTTP requests.

const SESSION_KEY: &str = "viewer-api-session-id";
const SESSION_HEADER: &str = "X-Session-Id";

// ── Public API ────────────────────────────────────────────────────────────────

/// Returns the current session ID, generating and persisting one if absent.
///
/// The ID is a UUID v4 generated from `Math.random()` on first call and stored
/// in `sessionStorage`.  Subsequent calls within the same tab return the same
/// value.
pub fn get_session_id() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let storage = web_sys::window()
            .and_then(|w| w.session_storage().ok().flatten());

        if let Some(storage) = storage {
            if let Ok(Some(id)) = storage.get_item(SESSION_KEY) {
                return id;
            }
            let id = generate_uuid();
            let _ = storage.set_item(SESSION_KEY, &id);
            return id;
        }

        // sessionStorage unavailable (e.g. private-mode quota error).
        generate_uuid()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        "no-session".to_owned()
    }
}

/// Clears the session ID from `sessionStorage`.
///
/// The next call to [`get_session_id`] will generate a fresh ID.
pub fn clear_session() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.session_storage().ok().flatten())
        {
            let _ = storage.remove_item(SESSION_KEY);
        }
    }
}

/// Injects an `X-Session-Id` header into `headers` and returns it.
///
/// The session ID is obtained from [`get_session_id`].  Use this when
/// constructing request headers:
///
/// ```no_run
/// # use viewer_api_dioxus::with_session;
/// let headers = with_session(vec![
///     ("Content-Type".to_owned(), "application/json".to_owned()),
/// ]);
/// ```
pub fn with_session(mut headers: Vec<(String, String)>) -> Vec<(String, String)> {
    headers.push((SESSION_HEADER.to_owned(), get_session_id()));
    headers
}

// ── UUID generation ───────────────────────────────────────────────────────────

/// Generates a UUID v4 using `Math.random()`.
///
/// Applies the version-4 (`0100xxxx`) and variant (`10xxxxxx`) bit patterns
/// mandated by RFC 4122.  `Math.random()` is not cryptographically random but
/// is sufficient for session tracking.
#[cfg(target_arch = "wasm32")]
fn generate_uuid() -> String {
    let mut b: Vec<u8> = (0..16)
        .map(|_| (js_sys::Math::random() * 256.0) as u8)
        .collect();

    // Set version bits: b[6] = 0100xxxx
    b[6] = (b[6] & 0x0f) | 0x40;
    // Set variant bits: b[8] = 10xxxxxx
    b[8] = (b[8] & 0x3f) | 0x80;

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0], b[1], b[2], b[3],
        b[4], b[5],
        b[6], b[7],
        b[8], b[9],
        b[10], b[11], b[12], b[13], b[14], b[15],
    )
}
