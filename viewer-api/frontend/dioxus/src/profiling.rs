//! Zero-cost browser profiling scopes.
//!
//! [`profile_scope!`] opens a `tracing` span only when BOTH of the following
//! hold:
//!
//! 1. the crate is compiled for `wasm32` (the browser target), and
//! 2. the `profile-browser` cargo feature is enabled.
//!
//! When either condition is false the macro expands to nothing, so the hot
//! render loop carries no instrumentation in production builds — the
//! `wasm-bindgen` cross-boundary calls that `tracing-wasm` makes for every
//! span entry/exit cannot warp the captured timings because they are never
//! emitted.
//!
//! Under `profile-browser`, the span is recorded at `TRACE` level. The
//! `tracing-wasm` console layer (configured in
//! [`crate::tracing_setup`]) mirrors each span into the browser
//! `performance` timeline via `performance.measure`, which Chromium captures
//! under the `blink.user_timing` tracing category. Collect the trace with the
//! Playwright `startBrowserTrace` helper
//! (`e2e/shared/profiling.ts`).
//!
//! # Usage
//!
//! ```ignore
//! pub fn render_frame(state: &mut RenderState) {
//!     crate::profile_scope!("graph3d::render_frame");
//!     // ... hot path ...
//! }
//! ```
//!
//! The guard binding lives until the end of the enclosing block, so the span
//! is exited automatically when `render_frame` returns.

/// Open a `TRACE`-level profiling span for the current block.
///
/// Active only under `cfg(all(target_arch = "wasm32", feature =
/// "profile-browser"))`; otherwise expands to nothing.
#[cfg(all(target_arch = "wasm32", feature = "profile-browser"))]
#[macro_export]
macro_rules! profile_scope {
    ($name:expr $(,)?) => {
        let __viewer_api_profile_guard =
            ::tracing::trace_span!(target: "viewer_api::profiling", $name).entered();
    };
}

/// No-op fallback used in production builds and on non-wasm targets.
#[cfg(not(all(target_arch = "wasm32", feature = "profile-browser")))]
#[macro_export]
macro_rules! profile_scope {
    ($name:expr $(,)?) => {};
}
