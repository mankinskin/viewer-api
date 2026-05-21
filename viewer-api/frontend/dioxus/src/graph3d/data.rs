//! Data types for the shared 3-D graph view.
//!
//! Domain-agnostic: a node is just a positioned id with optional label/state,
//! and an edge is a typed reference between two node indices.

use std::collections::HashSet;

use super::{
    camera::{
        Camera,
        CAMERA_FOV,
    },
    theme::GraphThemeSettings,
};

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

/// Optional per-frame node transform applied in the renderer after layout.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NodeViewTransform {
    pub mode: NodeViewTransformMode,
    /// Target viewport fill ratio for the flattened plane.
    pub screen_fill: f32,
    /// Depth of the screen plane relative to the current camera distance.
    pub plane_depth_factor: f32,
    /// Blend factor from the original layout into the transformed layout.
    pub strength: f32,
}

impl NodeViewTransform {
    pub fn camera_plane(screen_fill: f32) -> Self {
        Self::camera_plane_with_strength(screen_fill, 1.0)
    }

    pub fn camera_plane_with_strength(
        screen_fill: f32,
        strength: f32,
    ) -> Self {
        Self {
            mode: NodeViewTransformMode::CameraPlane,
            screen_fill: screen_fill.clamp(0.55, 0.96),
            plane_depth_factor: 0.56,
            strength: strength.clamp(0.0, 1.0),
        }
    }

    pub fn camera_plane_view_direction(
        screen_fill: f32,
        strength: f32,
    ) -> Self {
        Self {
            mode: NodeViewTransformMode::CameraPlaneViewDirection,
            screen_fill: screen_fill.clamp(0.55, 3.0),
            plane_depth_factor: 0.56,
            strength: strength.clamp(0.0, 1.0),
        }
    }

    pub fn is_active(self) -> bool {
        !matches!(self.mode, NodeViewTransformMode::Disabled)
            && self.strength > 0.001
    }
}

