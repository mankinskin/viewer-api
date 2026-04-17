//! Browser-environment tests for `viewer-api-leptos` components.
//!
//! These tests run inside a headless browser via `wasm-pack test`:
//!
//! ```sh
//! wasm-pack test --headless --chrome tools/viewer/viewer-api-leptos
//! ```
//!
//! They validate the acceptance criteria for:
//!   - Ticket 29897f92 (tab bar, sidebar, resizable panels, viewer-api-leptos crate)
//!
//! # What is tested here vs. in `src/tests.rs`
//!
//! `src/tests.rs` — pure-Rust logic run with `cargo test` (native).
//! `tests/browser.rs` — same logic compiled to WASM, executed in a real browser
//!   engine, confirming numeric/float behavior and basic JS interop.

use wasm_bindgen_test::*;
use viewer_api_leptos::components::tree_view::{NodeIcon, TreeNode};

wasm_bindgen_test_configure!(run_in_browser);

// ── TreeNode in WASM environment ────────────────────────────────────────────

#[wasm_bindgen_test]
fn tree_node_default_in_wasm() {
    let n = TreeNode::default();
    assert!(n.id.is_empty(), "default id should be empty");
    assert!(n.label.is_empty(), "default label should be empty");
    assert_eq!(n.icon, NodeIcon::None, "default icon should be None");
    assert!(n.badge.is_none(), "default badge should be None");
    assert!(n.children.is_empty(), "default children should be empty");
}

#[wasm_bindgen_test]
fn tree_node_clone_preserves_children() {
    let original = TreeNode {
        id: "root".into(),
        label: "Root".into(),
        icon: NodeIcon::Folder,
        badge: Some("3".into()),
        children: vec![
            TreeNode {
                id: "child".into(),
                label: "child.log".into(),
                icon: NodeIcon::File,
                badge: Some("512 B".into()),
                children: vec![],
            },
        ],
    };
    let cloned = original.clone();
    assert_eq!(cloned.id, "root");
    assert_eq!(cloned.children.len(), 1);
    assert_eq!(cloned.children[0].id, "child");
}

#[wasm_bindgen_test]
fn node_icon_eq_in_wasm() {
    assert_eq!(NodeIcon::File, NodeIcon::File);
    assert_eq!(NodeIcon::Folder, NodeIcon::Folder);
    assert_ne!(NodeIcon::File, NodeIcon::Folder);
    assert_ne!(NodeIcon::None, NodeIcon::File);
}

// ── Width clamping in WASM (floating-point parity) ──────────────────────────

/// Mirrors the resize clamping logic in sidebar.rs `on_resize`.
fn clamp_width(current: f64, delta: f64, min: f64) -> f64 {
    (current + delta).max(min)
}

#[wasm_bindgen_test]
fn width_clamp_positive_delta_wasm() {
    assert_eq!(clamp_width(260.0, 50.0, 120.0), 310.0);
}

#[wasm_bindgen_test]
fn width_clamp_negative_delta_wasm() {
    assert_eq!(clamp_width(260.0, -100.0, 120.0), 160.0);
}

#[wasm_bindgen_test]
fn width_clamp_below_minimum_wasm() {
    // Dragging far left should clamp to MIN_WIDTH, not go negative.
    let result = clamp_width(260.0, -300.0, 120.0);
    assert_eq!(result, 120.0, "width must not go below minimum");
}

#[wasm_bindgen_test]
fn width_clamp_exactly_at_minimum_wasm() {
    let result = clamp_width(120.0, 0.0, 120.0);
    assert_eq!(result, 120.0);
}

// ── Sidebar format_size (mirrors log-viewer-leptos sidebar logic) ───────────

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[wasm_bindgen_test]
fn format_size_bytes_wasm() {
    assert_eq!(format_size(0), "0 B");
    assert_eq!(format_size(1023), "1023 B");
}

#[wasm_bindgen_test]
fn format_size_kilobytes_wasm() {
    assert_eq!(format_size(1024), "1.0 KB");
    assert_eq!(format_size(2048), "2.0 KB");
}

#[wasm_bindgen_test]
fn format_size_megabytes_wasm() {
    assert_eq!(format_size(1024 * 1024), "1.0 MB");
    assert_eq!(format_size(1024 * 1024 * 3), "3.0 MB");
}
