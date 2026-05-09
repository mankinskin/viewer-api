//! Data types for the shared 3-D graph view.
//!
//! Domain-agnostic: a node is just a positioned id with optional label/state,
//! and an edge is a typed reference between two node indices.

use std::collections::HashSet;

use super::theme::GraphThemeSettings;

/// One node in 3-D world space.
#[derive(Debug, Clone, PartialEq)]
pub struct Node3D {
    pub id: String,
    pub label: Option<String>,
    pub state: Option<String>,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// One edge between two nodes (referenced by index into `Layout3D::nodes`).
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeRef3D {
    pub from_idx: usize,
    pub to_idx: usize,
    pub kind: String,
}

/// Screen-space node-card footprint profile for a shared Graph3D layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NodeCardProfile {
    /// Shared compact graph cards, currently used by spec-viewer.
    #[default]
    Compact,
    /// Wider ticket cards with a fixed two-row summary layout.
    TicketWide,
}

/// A complete 3-D graph: positioned nodes + indexed edges.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout3D {
    pub nodes: Vec<Node3D>,
    pub edges: Vec<EdgeRef3D>,
    pub node_card_profile: NodeCardProfile,
}

/// Per-edge instance floats (posA[3]+posB[3]+color[4]+flags[1]+edgeType[1]).
pub(crate) const EDGE_INST_FLOATS: usize = 12;
pub(crate) const EDGE_FLAG_DIMMED: f32 = -1.0;
pub(crate) const EDGE_FLAG_DEFAULT: f32 = 0.0;
pub(crate) const EDGE_FLAG_HOVERED: f32 = 1.0;
pub(crate) const EDGE_FLAG_SELECTED: f32 = 2.0;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct EdgeVisualState<'a> {
    pub selected_node_id: Option<&'a str>,
    pub hovered_node_id: Option<&'a str>,
}

impl<'a> EdgeVisualState<'a> {
    pub(crate) fn active_focus(
        self,
        nodes: &[Node3D],
    ) -> Option<(&'a str, f32)> {
        self.selected_node_id
            .map(|node_id| (node_id, EDGE_FLAG_SELECTED))
            .or_else(|| {
                self.hovered_node_id
                    .map(|node_id| (node_id, EDGE_FLAG_HOVERED))
            })
            .filter(|(focus_id, _)| {
                nodes.iter().any(|node| node.id == *focus_id)
            })
    }
}

/// Half-extent of the coordinate grid (world units).
const GRID_HALF: f32 = 30.0;
/// Number of grid lines we generate (used for buffer pre-sizing).
const GRID_LINE_COUNT: usize = ((GRID_HALF as i32) * 2 + 1) as usize * 2;

fn grid_line_color(coord: f32) -> (f32, f32, f32, f32) {
    // Axis (coord == 0): brighter cool colour.
    if coord.abs() < 0.001 {
        return (0.28, 0.34, 0.46, 0.18);
    }
    // Major gridline every 5 units: mid alpha.
    if (coord.rem_euclid(5.0)).abs() < 0.01
        || (coord.rem_euclid(5.0) - 5.0).abs() < 0.01
    {
        return (0.22, 0.26, 0.36, 0.10);
    }
    // Minor: dim.
    (0.18, 0.20, 0.28, 0.045)
}

impl Layout3D {
    pub fn new(
        nodes: Vec<Node3D>,
        edges: Vec<EdgeRef3D>,
    ) -> Self {
        Self {
            nodes,
            edges,
            node_card_profile: NodeCardProfile::default(),
        }
    }

    pub fn with_node_card_profile(
        mut self,
        node_card_profile: NodeCardProfile,
    ) -> Self {
        self.node_card_profile = node_card_profile;
        self
    }

    /// Bounding-sphere centre and radius of all nodes (for camera framing).
    pub fn bounds(&self) -> ([f32; 3], f32) {
        if self.nodes.is_empty() {
            return ([0.0, 0.0, 0.0], 1.0);
        }
        let n = self.nodes.len() as f32;
        let cx = self.nodes.iter().map(|n| n.x).sum::<f32>() / n;
        let cy = self.nodes.iter().map(|n| n.y).sum::<f32>() / n;
        let cz = self.nodes.iter().map(|n| n.z).sum::<f32>() / n;
        let radius = self
            .nodes
            .iter()
            .map(|nd| {
                let dx = nd.x - cx;
                let dy = nd.y - cy;
                let dz = nd.z - cz;
                (dx * dx + dy * dy + dz * dz).sqrt()
            })
            .fold(0.0_f32, f32::max);
        ([cx, cy, cz], radius.max(1.0))
    }