const CAMERA_PLANE_CLEARANCE_PASSES: usize = 14;
const CAMERA_PLANE_COMPACT_CARD_BASE_WIDTH_PX: f32 = 245.0;
const CAMERA_PLANE_COMPACT_CARD_BASE_HEIGHT_PX: f32 = 196.0;
const CAMERA_PLANE_TICKET_CARD_BASE_WIDTH_PX: f32 = 260.0;
const CAMERA_PLANE_TICKET_CARD_BASE_HEIGHT_PX: f32 = 56.0;
const CAMERA_PLANE_CENTER_RADIUS: f32 = 0.30;
const CAMERA_PLANE_EDGE_FOOTPRINT_SCALE: f32 = 0.74;
const CAMERA_PLANE_CENTER_FOOTPRINT_SCALE: f32 = 1.08;
const CAMERA_PLANE_EDGE_BORDER_X_PX: f32 = 2.0;
const CAMERA_PLANE_EDGE_BORDER_Y_PX: f32 = 2.0;
const CAMERA_PLANE_CENTER_BORDER_X_PX: f32 = 18.0;
const CAMERA_PLANE_CENTER_BORDER_Y_PX: f32 = 14.0;
const CAMERA_PLANE_MAX_PUSH_PX: f32 = 96.0;
const CAMERA_PLANE_MIN_CENTER_PIXEL_SCALE: f32 = 0.46;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NodeViewTransformMode {
    #[default]
    Disabled,
    CameraPlane,
    CameraPlaneViewDirection,
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

    pub(crate) fn build_gpu_edge_instances(&self) -> (Vec<f32>, u32) {
        let mut data = Vec::with_capacity(GRID_LINE_COUNT * EDGE_INST_FLOATS);
        let count = append_grid_edge_instances(&mut data);
        (data, count)
    }

    pub(crate) fn build_edge_instances_with_visual_state(
        &self,
        visuals: EdgeVisualState<'_>,
        theme: &GraphThemeSettings,
    ) -> (Vec<f32>, u32) {
        let mut data = Vec::with_capacity(
            (self.edges.len() + GRID_LINE_COUNT) * EDGE_INST_FLOATS,
        );
        let mut count = append_grid_edge_instances(&mut data);
        let graph_edge_type = match self.node_card_profile {
            NodeCardProfile::Compact => 1.0,
            NodeCardProfile::TicketWide => 2.0,
        };
        let active_focus = visuals.active_focus(&self.nodes);

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

fn append_grid_edge_instances(data: &mut Vec<f32>) -> u32 {
    let mut count = 0u32;

    // Coordinate grid on the y=0 plane: the DOM/SVG overlay owns graph-edge
    // styling, while the GPU pass keeps only the shared grid underneath it.
    let half = GRID_HALF;
    let step = 1.0_f32;
    let mut z = -half;
    while z <= half + 0.0001 {
        let (r, g, b, a) = grid_line_color(z);
        data.extend_from_slice(&[
            -half, 0.0, z, half, 0.0, z, r, g, b, a, 0.0, 0.0,
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

    count
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

pub(crate) fn apply_node_view_transform(
    layout: &Layout3D,
    camera: &Camera,
    viewport_width: f32,
    viewport_height: f32,
    transform: NodeViewTransform,
) -> Layout3D {
    if !transform.is_active() || layout.nodes.is_empty() {
        return layout.clone();
    }

    match transform.mode {
        NodeViewTransformMode::Disabled => layout.clone(),
        NodeViewTransformMode::CameraPlane => {
            let basis = CameraBasis::from_camera(camera);
            let aspect = viewport_width / viewport_height.max(1.0);
            let plane_depth = (camera.distance * transform.plane_depth_factor)
                .clamp(3.5, 72.0)
                .min(camera_plane_max_readable_depth());
            let plane_half_height =
                (CAMERA_FOV * 0.5).tan() * plane_depth * transform.screen_fill;
            let plane_half_width = plane_half_height * aspect.max(0.2);

            let mut camera_space = Vec::with_capacity(layout.nodes.len());
            let mut min_x = f32::INFINITY;
            let mut max_x = f32::NEG_INFINITY;
            let mut min_y = f32::INFINITY;
            let mut max_y = f32::NEG_INFINITY;

            for node in &layout.nodes {
                let position =
                    world_to_camera_space([node.x, node.y, node.z], &basis);
                min_x = min_x.min(position[0]);
                max_x = max_x.max(position[0]);
                min_y = min_y.min(position[1]);
                max_y = max_y.max(position[1]);
                camera_space.push(position);
            }

            let center_x = (min_x + max_x) * 0.5;
            let center_y = (min_y + max_y) * 0.5;
            let half_width = ((max_x - min_x) * 0.5).max(0.001);
            let half_height = ((max_y - min_y) * 0.5).max(0.001);
            let scale = (plane_half_width / half_width)
                .min(plane_half_height / half_height)
                .clamp(0.15, 6.0);
            let mut plane_positions: Vec<[f32; 2]> = camera_space
                .iter()
                .map(|position| {
                    [
                        (position[0] - center_x) * scale,
                        (position[1] - center_y) * scale,
                    ]
                })
                .collect();

            relax_camera_plane_clearance(
                plane_positions.as_mut_slice(),
                plane_depth,
                viewport_width,
                viewport_height,
                layout.node_card_profile,
            );
            stabilize_camera_plane_center_cells(
                plane_positions.as_mut_slice(),
                plane_depth,
                viewport_width,
                viewport_height,
                layout.node_card_profile,
            );
            stabilize_camera_plane_center_cells(
                plane_positions.as_mut_slice(),
                plane_depth,
                viewport_width,
                viewport_height,
                layout.node_card_profile,
            );

            let mut transformed = layout.clone();
            for (node, plane_position) in
                transformed.nodes.iter_mut().zip(plane_positions.iter())
            {
                let base_position = [node.x, node.y, node.z];
                let world = camera_to_world_space(
                    [plane_position[0], plane_position[1], plane_depth],
                    &basis,
                );
                node.x = base_position[0]
                    + (world[0] - base_position[0]) * transform.strength;
                node.y = base_position[1]
                    + (world[1] - base_position[1]) * transform.strength;
                node.z = base_position[2]
                    + (world[2] - base_position[2]) * transform.strength;
            }

            transformed
        },
        NodeViewTransformMode::CameraPlaneViewDirection => {
            let basis = camera_plane_view_direction_basis(
                layout,
                camera,
                transform,
                viewport_width,
                viewport_height,
            );
            let aspect = viewport_width / viewport_height.max(1.0);
            let plane_depth = basis.plane_depth;
            let plane_half_height =
                (CAMERA_FOV * 0.5).tan() * plane_depth * transform.screen_fill;
            let plane_half_width = plane_half_height * aspect.max(0.2);

            let mut camera_space = Vec::with_capacity(layout.nodes.len());
            let mut min_x = f32::INFINITY;
            let mut max_x = f32::NEG_INFINITY;
            let mut min_y = f32::INFINITY;
            let mut max_y = f32::NEG_INFINITY;

            for node in &layout.nodes {
                let position = world_to_camera_space(
                    [node.x, node.y, node.z],
                    &basis.camera,
                );
                min_x = min_x.min(position[0]);
                max_x = max_x.max(position[0]);
                min_y = min_y.min(position[1]);
                max_y = max_y.max(position[1]);
                camera_space.push(position);
            }

            let center_x = (min_x + max_x) * 0.5;
            let center_y = (min_y + max_y) * 0.5;
            let half_width = ((max_x - min_x) * 0.5).max(0.001);
            let half_height = ((max_y - min_y) * 0.5).max(0.001);
            let scale = (plane_half_width / half_width)
                .min(plane_half_height / half_height)
                .clamp(0.15, 6.0);
            let mut plane_positions: Vec<[f32; 2]> = camera_space
                .iter()
                .map(|position| {
                    [
                        (position[0] - center_x) * scale,
                        (position[1] - center_y) * scale,
                    ]
                })
                .collect();

            relax_camera_plane_clearance(
                plane_positions.as_mut_slice(),
                plane_depth,
                viewport_width,
                viewport_height,
                layout.node_card_profile,
            );
            stabilize_camera_plane_center_cells(
                plane_positions.as_mut_slice(),
                plane_depth,
                viewport_width,
                viewport_height,
                layout.node_card_profile,
            );
            stabilize_camera_plane_center_cells(
                plane_positions.as_mut_slice(),
                plane_depth,
                viewport_width,
                viewport_height,
                layout.node_card_profile,
            );

            let mut transformed = layout.clone();
            for (node, plane_position) in
                transformed.nodes.iter_mut().zip(plane_positions.iter())
            {
                let base_position = [node.x, node.y, node.z];
                let world = camera_to_world_space(
                    [plane_position[0], plane_position[1], plane_depth],
                    &basis.camera,
                );
                node.x = base_position[0]
                    + (world[0] - base_position[0]) * transform.strength;
                node.y = base_position[1]
                    + (world[1] - base_position[1]) * transform.strength;
                node.z = base_position[2]
                    + (world[2] - base_position[2]) * transform.strength;
            }

            transformed
        },
    }
}

struct CameraPlaneViewDirectionBasis {
    camera: CameraBasis,
    plane_depth: f32,
}

fn camera_plane_view_direction_basis(
    layout: &Layout3D,
    camera: &Camera,
    transform: NodeViewTransform,
    viewport_width: f32,
    viewport_height: f32,
) -> CameraPlaneViewDirectionBasis {
    let mut basis = CameraBasis::from_camera(camera);
    let aspect = viewport_width / viewport_height.max(1.0);
    let tan_half_fov = (CAMERA_FOV * 0.5).tan().max(0.001);
    let fit_fill = transform.screen_fill.clamp(0.55, 0.96);
    let (center, _) = layout.bounds();
    let mut max_right = 0.0_f32;
    let mut max_up = 0.0_f32;
    let mut min_forward = f32::INFINITY;

    for node in &layout.nodes {
        let delta =
            [node.x - center[0], node.y - center[1], node.z - center[2]];
        max_right = max_right.max(dot(delta, basis.right).abs());
        max_up = max_up.max(dot(delta, basis.up).abs());
        min_forward = min_forward.min(dot(delta, basis.forward));
    }

    let plane_depth_x =
        max_right / (fit_fill * tan_half_fov * aspect.max(0.2)).max(0.001);
    let plane_depth_y = max_up / (fit_fill * tan_half_fov).max(0.001);
    let plane_depth = plane_depth_x
        .max(plane_depth_y)
        .clamp(3.5, 72.0)
        .min(camera_plane_max_readable_depth());
    let eye_distance = (plane_depth / transform.plane_depth_factor.max(0.05))
        .max((-min_forward).max(0.0) + 1.0);
    basis.eye = [
        center[0] - basis.forward[0] * eye_distance,
        center[1] - basis.forward[1] * eye_distance,
        center[2] - basis.forward[2] * eye_distance,
    ];

    CameraPlaneViewDirectionBasis {
        camera: basis,
        plane_depth,
    }
}

fn camera_plane_max_readable_depth() -> f32 {
    22.0 / CAMERA_PLANE_MIN_CENTER_PIXEL_SCALE
}

fn relax_camera_plane_clearance(
    plane_positions: &mut [[f32; 2]],
    depth: f32,
    viewport_width: f32,
    viewport_height: f32,
    profile: NodeCardProfile,
) {
    if plane_positions.len() < 2 {
        return;
    }

    let tan_half_fov = (CAMERA_FOV * 0.5).tan();
    let aspect = viewport_width / viewport_height.max(1.0);

    for _ in 0..CAMERA_PLANE_CLEARANCE_PASSES {
        let mut pixel_offsets = vec![[0.0_f32; 2]; plane_positions.len()];
        let mut had_overlap = false;

        for i in 0..plane_positions.len() {
            let camera_i =
                [plane_positions[i][0], plane_positions[i][1], depth];
            let screen_i = camera_plane_to_screen_px(
                plane_positions[i],
                depth,
                viewport_width,
                viewport_height,
                tan_half_fov,
                aspect,
            );
            let weight_i = camera_plane_view_center_weight(
                screen_i,
                viewport_width,
                viewport_height,
            );
            let required_i = required_camera_plane_half_extents_px(
                camera_i, profile, weight_i,
            );

            for j in (i + 1)..plane_positions.len() {
                let camera_j =
                    [plane_positions[j][0], plane_positions[j][1], depth];
                let screen_j = camera_plane_to_screen_px(
                    plane_positions[j],
                    depth,
                    viewport_width,
                    viewport_height,
                    tan_half_fov,
                    aspect,
                );
                let weight_j = camera_plane_view_center_weight(
                    screen_j,
                    viewport_width,
                    viewport_height,
                );
                let required_j = required_camera_plane_half_extents_px(
                    camera_j, profile, weight_j,
                );
                let dx = screen_j[0] - screen_i[0];
                let dy = screen_j[1] - screen_i[1];
                let overlap_x = required_i[0] + required_j[0] - dx.abs();
                let overlap_y = required_i[1] + required_j[1] - dy.abs();

                if overlap_x <= 0.0 || overlap_y <= 0.0 {
                    continue;
                }

                had_overlap = true;
                let pair_strength = 0.6 + 1.0 * weight_i.max(weight_j);
                let direction_x = camera_plane_pair_axis_direction(
                    dx,
                    screen_i[0],
                    screen_j[0],
                    i,
                    j,
                );
                let direction_y = camera_plane_pair_axis_direction(
                    dy,
                    screen_i[1],
                    screen_j[1],
                    i,
                    j,
                );
                let push_x = overlap_x.min(CAMERA_PLANE_MAX_PUSH_PX)
                    * 0.38
                    * pair_strength;
                let push_y = overlap_y.min(CAMERA_PLANE_MAX_PUSH_PX)
                    * 0.38
                    * pair_strength;

                pixel_offsets[i][0] -= direction_x * push_x;
                pixel_offsets[j][0] += direction_x * push_x;
                pixel_offsets[i][1] -= direction_y * push_y;
                pixel_offsets[j][1] += direction_y * push_y;
            }
        }

        if !had_overlap {
            break;
        }

        let mut max_adjustment = 0.0_f32;
        for (plane_position, pixel_offset) in
            plane_positions.iter_mut().zip(pixel_offsets.iter())
        {
            let delta_x = screen_px_to_camera_plane_x(
                pixel_offset[0]
                    .clamp(-CAMERA_PLANE_MAX_PUSH_PX, CAMERA_PLANE_MAX_PUSH_PX),
                depth,
                viewport_width,
                tan_half_fov,
                aspect,
            );
            let delta_y = screen_px_to_camera_plane_y(
                pixel_offset[1]
                    .clamp(-CAMERA_PLANE_MAX_PUSH_PX, CAMERA_PLANE_MAX_PUSH_PX),
                depth,
                viewport_height,
                tan_half_fov,
            );
            plane_position[0] += delta_x;
            plane_position[1] += delta_y;
            max_adjustment =
                max_adjustment.max(delta_x.abs()).max(delta_y.abs());
        }
        recenter_camera_plane_positions(plane_positions);

        if max_adjustment < 0.001 {
            break;
        }
    }
}

fn recenter_camera_plane_positions(plane_positions: &mut [[f32; 2]]) {
    if plane_positions.is_empty() {
        return;
    }

    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for position in plane_positions.iter() {
        min_x = min_x.min(position[0]);
        max_x = max_x.max(position[0]);
        min_y = min_y.min(position[1]);
        max_y = max_y.max(position[1]);
    }

    let center_x = (min_x + max_x) * 0.5;
    let center_y = (min_y + max_y) * 0.5;
    for position in plane_positions.iter_mut() {
        position[0] -= center_x;
        position[1] -= center_y;
    }
}

fn stabilize_camera_plane_center_cells(
    plane_positions: &mut [[f32; 2]],
    depth: f32,
    viewport_width: f32,
    viewport_height: f32,
    profile: NodeCardProfile,
) {
    if plane_positions.len() < 2 {
        return;
    }

    let tan_half_fov = (CAMERA_FOV * 0.5).tan();
    let aspect = viewport_width / viewport_height.max(1.0);
    let mut center_cells = Vec::new();

    for (index, plane_position) in plane_positions.iter().enumerate() {
        let camera_position = [plane_position[0], plane_position[1], depth];
        let screen_position = camera_plane_to_screen_px(
            *plane_position,
            depth,
            viewport_width,
            viewport_height,
            tan_half_fov,
            aspect,
        );
        let weight = camera_plane_view_center_weight(
            screen_position,
            viewport_width,
            viewport_height,
        );
        if weight < 0.35 {
            continue;
        }

        center_cells.push((
            index,
            screen_position,
            required_camera_plane_half_extents_px(
                camera_position,
                profile,
                weight,
            ),
            weight,
        ));
    }

    center_cells.sort_by(|left, right| {
        right
            .3
            .partial_cmp(&left.3)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut settled: Vec<([f32; 2], [f32; 2])> = Vec::new();
    for (order, (index, mut screen_position, required_half_extents, _weight)) in
        center_cells.into_iter().enumerate()
    {
        if order == 0 {
            screen_position = [0.0, 0.0];
        }

        for attempt in 0..12 {
            let mut displacement = [0.0_f32; 2];
            let mut had_overlap = false;

            for (other_screen, other_required_half_extents) in settled.iter() {
                let dx = screen_position[0] - other_screen[0];
                let dy = screen_position[1] - other_screen[1];
                let overlap_x = required_half_extents[0]
                    + other_required_half_extents[0]
                    - dx.abs();
                let overlap_y = required_half_extents[1]
                    + other_required_half_extents[1]
                    - dy.abs();

                if overlap_x <= 0.0 || overlap_y <= 0.0 {
                    continue;
                }

                had_overlap = true;
                let mut direction = normalize_2d([
                    if dx.abs() > 0.5 {
                        dx
                    } else {
                        screen_position[0]
                    },
                    if dy.abs() > 0.5 {
                        dy
                    } else {
                        screen_position[1]
                    },
                ]);
                if direction[0].abs() < 0.001 && direction[1].abs() < 0.001 {
                    let angle =
                        attempt as f32 * 1.618_034 + index as f32 * 0.73;
                    direction = [angle.cos(), angle.sin()];
                }

                let push = overlap_x.max(overlap_y) + 10.0;
                displacement[0] += direction[0] * push;
                displacement[1] += direction[1] * push;
            }

            if !had_overlap {
                break;
            }

            let clamped = clamp_2d(displacement, CAMERA_PLANE_MAX_PUSH_PX);
            screen_position[0] += clamped[0];
            screen_position[1] += clamped[1];
        }

        if screen_overlaps_settled(
            screen_position,
            required_half_extents,
            settled.as_slice(),
        ) {
            let base_radius = length_2d(screen_position).max(24.0);
            'search: for radius_step in 1..=18 {
                let radius = base_radius + radius_step as f32 * 34.0;
                for angle_step in 0..16 {
                    let angle = index as f32 * 0.11
                        + angle_step as f32 * (std::f32::consts::TAU / 16.0);
                    let candidate =
                        [radius * angle.cos(), radius * angle.sin()];
                    if !screen_overlaps_settled(
                        candidate,
                        required_half_extents,
                        settled.as_slice(),
                    ) {
                        screen_position = candidate;
                        break 'search;
                    }
                }
            }
        }

        plane_positions[index][0] = screen_px_to_camera_plane_x(
            screen_position[0],
            depth,
            viewport_width,
            tan_half_fov,
            aspect,
        );
        plane_positions[index][1] = screen_px_to_camera_plane_y(
            screen_position[1],
            depth,
            viewport_height,
            tan_half_fov,
        );
        settled.push((screen_position, required_half_extents));
    }

    recenter_camera_plane_positions(plane_positions);
}

fn camera_plane_pair_axis_direction(
    delta: f32,
    screen_a: f32,
    screen_b: f32,
    index_a: usize,
    index_b: usize,
) -> f32 {
    if delta.abs() > 0.5 {
        return delta.signum();
    }

    let outward_delta = screen_a + screen_b;
    if outward_delta.abs() > 0.5 {
        outward_delta.signum()
    } else if ((index_a + index_b) & 1) == 0 {
        1.0
    } else {
        -1.0
    }
}

fn required_camera_plane_half_extents_px(
    camera_position: [f32; 3],
    profile: NodeCardProfile,
    center_weight: f32,
) -> [f32; 2] {
    let half_extents = projected_card_half_extents_px(camera_position, profile);
    let footprint_scale = lerp(
        CAMERA_PLANE_EDGE_FOOTPRINT_SCALE,
        CAMERA_PLANE_CENTER_FOOTPRINT_SCALE,
        center_weight,
    );

    [
        half_extents[0] * footprint_scale
            + lerp(
                CAMERA_PLANE_EDGE_BORDER_X_PX,
                CAMERA_PLANE_CENTER_BORDER_X_PX,
                center_weight,
            ),
        half_extents[1] * footprint_scale
            + lerp(
                CAMERA_PLANE_EDGE_BORDER_Y_PX,
                CAMERA_PLANE_CENTER_BORDER_Y_PX,
                center_weight,
            ),
    ]
}

fn camera_plane_view_center_weight(
    screen_position: [f32; 2],
    viewport_width: f32,
    viewport_height: f32,
) -> f32 {
    let radius = ((screen_position[0] / (viewport_width * 0.5).max(1.0))
        .powi(2)
        + (screen_position[1] / (viewport_height * 0.5).max(1.0)).powi(2))
    .sqrt();
    let weight = (1.0 - radius / CAMERA_PLANE_CENTER_RADIUS).clamp(0.0, 1.0);
    weight * weight
}

fn camera_plane_to_screen_px(
    plane_position: [f32; 2],
    depth: f32,
    viewport_width: f32,
    viewport_height: f32,
    tan_half_fov: f32,
    aspect: f32,
) -> [f32; 2] {
    [
        plane_position[0] * viewport_width * 0.5
            / (depth * tan_half_fov * aspect),
        plane_position[1] * viewport_height * 0.5 / (depth * tan_half_fov),
    ]
}

fn screen_px_to_camera_plane_x(
    pixels: f32,
    depth: f32,
    viewport_width: f32,
    tan_half_fov: f32,
    aspect: f32,
) -> f32 {
    (pixels * 2.0 / viewport_width) * depth * tan_half_fov * aspect
}

fn screen_px_to_camera_plane_y(
    pixels: f32,
    depth: f32,
    viewport_height: f32,
    tan_half_fov: f32,
) -> f32 {
    (pixels * 2.0 / viewport_height) * depth * tan_half_fov
}

fn projected_card_half_extents_px(
    camera_position: [f32; 3],
    profile: NodeCardProfile,
) -> [f32; 2] {
    let distance = length(camera_position).max(0.1);
    let pixel_scale = pixel_scale_for_distance(distance);
    let card_size = node_card_base_size_px(profile);
    [
        card_size[0] * pixel_scale * 0.5,
        card_size[1] * pixel_scale * 0.5,
    ]
}

fn node_card_base_size_px(profile: NodeCardProfile) -> [f32; 2] {
    match profile {
        NodeCardProfile::Compact => [
            CAMERA_PLANE_COMPACT_CARD_BASE_WIDTH_PX,
            CAMERA_PLANE_COMPACT_CARD_BASE_HEIGHT_PX,
        ],
        NodeCardProfile::TicketWide => [
            CAMERA_PLANE_TICKET_CARD_BASE_WIDTH_PX,
            CAMERA_PLANE_TICKET_CARD_BASE_HEIGHT_PX,
        ],
    }
}

fn pixel_scale_for_distance(distance: f32) -> f32 {
    (22.0 / distance).clamp(0.14, 3.5)
}

fn lerp(
    start: f32,
    end: f32,
    t: f32,
) -> f32 {
    start + (end - start) * t.clamp(0.0, 1.0)
}

fn normalize_2d(vector: [f32; 2]) -> [f32; 2] {
    let len = (vector[0] * vector[0] + vector[1] * vector[1]).sqrt();
    if len < 0.0001 {
        [0.0, 0.0]
    } else {
        [vector[0] / len, vector[1] / len]
    }
}

fn clamp_2d(
    vector: [f32; 2],
    max_len: f32,
) -> [f32; 2] {
    let len = (vector[0] * vector[0] + vector[1] * vector[1]).sqrt();
    if len <= max_len || len < 0.0001 {
        vector
    } else {
        let scale = max_len / len;
        [vector[0] * scale, vector[1] * scale]
    }
}

fn length_2d(vector: [f32; 2]) -> f32 {
    (vector[0] * vector[0] + vector[1] * vector[1]).sqrt()
}

fn screen_overlaps_settled(
    screen_position: [f32; 2],
    required_half_extents: [f32; 2],
    settled: &[([f32; 2], [f32; 2])],
) -> bool {
    settled
        .iter()
        .any(|(other_screen, other_required_half_extents)| {
            let dx = (screen_position[0] - other_screen[0]).abs();
            let dy = (screen_position[1] - other_screen[1]).abs();
            dx < required_half_extents[0] + other_required_half_extents[0]
                && dy
                    < required_half_extents[1] + other_required_half_extents[1]
        })
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

#[derive(Clone, Copy)]
struct CameraBasis {
    eye: [f32; 3],
    forward: [f32; 3],
    right: [f32; 3],
    up: [f32; 3],
}

impl CameraBasis {
    fn from_camera(camera: &Camera) -> Self {
        let eye = camera.eye();
        let forward = normalize([
            camera.target[0] - eye[0],
            camera.target[1] - eye[1],
            camera.target[2] - eye[2],
        ]);
        let mut right = cross(forward, [0.0, 1.0, 0.0]);
        if length(right) < 0.001 {
            right = [1.0, 0.0, 0.0];
        } else {
            right = normalize(right);
        }
        let up = normalize(cross(right, forward));
        Self {
            eye,
            forward,
            right,
            up,
        }
    }
}

fn world_to_camera_space(
    world: [f32; 3],
    basis: &CameraBasis,
) -> [f32; 3] {
    let delta = [
        world[0] - basis.eye[0],
        world[1] - basis.eye[1],
        world[2] - basis.eye[2],
    ];
    [
        dot(delta, basis.right),
        dot(delta, basis.up),
        dot(delta, basis.forward),
    ]
}

fn camera_to_world_space(
    camera_space: [f32; 3],
    basis: &CameraBasis,
) -> [f32; 3] {
    [
        basis.eye[0]
            + basis.right[0] * camera_space[0]
            + basis.up[0] * camera_space[1]
            + basis.forward[0] * camera_space[2],
        basis.eye[1]
            + basis.right[1] * camera_space[0]
            + basis.up[1] * camera_space[1]
            + basis.forward[1] * camera_space[2],
        basis.eye[2]
            + basis.right[2] * camera_space[0]
            + basis.up[2] * camera_space[1]
            + basis.forward[2] * camera_space[2],
    ]
}

fn dot(
    a: [f32; 3],
    b: [f32; 3],
) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(
    a: [f32; 3],
    b: [f32; 3],
) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(vector: [f32; 3]) -> f32 {
    dot(vector, vector).sqrt()
}

fn normalize(vector: [f32; 3]) -> [f32; 3] {
    let len = length(vector);
    if len < 0.0001 {
        [0.0, 0.0, 0.0]
    } else {
        [vector[0] / len, vector[1] / len, vector[2] / len]
    }
}

#[cfg(test)]
mod tests {
    use super::{
        animate_layout_nodes,
        apply_node_view_transform,
        apply_selected_node_auto_layout,
        camera_plane_to_screen_px,
        camera_plane_view_center_weight,
        length,
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
            .map(|node| {
                world_to_camera_space([node.x, node.y, node.z], &basis)[2]
            })
            .collect();
        let min_depth = depths.iter().copied().fold(f32::INFINITY, f32::min);
        let max_depth =
            depths.iter().copied().fold(f32::NEG_INFINITY, f32::max);

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
        let transform =
            NodeViewTransform::camera_plane_view_direction(0.82, 1.0);
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
}
