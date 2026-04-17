//! Tests for viewer-api-leptos components.
//!
//! This file runs as native Rust tests (no WASM required) for pure-logic
//! functions, and as wasm-bindgen-test tests for DOM-dependent behavior
//! (see `tests/browser/` for those).

// ── TreeNode construction ─────────────────────────────────────────────────────

#[cfg(test)]
mod tree_view_tests {
    use crate::components::tree_view::{NodeIcon, TreeNode};

    #[test]
    fn tree_node_defaults() {
        let node = TreeNode::default();
        assert!(node.id.is_empty());
        assert!(node.label.is_empty());
        assert_eq!(node.icon, NodeIcon::None);
        assert!(node.badge.is_none());
        assert!(node.children.is_empty());
    }

    #[test]
    fn tree_node_with_children() {
        let parent = TreeNode {
            id: "root".into(),
            label: "Root".into(),
            icon: NodeIcon::Folder,
            badge: Some("3".into()),
            children: vec![
                TreeNode {
                    id: "a".into(),
                    label: "a.log".into(),
                    icon: NodeIcon::File,
                    badge: Some("1.2 KB".into()),
                    children: vec![],
                },
                TreeNode {
                    id: "b".into(),
                    label: "b.log".into(),
                    icon: NodeIcon::File,
                    badge: None,
                    children: vec![],
                },
            ],
        };

        assert_eq!(parent.children.len(), 2);
        assert_eq!(parent.children[0].id, "a");
        assert!(parent.children[1].badge.is_none());
    }

    #[test]
    fn node_icon_partial_eq() {
        assert_eq!(NodeIcon::File, NodeIcon::File);
        assert_ne!(NodeIcon::File, NodeIcon::Folder);
        assert_ne!(NodeIcon::File, NodeIcon::None);
    }
}

// ── Sidebar width calculation ────────────────────────────────────────────────

#[cfg(test)]
mod sidebar_tests {
    /// Mirrors the clamping logic in sidebar.rs `on_resize`.
    fn clamp_width(current: f64, delta: f64, min: f64) -> f64 {
        (current + delta).max(min)
    }

    #[test]
    fn width_increases_on_positive_delta() {
        assert_eq!(clamp_width(260.0, 40.0, 120.0), 300.0);
    }

    #[test]
    fn width_decreases_on_negative_delta() {
        assert_eq!(clamp_width(260.0, -60.0, 120.0), 200.0);
    }

    #[test]
    fn width_clamped_to_minimum() {
        assert_eq!(clamp_width(260.0, -200.0, 120.0), 120.0);
    }

    #[test]
    fn width_clamp_at_min_boundary() {
        assert_eq!(clamp_width(120.0, -1.0, 120.0), 120.0);
    }

    /// Mirrors format_size from sidebar.rs (duplicated here for testing).
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

    #[test]
    fn format_size_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn format_size_kilobytes() {
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(2048), "2.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
    }

    #[test]
    fn format_size_megabytes() {
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 2), "2.0 MB");
    }
}