    /// Build the flat per-instance edge buffer the GPU consumes.
    ///
    /// Includes a coordinate grid on the y=0 plane (rendered as
    /// `edgeType = 0` thin AA lines) followed by the actual graph edges.
    /// Matches the TS reference (`pipeline.ts buildGridData()`).
    pub(crate) fn build_edge_instances(&self) -> (Vec<f32>, u32) {
        self.build_edge_instances_with_visual_state(
            EdgeVisualState::default(),
            &GraphThemeSettings::default(),
        )
    }

    pub(crate) fn build_edge_instances_with_visual_state(
        &self,
        visuals: EdgeVisualState<'_>,
        theme: &GraphThemeSettings,
    ) -> (Vec<f32>, u32) {
        let mut data = Vec::with_capacity(
            (self.edges.len() + GRID_LINE_COUNT) * EDGE_INST_FLOATS,
        );
        let mut count = 0u32;
        let graph_edge_type = match self.node_card_profile {
            NodeCardProfile::Compact => 1.0,
            NodeCardProfile::TicketWide => 2.0,
        };
        let active_focus = visuals.active_focus(&self.nodes);

        // ── Coordinate grid (y = 0 plane) ──
        // Step 1 world-unit, extent ±GRID_HALF on each axis.
        let half = GRID_HALF;
        let step = 1.0_f32;
        let mut z = -half;
        while z <= half + 0.0001 {
            // Highlight axis lines (z == 0) with a brighter alpha; major
            // gridlines every 5 units get a mid alpha; minor lines stay dim.
            let (r, g, b, a) = grid_line_color(z);
            data.extend_from_slice(&[
                -half, 0.0, z, half, 0.0, z, r, g, b, a, 0.0, // flags
                0.0, // edgeType = 0 (simple thin AA line)
            ]);
            count += 1;
            z += step;
        }
        let mut x = -half;
        while x <= half + 0.0001 {
            let (r, g, b, a) = grid_line_color(x);
            data.extend_from_slice(&[
                x, 0.0, -half, x, 0.0, half, r, g, b, a, 0.0, 0.0,
            ]);
            count += 1;
            x += step;
        }

        // ── Graph edges ──
        for edge in &self.edges {
            let Some(a) = self.nodes.get(edge.from_idx) else {
                continue;
            };
            let Some(b) = self.nodes.get(edge.to_idx) else {
                continue;
            };
            let (r, g, bl, alpha) = edge_color(&edge.kind, theme);
            let flag =
                edge_visual_flag(a.id.as_str(), b.id.as_str(), active_focus);
            data.extend_from_slice(&[
                a.x,
                a.y,
                a.z,
                b.x,
                b.y,
                b.z,
                r,
                g,
                bl,
                alpha,
                flag, // flags
                graph_edge_type,
            ]);
            count += 1;
        }
        (data, count)
    }

    /// Per-node occluder-quad instance buffer (xyz + pad).
    pub(crate) fn build_node_quads(&self) -> (Vec<f32>, u32) {
        let mut data = Vec::with_capacity(self.nodes.len() * 4);
        for n in &self.nodes {
            data.push(n.x);
            data.push(n.y);
            data.push(n.z);
            data.push(0.0);
        }
        (data, self.nodes.len() as u32)
    }
}

pub(crate) fn layout_nodes_match(
    current: &Layout3D,
    target: &Layout3D,
) -> bool {
    current.node_card_profile == target.node_card_profile
        && current.nodes.len() == target.nodes.len()
        && current.nodes.iter().zip(target.nodes.iter()).all(
            |(current_node, target_node)| current_node.id == target_node.id,
        )
}

