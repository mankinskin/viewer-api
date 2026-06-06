//! Browser micro-benchmarks for the 3-D graph hot path.
//!
//! These run inside a real headless Chromium so the measured timings reflect
//! the V8/WebAssembly runtime that ships to users, not a native build.
//!
//! Run with:
//!
//! ```bash
//! wasm-pack test --chrome --headless \
//!   memory-viewers/viewer-api/viewer-api/frontend/dioxus
//! ```
//!
//! Each benchmark uses the browser high-resolution clock
//! (`window.performance.now()`) and logs the elapsed milliseconds to the
//! console so the timing is captured in `wasm-pack` stdout. They assert only
//! that the work completed (no wall-clock threshold) so they stay stable
//! across machines while still surfacing regressions in the logged numbers.
#![cfg(target_arch = "wasm32")]

use viewer_api_dioxus::graph3d::math;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

fn performance() -> web_sys::Performance {
    web_sys::window()
        .expect("window")
        .performance()
        .expect("performance")
}

fn log(line: String) {
    web_sys::console::log_1(&line.into());
}

/// Project matrix construction is invoked once per camera change and feeds
/// every per-frame uniform upload. Benchmark a large batch to amortise the
/// clock resolution.
#[wasm_bindgen_test]
fn bench_perspective_projection() {
    const ITERS: u32 = 200_000;
    let perf = performance();
    let start = perf.now();
    let mut acc = 0.0f32;
    for i in 0..ITERS {
        let aspect = 1.0 + (i as f32) * 1e-6;
        let m = math::perspective(1.2, aspect, 0.1, 1000.0);
        acc += m[0] + m[14];
    }
    let elapsed = perf.now() - start;
    log(format!(
        "bench_perspective_projection: {ITERS} iters in {elapsed:.3}ms ({:.4}us/iter, acc={acc})",
        elapsed * 1000.0 / ITERS as f64
    ));
    assert!(elapsed.is_finite());
}

/// `look_at` runs for the orbit camera on every camera animation step.
#[wasm_bindgen_test]
fn bench_look_at_view_matrix() {
    const ITERS: u32 = 200_000;
    let perf = performance();
    let start = perf.now();
    let mut acc = 0.0f32;
    for i in 0..ITERS {
        let t = (i as f32) * 1e-4;
        let eye = [t.cos() * 10.0, 5.0, t.sin() * 10.0];
        let m = math::look_at(eye, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        acc += m[0] + m[5];
    }
    let elapsed = perf.now() - start;
    log(format!(
        "bench_look_at_view_matrix: {ITERS} iters in {elapsed:.3}ms ({:.4}us/iter, acc={acc})",
        elapsed * 1000.0 / ITERS as f64
    ));
    assert!(elapsed.is_finite());
}
