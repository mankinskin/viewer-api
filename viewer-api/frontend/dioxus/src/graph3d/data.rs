//! Data types for the shared 3-D graph view.
//!
//! Domain-agnostic: a node is just a positioned id with optional label/state,
//! and an edge is a typed reference between two node indices.

/// One node in 3-D world space.
#[derive(Debug, Clone, PartialEq)]
pub struct Node3D {
    pub id:    String,
    pub label: Option<String>,
    pub state: Option<String>,
    pub x:     f32,
    pub y:     f32,
    pub z:     f32,
}

/// One edge between two nodes (referenced by index into `Layout3D::nodes`).
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeRef3D {
    pub from_idx: usize,
    pub to_idx:   usize,
    pub kind:     String,
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

/// Half-extent of the coordinate grid (world units).
const GRID_HALF: f32 = 30.0;
/// Number of grid lines we generate (used for buffer pre-sizing).
const GRID_LINE_COUNT: usize = ((GRID_HALF as i32) * 2 + 1) as usize * 2;

fn grid_line_color(coord: f32) -> (f32, f32, f32, f32) {
    // Axis (coord == 0): brighter cool colour.
    if coord.abs() < 0.001 {
        return (0.45, 0.55, 0.75, 0.55);
    }
    // Major gridline every 5 units: mid alpha.
    if (coord.rem_euclid(5.0)).abs() < 0.01
        || (coord.rem_euclid(5.0) - 5.0).abs() < 0.01
    {
        return (0.35, 0.40, 0.55, 0.35);
    }
    // Minor: dim.
    (0.30, 0.32, 0.42, 0.18)
}

impl Layout3D {
    pub fn new(nodes: Vec<Node3D>, edges: Vec<EdgeRef3D>) -> Self {
        Self { nodes, edges, node_card_profile: NodeCardProfile::default() }
    }

    pub fn with_node_card_profile(mut self, node_card_profile: NodeCardProfile) -> Self {
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
        let mut data = Vec::with_capacity(
            (self.edges.len() + GRID_LINE_COUNT) * EDGE_INST_FLOATS,
        );
        let mut count = 0u32;
        let graph_edge_type = match self.node_card_profile {
            NodeCardProfile::Compact => 1.0,
            NodeCardProfile::TicketWide => 2.0,
        };

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
                -half, 0.0, z,
                 half, 0.0, z,
                 r,    g,   b, a,
                 0.0,        // flags
                 0.0,        // edgeType = 0 (simple thin AA line)
            ]);
            count += 1;
            z += step;
        }
        let mut x = -half;
        while x <= half + 0.0001 {
            let (r, g, b, a) = grid_line_color(x);
            data.extend_from_slice(&[
                x, 0.0, -half,
                x, 0.0,  half,
                r, g,   b, a,
                0.0,
                0.0,
            ]);
            count += 1;
            x += step;
        }

        // ── Graph edges ──
        for edge in &self.edges {
            let Some(a) = self.nodes.get(edge.from_idx) else { continue };
            let Some(b) = self.nodes.get(edge.to_idx)   else { continue };
            let (r, g, bl, alpha) = edge_color(&edge.kind);
            data.extend_from_slice(&[
                a.x, a.y, a.z,
                b.x, b.y, b.z,
                r,   g,   bl, alpha,
                0.0,        // flags
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

fn edge_color(kind: &str) -> (f32, f32, f32, f32) {
    match kind {
        "depends_on" | "dep" | "code_ref" => (0.15, 0.75, 0.90, 0.70),
        "blocks"                          => (0.90, 0.40, 0.20, 0.70),
        "parent" | "child" | "section"    => (0.55, 0.45, 0.85, 0.60),
        _                                 => (0.50, 0.50, 0.70, 0.50),
    }
}

#[cfg(test)]
mod tests {
    use super::{EdgeRef3D, Layout3D, Node3D, NodeCardProfile, EDGE_INST_FLOATS, GRID_LINE_COUNT};

    fn sample_nodes() -> Vec<Node3D> {
        vec![
            Node3D { id: "root".into(), label: None, state: None, x: 0.0, y: 0.0, z: 0.0 },
            Node3D { id: "child".into(), label: None, state: None, x: 2.0, y: 0.0, z: 0.0 },
        ]
    }

    fn sample_edges() -> Vec<EdgeRef3D> {
        vec![EdgeRef3D { from_idx: 0, to_idx: 1, kind: "depends_on".into() }]
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
}
