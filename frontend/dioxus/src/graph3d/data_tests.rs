use super::{
    anchor_zoom_scale_for_distance,
    animate_layout_nodes,
    apply_node_view_transform,
    apply_selected_node_auto_layout,
    camera_plane_to_screen_px,
    camera_plane_view_center_weight,
    length,
    node_detail_dimensions_px,
    node_detail_tier,
    pixel_scale_for_distance,
    required_camera_plane_half_extents_px,
    world_to_camera_space,
    CameraBasis,
    EdgeRef3D,
    EdgeVisualState,
    GraphThemeSettings,
    Layout3D,
    Node3D,
    NodeCardProfile,
    NodeDetailTier,
    NodeViewTransform,
    CAMERA_PLANE_MIN_CENTER_PIXEL_SCALE,
    EDGE_FLAG_DEFAULT,
    EDGE_FLAG_DIMMED,
    EDGE_FLAG_HOVERED,
    EDGE_FLAG_SELECTED,
    EDGE_INST_FLOATS,
    GRID_LINE_COUNT,
};
use crate::graph3d::camera::{
    Camera,
    CAMERA_FOV,
};

fn sample_nodes() -> Vec<Node3D> {
    vec![
        Node3D {
            id: "root".into(),
            label: None,
            state: None,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        Node3D {
            id: "child".into(),
            label: None,
            state: None,
            x: 2.0,
            y: 0.0,
            z: 0.0,
        },
    ]
}

fn sample_edges() -> Vec<EdgeRef3D> {
    vec![EdgeRef3D {
        from_idx: 0,
        to_idx: 1,
        kind: "depends_on".into(),
    }]
}

fn focus_nodes() -> Vec<Node3D> {
    vec![
        Node3D {
            id: "root".into(),
            label: None,
            state: None,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        Node3D {
            id: "child".into(),
            label: None,
            state: None,
            x: 2.0,
            y: 0.0,
            z: 0.0,
        },
        Node3D {
            id: "leaf".into(),
            label: None,
            state: None,
            x: 4.0,
            y: 0.0,
            z: 0.0,
        },
        Node3D {
            id: "other".into(),
            label: None,
            state: None,
            x: 6.0,
            y: 0.0,
            z: 0.0,
        },
    ]
}

fn focus_edges() -> Vec<EdgeRef3D> {
    vec![
        EdgeRef3D {
            from_idx: 0,
            to_idx: 1,
            kind: "depends_on".into(),
        },
        EdgeRef3D {
            from_idx: 2,
            to_idx: 3,
            kind: "blocks".into(),
        },
    ]
}

fn graph_edge_flag(
    edge_data: &[f32],
    edge_index: usize,
) -> f32 {
    edge_data[(GRID_LINE_COUNT + edge_index) * EDGE_INST_FLOATS + 10]
}

#[test]
fn compact_profile_uses_compact_directed_edge_type() {
    let layout = Layout3D::new(sample_nodes(), sample_edges());
    let (edge_data, edge_count) = layout.build_edge_instances();

    assert_eq!(edge_count as usize, GRID_LINE_COUNT + 1);
    let edge_type = edge_data[(GRID_LINE_COUNT + 1) * EDGE_INST_FLOATS - 1];
    assert_eq!(edge_type, 1.0);
}

#[test]
fn ticket_wide_profile_uses_ticket_directed_edge_type() {
    let layout = Layout3D::new(sample_nodes(), sample_edges())
        .with_node_card_profile(NodeCardProfile::TicketWide);
    let (edge_data, edge_count) = layout.build_edge_instances();

    assert_eq!(edge_count as usize, GRID_LINE_COUNT + 1);
    let edge_type = edge_data[(GRID_LINE_COUNT + 1) * EDGE_INST_FLOATS - 1];
    assert_eq!(edge_type, 2.0);
}

#[test]
fn selected_focus_marks_incident_edges_and_dims_the_rest() {
    let layout = Layout3D::new(focus_nodes(), focus_edges());
    let (edge_data, edge_count) = layout
        .build_edge_instances_with_visual_state(
            EdgeVisualState {
                selected_node_id: Some("root"),
                hovered_node_id: Some("leaf"),
            },
            &GraphThemeSettings::default(),
        );

    assert_eq!(edge_count as usize, GRID_LINE_COUNT + 2);
    assert_eq!(graph_edge_flag(&edge_data, 0), EDGE_FLAG_SELECTED);
    assert_eq!(graph_edge_flag(&edge_data, 1), EDGE_FLAG_DIMMED);
}

#[test]
fn hovered_focus_is_used_when_no_selection_exists() {
    let layout = Layout3D::new(focus_nodes(), focus_edges());
    let (edge_data, edge_count) = layout
        .build_edge_instances_with_visual_state(
            EdgeVisualState {
                selected_node_id: None,
                hovered_node_id: Some("leaf"),
            },
            &GraphThemeSettings::default(),
        );

    assert_eq!(edge_count as usize, GRID_LINE_COUNT + 2);
    assert_eq!(graph_edge_flag(&edge_data, 0), EDGE_FLAG_DIMMED);
    assert_eq!(graph_edge_flag(&edge_data, 1), EDGE_FLAG_HOVERED);
}

#[test]
fn stale_hover_focus_is_ignored() {
    let layout = Layout3D::new(focus_nodes(), focus_edges());
    let (edge_data, edge_count) = layout
        .build_edge_instances_with_visual_state(
            EdgeVisualState {
                selected_node_id: None,
                hovered_node_id: Some("missing-node"),
            },
            &GraphThemeSettings::default(),
        );

    assert_eq!(edge_count as usize, GRID_LINE_COUNT + 2);
    assert_eq!(graph_edge_flag(&edge_data, 0), EDGE_FLAG_DEFAULT);
    assert_eq!(graph_edge_flag(&edge_data, 1), EDGE_FLAG_DEFAULT);
}

#[test]
fn gpu_edge_instances_keep_only_grid_lines() {
    let layout = Layout3D::new(sample_nodes(), sample_edges());
    let (edge_data, edge_count) = layout.build_gpu_edge_instances();

    assert_eq!(edge_count as usize, GRID_LINE_COUNT);
    assert_eq!(edge_data.len(), GRID_LINE_COUNT * EDGE_INST_FLOATS);
}

#[test]
fn node_detail_tier_thresholds_preserve_selected_rich_detail() {
    let theme = GraphThemeSettings::default();
    assert_eq!(
        node_detail_tier(0.20, true, false, &theme),
        NodeDetailTier::Rich
    );
    assert_eq!(
        node_detail_tier(0.20, false, false, &theme),
        NodeDetailTier::Icon
    );
    assert_eq!(
        node_detail_tier(0.34, false, false, &theme),
        NodeDetailTier::Label
    );
    assert_eq!(
        node_detail_tier(0.72, false, false, &theme),
        NodeDetailTier::Rich
    );
}

#[test]
fn node_detail_tier_selection() {
    let theme = GraphThemeSettings::default();
    assert_eq!(
        node_detail_tier(0.20, true, false, &theme),
        NodeDetailTier::Rich
    );
    assert_eq!(
        node_detail_tier(0.20, false, false, &theme),
        NodeDetailTier::Icon
    );
    assert_eq!(
        node_detail_tier(0.34, false, false, &theme),
        NodeDetailTier::Label
    );
    assert_eq!(
        node_detail_tier(0.50, false, false, &theme),
        NodeDetailTier::Compact
    );
    assert_eq!(
        node_detail_tier(0.72, false, false, &theme),
        NodeDetailTier::Rich
    );
}

#[test]
fn node_detail_tier_hover_promotion() {
    let theme = GraphThemeSettings::default();
    assert_eq!(
        node_detail_tier(0.10, false, true, &theme),
        NodeDetailTier::Icon
    );
    assert_eq!(
        node_detail_tier(0.20, false, true, &theme),
        NodeDetailTier::Label
    );
    assert_eq!(
        node_detail_tier(0.34, false, true, &theme),
        NodeDetailTier::Compact
    );
    assert_eq!(
        node_detail_tier(0.50, false, true, &theme),
        NodeDetailTier::Rich
    );
}

#[test]
fn render_tuning_keeps_focus_rich_at_ticket_scales() {
    let mut theme = GraphThemeSettings::default();
    theme.render_tuning.rich_detail_threshold = 1.08;

    assert_eq!(
        node_detail_tier(1.066, false, false, &theme),
        NodeDetailTier::Compact
    );
    assert_eq!(
        node_detail_tier(1.104, false, false, &theme),
        NodeDetailTier::Rich
    );
    assert_eq!(
        node_detail_tier(1.066, true, false, &theme),
        NodeDetailTier::Rich
    );
    assert_eq!(
        node_detail_tier(1.066, false, true, &theme),
        NodeDetailTier::Rich
    );
}

#[test]
fn anchor_zoom_scale_uses_default_and_ticket_render_tuning() {
    let distance = 110.6469;
    let default_theme = GraphThemeSettings::default();
    assert!(
        (anchor_zoom_scale_for_distance(
            "right-center",
            distance,
            &default_theme
        ) - 0.4008)
            .abs()
            < 0.0001
    );

    let mut ticket_theme = GraphThemeSettings::default();
    ticket_theme.render_tuning.row_label_scale_numerator = 13.0;
    ticket_theme.render_tuning.row_label_boost_factor = 0.0;
    assert!(
        (anchor_zoom_scale_for_distance(
            "right-center",
            distance,
            &ticket_theme
        ) - 0.1175)
            .abs()
            < 0.0001
    );
}

#[test]
fn node_detail_dimensions_follow_profile_and_tier() {
    assert_eq!(
        node_detail_dimensions_px(
            NodeDetailTier::Minimal,
            NodeCardProfile::TicketWide,
        ),
        [52.0, 52.0],
    );
    assert_eq!(
        node_detail_dimensions_px(
            NodeDetailTier::Compact,
            NodeCardProfile::TicketWide,
        ),
        [176.0, 72.0],
    );
    assert_eq!(
        node_detail_dimensions_px(
            NodeDetailTier::Compact,
            NodeCardProfile::Compact,
        ),
        [172.0, 92.0],
    );
    assert_eq!(
        node_detail_dimensions_px(
            NodeDetailTier::Rich,
            NodeCardProfile::TicketWide,
        ),
        [212.0, 132.0],
    );
}

#[test]
fn auto_layout_selected_node_pushes_other_nodes_outward() {
    let layout = Layout3D::new(focus_nodes(), focus_edges());
    let focused = apply_selected_node_auto_layout(&layout, Some("root"), true);

    assert!(focused.nodes[1].x > layout.nodes[1].x);
    assert!(focused.nodes[2].x > layout.nodes[2].x);
    assert!(focused.nodes[0].x == layout.nodes[0].x);
}

#[test]
fn layout_animation_moves_nodes_toward_targets() {
    let mut current = Layout3D::new(sample_nodes(), sample_edges());
    let mut target = current.clone();
    target.nodes[1].x = 8.0;
    target.nodes[1].y = 3.0;

    let moved = animate_layout_nodes(&mut current, &target, 1.0 / 60.0, 12.0);
    assert!(moved);
    assert!(current.nodes[1].x > 2.0);
    assert!(current.nodes[1].x < 8.0);
    assert!(current.nodes[1].y > 0.0);
}

#[test]
fn camera_plane_transform_flattens_nodes_to_one_depth() {
    let layout = Layout3D::new(
        vec![
            Node3D {
                id: "a".into(),
                label: None,
                state: None,
                x: -3.0,
                y: 1.5,
                z: -4.0,
            },
            Node3D {
                id: "b".into(),
                label: None,
                state: None,
                x: 4.0,
                y: -2.0,
                z: 6.0,
            },
            Node3D {
                id: "c".into(),
                label: None,
                state: None,
                x: 1.0,
                y: 3.0,
                z: 1.0,
            },
        ],
        Vec::new(),
    );
    let camera = Camera {
        yaw: 0.45,
        pitch: 0.3,
        distance: 28.0,
        target: [0.0, 0.0, 0.0],
    };

    let transformed = apply_node_view_transform(
        &layout,
        &camera,
        1280.0,
        720.0,
        NodeViewTransform::camera_plane(0.84),
    );
    let basis = CameraBasis::from_camera(&camera);
    let depths: Vec<f32> = transformed
        .nodes
        .iter()
        .map(|node| world_to_camera_space([node.x, node.y, node.z], &basis)[2])
        .collect();
    let min_depth = depths.iter().copied().fold(f32::INFINITY, f32::min);
    let max_depth = depths.iter().copied().fold(f32::NEG_INFINITY, f32::max);

    assert!((max_depth - min_depth) < 0.001);
}

#[test]
fn camera_plane_transform_tracks_camera_orientation() {
    let layout = Layout3D::new(sample_nodes(), sample_edges());
    let transform = NodeViewTransform::camera_plane(0.82);
    let first = apply_node_view_transform(
        &layout,
        &Camera {
            yaw: 0.1,
            pitch: 0.25,
            distance: 24.0,
            target: [0.0, 0.0, 0.0],
        },
        1280.0,
        720.0,
        transform,
    );
    let second = apply_node_view_transform(
        &layout,
        &Camera {
            yaw: 0.9,
            pitch: 0.25,
            distance: 24.0,
            target: [0.0, 0.0, 0.0],
        },
        1280.0,
        720.0,
        transform,
    );

    assert_ne!(first.nodes[1].x, second.nodes[1].x);
    assert_ne!(first.nodes[1].z, second.nodes[1].z);
}

#[test]
fn camera_plane_transform_clears_center_overlap() {
    let layout = Layout3D::new(
        vec![
            Node3D {
                id: "center-a".into(),
                label: None,
                state: None,
                x: -0.08,
                y: 0.0,
                z: 0.0,
            },
            Node3D {
                id: "center-b".into(),
                label: None,
                state: None,
                x: 0.08,
                y: 0.0,
                z: 0.0,
            },
            Node3D {
                id: "edge".into(),
                label: None,
                state: None,
                x: 16.0,
                y: -4.0,
                z: 0.0,
            },
        ],
        Vec::new(),
    );
    let camera = Camera {
        yaw: 0.3,
        pitch: 0.2,
        distance: 120.0,
        target: [0.0, 0.0, 0.0],
    };
    let viewport_width = 1280.0;
    let viewport_height = 720.0;
    let transformed = apply_node_view_transform(
        &layout,
        &camera,
        viewport_width,
        viewport_height,
        NodeViewTransform::camera_plane(0.84),
    );
    let basis = CameraBasis::from_camera(&camera);
    let tan_half_fov = (CAMERA_FOV * 0.5).tan();
    let aspect = viewport_width / viewport_height;

    let camera_positions: Vec<[f32; 3]> = transformed
        .nodes
        .iter()
        .map(|node| world_to_camera_space([node.x, node.y, node.z], &basis))
        .collect();
    let screen_positions: Vec<[f32; 2]> = camera_positions
        .iter()
        .map(|position| {
            camera_plane_to_screen_px(
                [position[0], position[1]],
                position[2],
                viewport_width,
                viewport_height,
                tan_half_fov,
                aspect,
            )
        })
        .collect();

    let center_a_weight = camera_plane_view_center_weight(
        screen_positions[0],
        viewport_width,
        viewport_height,
    );
    let center_b_weight = camera_plane_view_center_weight(
        screen_positions[1],
        viewport_width,
        viewport_height,
    );
    let required_a = required_camera_plane_half_extents_px(
        camera_positions[0],
        NodeCardProfile::Compact,
        center_a_weight,
    );
    let required_b = required_camera_plane_half_extents_px(
        camera_positions[1],
        NodeCardProfile::Compact,
        center_b_weight,
    );
    let dx = (screen_positions[1][0] - screen_positions[0][0]).abs();
    let dy = (screen_positions[1][1] - screen_positions[0][1]).abs();

    assert!(
        dx >= required_a[0] + required_b[0] - 1.0
            || dy >= required_a[1] + required_b[1] - 1.0
    );
}

#[test]
fn camera_plane_transform_keeps_center_nodes_readable() {
    let layout = Layout3D::new(
        vec![Node3D {
            id: "center".into(),
            label: None,
            state: None,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }],
        Vec::new(),
    );
    let camera = Camera {
        yaw: 0.3,
        pitch: 0.4,
        distance: 140.0,
        target: [0.0, 0.0, 0.0],
    };

    let transformed = apply_node_view_transform(
        &layout,
        &camera,
        1280.0,
        720.0,
        NodeViewTransform::camera_plane(0.84),
    );
    let basis = CameraBasis::from_camera(&camera);
    let camera_position = world_to_camera_space(
        [
            transformed.nodes[0].x,
            transformed.nodes[0].y,
            transformed.nodes[0].z,
        ],
        &basis,
    );

    assert!(
        pixel_scale_for_distance(length(camera_position))
            >= CAMERA_PLANE_MIN_CENTER_PIXEL_SCALE - 0.01
    );
}

#[test]
fn camera_plane_transform_strength_softens_low_influence() {
    let layout = Layout3D::new(sample_nodes(), sample_edges());
    let camera = Camera {
        yaw: 0.45,
        pitch: 0.3,
        distance: 28.0,
        target: [0.0, 0.0, 0.0],
    };
    let weak = apply_node_view_transform(
        &layout,
        &camera,
        1280.0,
        720.0,
        NodeViewTransform::camera_plane_with_strength(0.82, 0.08),
    );
    let strong = apply_node_view_transform(
        &layout,
        &camera,
        1280.0,
        720.0,
        NodeViewTransform::camera_plane_with_strength(0.82, 1.0),
    );

    let weak_shift = layout
        .nodes
        .iter()
        .zip(weak.nodes.iter())
        .map(|(base, moved)| {
            length([moved.x - base.x, moved.y - base.y, moved.z - base.z])
        })
        .sum::<f32>();
    let strong_shift = layout
        .nodes
        .iter()
        .zip(strong.nodes.iter())
        .map(|(base, moved)| {
            length([moved.x - base.x, moved.y - base.y, moved.z - base.z])
        })
        .sum::<f32>();

    assert!(weak_shift > 0.0);
    assert!(weak_shift < strong_shift * 0.15);
}

#[test]
fn camera_plane_view_direction_transform_ignores_zoom_and_pan() {
    let layout = Layout3D::new(sample_nodes(), sample_edges());
    let transform = NodeViewTransform::camera_plane_view_direction(0.82, 1.0);
    let first = apply_node_view_transform(
        &layout,
        &Camera {
            yaw: 0.35,
            pitch: 0.2,
            distance: 14.0,
            target: [0.0, 0.0, 0.0],
        },
        1280.0,
        720.0,
        transform,
    );
    let second = apply_node_view_transform(
        &layout,
        &Camera {
            yaw: 0.35,
            pitch: 0.2,
            distance: 52.0,
            target: [18.0, -7.0, 11.0],
        },
        1280.0,
        720.0,
        transform,
    );

    for (left, right) in first.nodes.iter().zip(second.nodes.iter()) {
        assert!(
            (left.x - right.x).abs() < 0.001,
            "x mismatch: {left:?} vs {right:?}"
        );
        assert!(
            (left.y - right.y).abs() < 0.001,
            "y mismatch: {left:?} vs {right:?}"
        );
        assert!(
            (left.z - right.z).abs() < 0.001,
            "z mismatch: {left:?} vs {right:?}"
        );
    }
}
