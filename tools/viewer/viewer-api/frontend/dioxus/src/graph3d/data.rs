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

/// A complete 3-D graph: positioned nodes + indexed edges.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout3D {
    pub nodes: Vec<Node3D>,
    pub edges: Vec<EdgeRef3D>,
}

/// Per-edge instance floats (posA[3]+posB[3]+color[4]+flags[1]+edgeType[1]).
pub(crate) const EDGE_INST_FLOATS: usize = 12;

impl Layout3D {
    pub fn new(nodes: Vec<Node3D>, edges: Vec<EdgeRef3D>) -> Self {
        Self { nodes, edges }
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
    pub(crate) fn build_edge_instances(&self) -> (Vec<f32>, u32) {
        let mut data = Vec::with_capacity(self.edges.len() * EDGE_INST_FLOATS);
        let mut count = 0u32;
        for edge in &self.edges {
            let Some(a) = self.nodes.get(edge.from_idx) else { continue };
            let Some(b) = self.nodes.get(edge.to_idx)   else { continue };
            let (r, g, bl, alpha) = edge_color(&edge.kind);
            data.extend_from_slice(&[
                a.x, a.y, a.z,
                b.x, b.y, b.z,
                r,   g,   bl, alpha,
                0.0,        // flags
                1.0,        // edgeType = normal energy beam
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