pub(crate) fn animate_layout_nodes(
    current: &mut Layout3D,
    target: &Layout3D,
    dt: f32,
    lerp_speed: f32,
) -> bool {
    if !layout_nodes_match(current, target) {
        return false;
    }

    let alpha = 1.0 - (-lerp_speed * dt.max(0.0)).exp();
    let epsilon = 0.0001;
    let mut any_moved = false;
    for (current_node, target_node) in
        current.nodes.iter_mut().zip(target.nodes.iter())
    {
        let dx = (target_node.x - current_node.x) * alpha;
        let dy = (target_node.y - current_node.y) * alpha;
        let dz = (target_node.z - current_node.z) * alpha;
        current_node.x += dx;
        current_node.y += dy;
        current_node.z += dz;
        if dx.abs() > epsilon || dy.abs() > epsilon || dz.abs() > epsilon {
            any_moved = true;
        }
    }

    if !any_moved {
        for (current_node, target_node) in
            current.nodes.iter_mut().zip(target.nodes.iter())
        {
            current_node.x = target_node.x;
            current_node.y = target_node.y;
            current_node.z = target_node.z;
        }
    }

    any_moved
}

pub(crate) fn apply_selected_node_auto_layout(
    layout: &Layout3D,
    selected_node_id: Option<&str>,
    enabled: bool,
) -> Layout3D {
    if !enabled {
        return layout.clone();
    }

    let Some(selected_idx) = selected_node_id.and_then(|selected_id| {
        layout.nodes.iter().position(|node| node.id == selected_id)
    }) else {
        return layout.clone();
    };

    let selected = &layout.nodes[selected_idx];
    let mut incident_nodes = HashSet::new();
    for edge in &layout.edges {
        if edge.from_idx == selected_idx {
            incident_nodes.insert(edge.to_idx);
        }
        if edge.to_idx == selected_idx {
            incident_nodes.insert(edge.from_idx);
        }
    }

    let mut focused = layout.clone();
    for (index, node) in focused.nodes.iter_mut().enumerate() {
        if index == selected_idx {
            continue;
        }

        let base = &layout.nodes[index];
        let mut dx = base.x - selected.x;
        let mut dy = base.y - selected.y;
        let mut dz = base.z - selected.z;
        let mut len = (dx * dx + dy * dy + dz * dz).sqrt();
        if len < 0.001 {
            let angle = (index as f32) * 2.399_963_1;
            dx = angle.cos();
            dz = angle.sin();
            dy = 0.0;
            len = 1.0;
        }

        let incident = incident_nodes.contains(&index);
        let push_scale = if incident { 1.24 } else { 1.10 };
        let vertical_bias = if incident { 0.28 } else { -0.08 };
        let inv_len = 1.0 / len;
        let nx = dx * inv_len;
        let ny = dy * inv_len;
        let nz = dz * inv_len;
        let pushed_distance = len * push_scale;

        node.x = selected.x + nx * pushed_distance;
        node.y = selected.y + ny * pushed_distance + vertical_bias;
        node.z = selected.z + nz * pushed_distance;
    }

    focused
}

pub(crate) fn edge_color(
    kind: &str,
    theme: &GraphThemeSettings,
) -> (f32, f32, f32, f32) {
    let color = theme.edge_color(kind);
    (color[0], color[1], color[2], color[3])
}

fn edge_visual_flag(
    from_id: &str,
    to_id: &str,
    active_focus: Option<(&str, f32)>,
) -> f32 {
    let Some((focus_id, focus_flag)) = active_focus else {
        return EDGE_FLAG_DEFAULT;
    };

    if from_id == focus_id || to_id == focus_id {
        focus_flag
    } else {
        EDGE_FLAG_DIMMED
    }
}

#[cfg(test)]
mod tests {
    use super::{
        animate_layout_nodes,
        apply_selected_node_auto_layout,
        EdgeRef3D,
        EdgeVisualState,
        GraphThemeSettings,
        Layout3D,
        Node3D,
        NodeCardProfile,
        EDGE_FLAG_DEFAULT,
        EDGE_FLAG_DIMMED,
        EDGE_FLAG_HOVERED,
        EDGE_FLAG_SELECTED,
        EDGE_INST_FLOATS,
        GRID_LINE_COUNT,
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
    fn auto_layout_selected_node_pushes_other_nodes_outward() {
        let layout = Layout3D::new(focus_nodes(), focus_edges());
        let focused =
            apply_selected_node_auto_layout(&layout, Some("root"), true);

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

        let moved =
            animate_layout_nodes(&mut current, &target, 1.0 / 60.0, 12.0);
        assert!(moved);
        assert!(current.nodes[1].x > 2.0);
        assert!(current.nodes[1].x < 8.0);
        assert!(current.nodes[1].y > 0.0);
    }
}
