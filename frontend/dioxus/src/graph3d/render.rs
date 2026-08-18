//! Per-frame render: GPU pass + CSS-3D DOM node positioning.

#![cfg(target_arch = "wasm32")]

use js_sys::{
    Array,
    Function,
    Reflect,
};
use wasm_bindgen::{
    prelude::*,
    JsCast,
};
use web_sys::{
    Element,
    HtmlElement,
};

use super::{
    camera::{
        animate_camera,
        Camera,
        CameraMode,
        Projection,
        CAMERA_FAR,
        CAMERA_FOV,
        CAMERA_NEAR,
        CAM_UNIFORM_FLOATS,
    },
    data::{
        anchor_zoom_scale_for_distance,
        animate_layout_nodes,
        apply_node_view_transform,
        edge_color,
        node_detail_dimensions_px,
        node_detail_tier,
        EdgeRef3D,
        EdgeVisualState,
        Layout3D,
        NodeCardProfile,
        NodeDetailTier,
        NodeViewTransform,
        EDGE_FLAG_SELECTED,
    },
    gpu::GpuResources,
    interop::*,
    math,
    theme::GraphThemeSettings,
};

const CAMERA_ANIMATION_SPEED: f32 = 6.0;
const LAYOUT_ANIMATION_SPEED: f32 = 12.0;

pub(crate) struct RenderState {
    pub gpu: GpuResources,
    pub base_layout: Layout3D,
    pub layout: Layout3D,
    pub target_layout: Layout3D,
    pub camera: Camera,
    pub camera_goal: Option<Camera>,
    pub camera_mode: CameraMode,
    pub edge_buf: web_sys::GpuBuffer,
    pub edge_count: u32,
    pub node_quad_buf: web_sys::GpuBuffer,
    pub node_count: u32,
    /// CSS id of the DOM container that hosts the node cards. Used to
    /// translate world-space projections into container-local pixels.
    pub container_id: String,
    /// Set by the drag interaction when a node has moved this frame; the
    /// renderer will rewrite `edge_buf` + `node_quad_buf` from `layout`
    /// before drawing.
    pub dirty_layout: bool,
    /// Set when focus-only state changes and only edge instances need a rewrite.
    pub dirty_edges: bool,
    /// Camera projection mode (perspective or orthographic).
    pub projection: Projection,
    pub graph_theme: GraphThemeSettings,
    pub viewport_insets: [f32; 4],
    pub selection_auto_layout: bool,
    pub node_view_transform: NodeViewTransform,
    pub selected_node_id: Option<String>,
    pub hovered_node_id: Option<String>,
    pub last_frame_time: Option<f32>,
    pub interaction_active: bool,
}

struct ScreenPos {
    x: f32,
    y: f32,
    z: f32,
    visible: bool,
}

#[derive(Clone, Copy)]
struct NodeScreenRect {
    center_x: f32,
    center_y: f32,
    half_w: f32,
    half_h: f32,
}

impl NodeScreenRect {
    fn left(self) -> f32 {
        self.center_x - self.half_w
    }
}

fn rects_overlap(
    a: NodeScreenRect,
    b: NodeScreenRect,
    pad_x: f32,
    pad_y: f32,
) -> bool {
    (a.center_x - b.center_x).abs() < (a.half_w + b.half_w + pad_x)
        && (a.center_y - b.center_y).abs() < (a.half_h + b.half_h + pad_y)
}

fn right_center_anchor_rect(
    right_x: f32,
    center_y: f32,
    width: f32,
    height: f32,
) -> NodeScreenRect {
    NodeScreenRect {
        center_x: right_x - width * 0.5,
        center_y,
        half_w: width * 0.5,
        half_h: height * 0.5,
    }
}

fn resolve_right_center_anchor_position(
    html_el: &HtmlElement,
    screen_x: f32,
    screen_y: f32,
    scale: f32,
    viewport_w: f32,
    viewport_h: f32,
    node_rects: &[Option<NodeScreenRect>],
) -> (f32, f32) {
    let width = (html_el.offset_width().max(1) as f32) * scale;
    let height = (html_el.offset_height().max(1) as f32) * scale;
    let min_visible_right = (24.0 + (width * 0.16).min(28.0)).max(42.0);
    let max_right = (viewport_w - 16.0).max(min_visible_right);
    let center_y = screen_y.clamp(
        18.0 + height * 0.5,
        (viewport_h - 18.0 - height * 0.5).max(18.0 + height * 0.5),
    );
    let gap_x = 18.0 + width * 0.05;
    let gap_y = 10.0 + height * 0.05;
    let mut right_x = screen_x.min(max_right);

    for _ in 0..2 {
        let mut shifted = false;
        let probe = right_center_anchor_rect(right_x, center_y, width, height);
        for node_rect in node_rects.iter().flatten() {
            if rects_overlap(probe, *node_rect, gap_x, gap_y) {
                let candidate = right_x.min(node_rect.left() - gap_x);
                if candidate < right_x - 0.1 {
                    right_x = candidate;
                    shifted = true;
                }
            }
        }
        if !shifted {
            break;
        }
    }

    (right_x.max(min_visible_right), center_y)
}

fn resolve_viewport_rect(
    container_w: f32,
    container_h: f32,
    viewport_insets: [f32; 4],
) -> (f32, f32, f32, f32) {
    let left = viewport_insets[0].max(0.0);
    let top = viewport_insets[1].max(0.0);
    let right = viewport_insets[2].max(0.0);
    let bottom = viewport_insets[3].max(0.0);
    let width = (container_w - left - right).max(1.0);
    let height = (container_h - top - bottom).max(1.0);

    let clamped_left = left.min((container_w - width).max(0.0));
    let clamped_top = top.min((container_h - height).max(0.0));
    (clamped_left, clamped_top, width, height)
}

fn world_to_screen(
    pos: [f32; 3],
    vp: &[f32; 16],
    vw: f32,
    vh: f32,
) -> ScreenPos {
    let x = vp[0] * pos[0] + vp[4] * pos[1] + vp[8] * pos[2] + vp[12];
    let y = vp[1] * pos[0] + vp[5] * pos[1] + vp[9] * pos[2] + vp[13];
    let z = vp[2] * pos[0] + vp[6] * pos[1] + vp[10] * pos[2] + vp[14];
    let w = vp[3] * pos[0] + vp[7] * pos[1] + vp[11] * pos[2] + vp[15];
    if w <= 0.001 {
        return ScreenPos {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            visible: false,
        };
    }
    let ndc_x = x / w;
    let ndc_y = y / w;
    let ndc_z = z / w;
    let sx = (ndc_x + 1.0) * 0.5 * vw;
    let sy = (1.0 - ndc_y) * 0.5 * vh;
    ScreenPos {
        x: sx,
        y: sy,
        z: ndc_z,
        visible: ndc_z >= 0.0 && ndc_z <= 1.0,
    }
}

/// Compute per-node screen-space footprints analytically from the projected
/// layout, mirroring the projection + LOD math used by `position_dom_nodes`
/// and `position_dom_edges`. This intentionally performs **no** DOM reads
/// (`getBoundingClientRect`/`offsetWidth`) so it never forces a synchronous
/// reflow after the per-frame transform writes — the root cause of node cards
/// trailing the GPU-drawn edges during orbit/pan/drag.
fn compute_node_screen_rects(
    state: &RenderState,
    layout: &Layout3D,
    vp: &[f32; 16],
    viewport_x: f32,
    viewport_y: f32,
    viewport_w: f32,
    viewport_h: f32,
) -> Vec<Option<NodeScreenRect>> {
    let mut rects = vec![None; layout.nodes.len()];
    let eye = state.camera.eye();
    let margin = 300.0;

    for (idx, node) in layout.nodes.iter().enumerate() {
        let screen = world_to_screen(
            [node.x, node.y, node.z],
            vp,
            viewport_w,
            viewport_h,
        );

        let dx = eye[0] - node.x;
        let dy = eye[1] - node.y;
        let dz = eye[2] - node.z;
        let dist = (dx * dx + dy * dy + dz * dz).sqrt().max(0.1);
        let pixel_scale = (22.0 / dist).clamp(0.14, 3.5);

        // Skip nodes that `position_dom_nodes` would have set to
        // `display: none` this frame so collision avoidance ignores them.
        if !screen.visible
            || screen.x < -margin
            || screen.x > viewport_w + margin
            || screen.y < -margin
            || screen.y > viewport_h + margin
            || pixel_scale < 0.08
        {
            continue;
        }

        let is_focus = state.selected_node_id.as_deref()
            == Some(node.id.as_str())
            || state.hovered_node_id.as_deref() == Some(node.id.as_str());
        let is_hover =
            state.hovered_node_id.as_deref() == Some(node.id.as_str());
        let detail_tier = node_detail_tier(
            pixel_scale,
            is_focus,
            is_hover,
            &state.graph_theme,
        );
        let [card_w, card_h] =
            node_detail_dimensions_px(detail_tier, layout.node_card_profile);

        rects[idx] = Some(NodeScreenRect {
            center_x: viewport_x + screen.x,
            center_y: viewport_y + screen.y,
            half_w: card_w * pixel_scale * 0.5,
            half_h: card_h * pixel_scale * 0.5,
        });
    }

    rects
}

fn clip_edge_endpoint(
    rect: NodeScreenRect,
    toward: (f32, f32),
    padding: f32,
) -> (f32, f32) {
    let dx = toward.0 - rect.center_x;
    let dy = toward.1 - rect.center_y;
    if dx.abs() < 0.001 && dy.abs() < 0.001 {
        return (rect.center_x, rect.center_y);
    }

    let tx = if dx.abs() > 0.001 {
        (rect.half_w + padding) / dx.abs()
    } else {
        f32::INFINITY
    };
    let ty = if dy.abs() > 0.001 {
        (rect.half_h + padding) / dy.abs()
    } else {
        f32::INFINITY
    };
    let t = tx.min(ty).max(0.0);

    (rect.center_x + dx * t, rect.center_y + dy * t)
}

fn parse_attr_f32(
    html_el: &HtmlElement,
    name: &str,
) -> Option<f32> {
    let raw = html_el.get_attribute(name)?;
    raw.parse::<f32>().ok()
}

fn anchor_transform(
    local_x: f32,
    local_y: f32,
    origin: &str,
    scale: f32,
) -> String {
    let translate = match origin {
        "center-bottom" => format!(
            "translate(-50%, -100%) translate({:.1}px, {:.1}px)",
            local_x, local_y,
        ),
        "right-center" => format!(
            "translate(-100%, -50%) translate({:.1}px, {:.1}px)",
            local_x, local_y,
        ),
        "left-center" => format!(
            "translate(0%, -50%) translate({:.1}px, {:.1}px)",
            local_x, local_y,
        ),
        _ => format!(
            "translate(-50%, -50%) translate({:.1}px, {:.1}px)",
            local_x, local_y,
        ),
    };

    if (scale - 1.0).abs() < 0.001 {
        translate
    } else {
        format!("{translate} scale({scale:.3})")
    }
}

fn anchor_zoom_scale(
    state: &RenderState,
    origin: &str,
    anchor: [f32; 3],
) -> f32 {
    let distance = match state.projection {
        // Orthographic overlay size should follow camera zoom, not shrink
        // just because an anchor sits farther from the target in world space.
        Projection::Orthographic => state.camera.distance.max(0.1),
        Projection::Perspective => {
            let eye = state.camera.eye();
            let forward = state.camera.forward();
            let to_anchor =
                [anchor[0] - eye[0], anchor[1] - eye[1], anchor[2] - eye[2]];
            (to_anchor[0] * forward[0]
                + to_anchor[1] * forward[1]
                + to_anchor[2] * forward[2])
                .max(0.1)
        },
    };

    anchor_zoom_scale_for_distance(origin, distance, &state.graph_theme)
}

fn position_dom_layout_anchors(
    doc: &web_sys::Document,
    state: &RenderState,
    _layout: &Layout3D,
    vp: &[f32; 16],
    node_rects: &[Option<NodeScreenRect>],
    viewport_x: f32,
    viewport_y: f32,
    viewport_w: f32,
    viewport_h: f32,
) {
    let margin = 320.0;

    if let Ok(anchor_nodes) = doc.query_selector_all(&format!(
        "#{} [data-layout-anchor-x]",
        state.container_id
    )) {
        for index in 0..anchor_nodes.length() {
            let Some(node) = anchor_nodes.item(index) else {
                continue;
            };
            let Ok(html_el) = node.dyn_into::<HtmlElement>() else {
                continue;
            };
            let (Some(x), Some(y), Some(z)) = (
                parse_attr_f32(&html_el, "data-layout-anchor-x"),
                parse_attr_f32(&html_el, "data-layout-anchor-y"),
                parse_attr_f32(&html_el, "data-layout-anchor-z"),
            ) else {
                continue;
            };

            let screen =
                world_to_screen([x, y, z], &vp, viewport_w, viewport_h);
            let origin = html_el
                .get_attribute("data-layout-anchor-origin")
                .unwrap_or_else(|| "center".to_string());
            let behind_camera =
                !screen.visible && screen.x == 0.0 && screen.y == 0.0;

            let (screen_x, screen_y, hide) = match origin.as_str() {
                // Keep column headers visible at the top edge even when the
                // projected world anchor drifts slightly above the viewport.
                "center-bottom" => {
                    let hide = behind_camera
                        || screen.x < -(margin * 2.0)
                        || screen.x > viewport_w + (margin * 2.0);
                    (
                        screen.x.clamp(28.0, (viewport_w - 28.0).max(28.0)),
                        screen.y.clamp(56.0, (viewport_h - 20.0).max(56.0)),
                        hide,
                    )
                },
                // Keep row labels readable near the left edge while still
                // allowing extra clearance from enlarged visible node cards.
                "right-center" => {
                    let hide = behind_camera
                        || screen.y < -margin
                        || screen.y > viewport_h + margin;
                    (
                        screen.x,
                        screen.y.clamp(18.0, (viewport_h - 18.0).max(18.0)),
                        hide,
                    )
                },
                _ => (
                    screen.x,
                    screen.y,
                    behind_camera
                        || screen.x < -margin
                        || screen.x > viewport_w + margin
                        || screen.y < -margin
                        || screen.y > viewport_h + margin,
                ),
            };
            if hide {
                let _ = html_el.style().set_property("display", "none");
                continue;
            }

            let _ = html_el.style().set_property("display", "block");
            let scale = anchor_zoom_scale(state, &origin, [x, y, z]);
            let (screen_x, screen_y) = match origin.as_str() {
                "right-center" => resolve_right_center_anchor_position(
                    &html_el,
                    screen_x,
                    screen_y,
                    scale,
                    viewport_w,
                    viewport_h,
                    &node_rects,
                ),
                _ => (screen_x, screen_y),
            };
            let local_x = viewport_x + screen_x;
            let local_y = viewport_y + screen_y;
            let transform = anchor_transform(local_x, local_y, &origin, scale);

            let _ = html_el.style().set_property(
                "z-index",
                &html_el
                    .get_attribute("data-layout-z-index")
                    .unwrap_or_else(|| "10".to_string()),
            );
            let _ = html_el.style().set_property("transform", &transform);
        }
    }

    if let Ok(line_nodes) = doc.query_selector_all(&format!(
        "#{} [data-layout-line-x1]",
        state.container_id
    )) {
        for index in 0..line_nodes.length() {
            let Some(node) = line_nodes.item(index) else {
                continue;
            };
            let Ok(html_el) = node.dyn_into::<HtmlElement>() else {
                continue;
            };
            let (Some(x1), Some(y1), Some(z1), Some(x2), Some(y2), Some(z2)) = (
                parse_attr_f32(&html_el, "data-layout-line-x1"),
                parse_attr_f32(&html_el, "data-layout-line-y1"),
                parse_attr_f32(&html_el, "data-layout-line-z1"),
                parse_attr_f32(&html_el, "data-layout-line-x2"),
                parse_attr_f32(&html_el, "data-layout-line-y2"),
                parse_attr_f32(&html_el, "data-layout-line-z2"),
            ) else {
                continue;
            };

            let start =
                world_to_screen([x1, y1, z1], &vp, viewport_w, viewport_h);
            let end =
                world_to_screen([x2, y2, z2], &vp, viewport_w, viewport_h);
            if (!start.visible && !end.visible)
                || ((start.x - end.x).abs() < 1.0
                    && (start.y - end.y).abs() < 1.0)
            {
                let _ = html_el.style().set_property("display", "none");
                continue;
            }

            let local_x1 = viewport_x + start.x;
            let local_y1 = viewport_y + start.y;
            let local_x2 = viewport_x + end.x;
            let local_y2 = viewport_y + end.y;
            let dx = local_x2 - local_x1;
            let dy = local_y2 - local_y1;
            let length = (dx * dx + dy * dy).sqrt().max(1.0);
            let angle_deg = dy.atan2(dx).to_degrees();

            let _ = html_el.style().set_property("display", "block");
            let _ = html_el.style().set_property(
                "z-index",
                &html_el
                    .get_attribute("data-layout-z-index")
                    .unwrap_or_else(|| "8".to_string()),
            );
            let _ = html_el
                .style()
                .set_property("width", &format!("{length:.1}px"));
            let _ = html_el.style().set_property(
                "transform",
                &format!(
                    "translate({:.1}px, {:.1}px) rotate({:.3}deg)",
                    local_x1, local_y1, angle_deg,
                ),
            );
        }
    }
}

fn position_dom_nodes(
    doc: &web_sys::Document,
    state: &RenderState,
    layout: &Layout3D,
    vp: &[f32; 16],
    viewport_x: f32,
    viewport_y: f32,
    viewport_w: f32,
    viewport_h: f32,
) {
    let eye = state.camera.eye();

    let Ok(node_list) = doc.query_selector_all(&format!(
        "#{} [data-node-idx]",
        state.container_id
    )) else {
        return;
    };
    for i in 0..node_list.length() {
        let Some(el) = node_list.item(i) else {
            continue;
        };
        let Ok(html_el) = el.dyn_into::<HtmlElement>() else {
            continue;
        };
        let Some(idx_str) = html_el.get_attribute("data-node-idx") else {
            continue;
        };
        let Ok(idx) = idx_str.parse::<usize>() else {
            continue;
        };
        let Some(node) = layout.nodes.get(idx) else {
            continue;
        };

        let screen = world_to_screen(
            [node.x, node.y, node.z],
            &vp,
            viewport_w,
            viewport_h,
        );

        let dx = eye[0] - node.x;
        let dy = eye[1] - node.y;
        let dz = eye[2] - node.z;
        let dist = (dx * dx + dy * dy + dz * dz).sqrt().max(0.1);
        let pixel_scale = (22.0 / dist).clamp(0.14, 3.5);
        let is_focus = state.selected_node_id.as_deref()
            == Some(node.id.as_str())
            || state.hovered_node_id.as_deref() == Some(node.id.as_str());
        let is_hover =
            state.hovered_node_id.as_deref() == Some(node.id.as_str());
        let detail_tier = node_detail_tier(
            pixel_scale,
            is_focus,
            is_hover,
            &state.graph_theme,
        );
        let [card_w, card_h] =
            node_detail_dimensions_px(detail_tier, layout.node_card_profile);

        let margin = 300.0;
        if !screen.visible
            || screen.x < -margin
            || screen.x > viewport_w + margin
            || screen.y < -margin
            || screen.y > viewport_h + margin
            || pixel_scale < 0.08
        {
            let _ = html_el.style().set_property("display", "none");
            continue;
        }

        // screen.x / screen.y are already container-local (no origin offset
        // needed) because the projection uses container dimensions.
        let local_x = viewport_x + screen.x;
        let local_y = viewport_y + screen.y;

        // Use explicit "block" instead of "" — when callers style cards via
        // CSS classes (e.g. `.content { display: none }`), removing the
        // inline override falls back to the CSS rule and the card stays
        // hidden. "block" wins as an inline override regardless.
        let _ = html_el.style().set_property("display", "block");
        // Selected nodes (class "node-card-selected") get a very high z-index
        // so they always render in the foreground over overlapping neighbours.
        let is_selected = html_el
            .get_attribute("class")
            .map(|c| c.contains("node-card-selected"))
            .unwrap_or(false);
        let z_idx = if is_selected {
            100_000i32
        } else {
            ((1.0 - screen.z) * 10000.0) as i32
        };
        let _ = html_el.style().set_property("z-index", &z_idx.to_string());
        let _ = html_el
            .style()
            .set_property("width", &format!("{card_w:.1}px"));
        let _ = html_el
            .style()
            .set_property("height", &format!("{card_h:.1}px"));
        sync_node_detail_tier(&html_el, detail_tier);

        let transform = format!(
            "translate(-50%, -50%) translate({:.1}px, {:.1}px) scale({:.3})",
            local_x, local_y, pixel_scale,
        );
        let _ = html_el.style().set_property("transform", &transform);
    }
}

fn sync_node_detail_tier(
    html_el: &HtmlElement,
    tier: NodeDetailTier,
) {
    let tier_name = tier.as_str();
    // Skip the nested query + per-child display writes when the card is
    // already on this LOD tier. During orbit/pan the tier is stable for most
    // frames, so this avoids N*M redundant DOM writes per frame.
    if html_el.get_attribute("data-node-lod").as_deref() == Some(tier_name) {
        return;
    }
    let _ = html_el.set_attribute("data-node-lod", tier_name);
    let Ok(detail_nodes) =
        html_el.query_selector_all("[data-node-detail-tier]")
    else {
        return;
    };

    for index in 0..detail_nodes.length() {
        let Some(node) = detail_nodes.item(index) else {
            continue;
        };
        let Ok(detail_el) = node.dyn_into::<HtmlElement>() else {
            continue;
        };
        let matches_tier =
            detail_el.get_attribute("data-node-detail-tier").as_deref()
                == Some(tier_name);
        let display = if matches_tier {
            detail_el
                .get_attribute("data-node-detail-display")
                .unwrap_or_else(|| "block".to_string())
        } else {
            "none".to_string()
        };
        let _ = detail_el.style().set_property("display", &display);
    }
}

fn position_dom_edges(
    doc: &web_sys::Document,
    state: &RenderState,
    layout: &Layout3D,
    vp: &[f32; 16],
    viewport_x: f32,
    viewport_y: f32,
    viewport_w: f32,
    viewport_h: f32,
) {
    let Some(container) = doc.get_element_by_id(&state.container_id) else {
        return;
    };
    let Ok(container) = container.dyn_into::<HtmlElement>() else {
        return;
    };
    let Some(svg) = doc
        .query_selector(&format!(
            "#{} [data-graph-edge-overlay='true']",
            state.container_id
        ))
        .ok()
        .flatten()
    else {
        return;
    };

    let container_rect = container.get_bounding_client_rect();
    let container_w = (container_rect.width() as f32).max(1.0);
    let container_h = (container_rect.height() as f32).max(1.0);

    let _ = svg.set_attribute("display", "block");
    let _ = svg.set_attribute("width", &format!("{container_w:.1}"));
    let _ = svg.set_attribute("height", &format!("{container_h:.1}"));
    let _ = svg.set_attribute(
        "viewBox",
        &format!("0 0 {container_w:.1} {container_h:.1}"),
    );

    let eye = state.camera.eye();
    let active_focus = edge_visual_state(state).active_focus(&layout.nodes);

    let Ok(edge_nodes) = doc.query_selector_all(&format!(
        "#{} [data-edge-idx]",
        state.container_id
    )) else {
        return;
    };

    for index in 0..edge_nodes.length() {
        let Some(line_node) = edge_nodes.item(index) else {
            continue;
        };
        let Ok(line) = line_node.dyn_into::<Element>() else {
            continue;
        };
        let Some(edge) = layout.edges.get(index as usize) else {
            let _ = line.set_attribute("display", "none");
            continue;
        };
        let Some(a) = layout.nodes.get(edge.from_idx) else {
            let _ = line.set_attribute("display", "none");
            continue;
        };
        let Some(b) = layout.nodes.get(edge.to_idx) else {
            let _ = line.set_attribute("display", "none");
            continue;
        };

        let screen_a =
            world_to_screen([a.x, a.y, a.z], vp, viewport_w, viewport_h);
        let screen_b =
            world_to_screen([b.x, b.y, b.z], vp, viewport_w, viewport_h);
        if (!screen_a.visible && !screen_b.visible)
            || (screen_a.x - screen_b.x).abs() < 1.0
                && (screen_a.y - screen_b.y).abs() < 1.0
        {
            let _ = line.set_attribute("display", "none");
            continue;
        }

        // Calculate card detail dimensions for node A in order to build NodeScreenRect analytically
        let dx_a = eye[0] - a.x;
        let dy_a = eye[1] - a.y;
        let dz_a = eye[2] - a.z;
        let dist_a = (dx_a * dx_a + dy_a * dy_a + dz_a * dz_a).sqrt().max(0.1);
        let pixel_scale_a = (22.0 / dist_a).clamp(0.14, 3.5);
        let is_focus_a = state.selected_node_id.as_deref()
            == Some(a.id.as_str())
            || state.hovered_node_id.as_deref() == Some(a.id.as_str());
        let is_hover_a =
            state.hovered_node_id.as_deref() == Some(a.id.as_str());
        let detail_tier_a = node_detail_tier(
            pixel_scale_a,
            is_focus_a,
            is_hover_a,
            &state.graph_theme,
        );
        let [card_w_a, card_h_a] =
            node_detail_dimensions_px(detail_tier_a, layout.node_card_profile);

        let rect_a = NodeScreenRect {
            center_x: viewport_x + screen_a.x,
            center_y: viewport_y + screen_a.y,
            half_w: card_w_a * pixel_scale_a * 0.5,
            half_h: card_h_a * pixel_scale_a * 0.5,
        };

        // Calculate card detail dimensions for node B
        let dx_b = eye[0] - b.x;
        let dy_b = eye[1] - b.y;
        let dz_b = eye[2] - b.z;
        let dist_b = (dx_b * dx_b + dy_b * dy_b + dz_b * dz_b).sqrt().max(0.1);
        let pixel_scale_b = (22.0 / dist_b).clamp(0.14, 3.5);
        let is_focus_b = state.selected_node_id.as_deref()
            == Some(b.id.as_str())
            || state.hovered_node_id.as_deref() == Some(b.id.as_str());
        let is_hover_b =
            state.hovered_node_id.as_deref() == Some(b.id.as_str());
        let detail_tier_b = node_detail_tier(
            pixel_scale_b,
            is_focus_b,
            is_hover_b,
            &state.graph_theme,
        );
        let [card_w_b, card_h_b] =
            node_detail_dimensions_px(detail_tier_b, layout.node_card_profile);

        let rect_b = NodeScreenRect {
            center_x: viewport_x + screen_b.x,
            center_y: viewport_y + screen_b.y,
            half_w: card_w_b * pixel_scale_b * 0.5,
            half_h: card_h_b * pixel_scale_b * 0.5,
        };

        let center_a = (rect_a.center_x, rect_a.center_y);
        let center_b = (rect_b.center_x, rect_b.center_y);
        let (x1, y1) = clip_edge_endpoint(rect_a, center_b, 0.0);
        let (x2, y2) = clip_edge_endpoint(rect_b, center_a, 0.0);

        let (stroke_width, stroke_opacity) = edge_overlay_style(
            layout,
            active_focus,
            edge,
            a.id.as_str(),
            b.id.as_str(),
        );
        let Some(path_d) = edge_overlay_path(x1, y1, x2, y2, stroke_width)
        else {
            let _ = line.set_attribute("display", "none");
            continue;
        };
        let (r, g, blue, _alpha) = edge_color(&edge.kind, &state.graph_theme);
        let stroke_alpha = (stroke_opacity
            * state.graph_theme.edge_overlay_opacity)
            .clamp(0.0, 1.0);
        let _ = line.set_attribute("display", "block");
        let _ = line.set_attribute("d", &path_d);
        let _ = line.set_attribute(
            "fill",
            &format!(
                "rgb({:.0} {:.0} {:.0} / {:.3})",
                r * 255.0,
                g * 255.0,
                blue * 255.0,
                stroke_alpha,
            ),
        );
        let _ = line.set_attribute("stroke", "none");
        let _ = line.remove_attribute("stroke-opacity");
        let _ = line.remove_attribute("opacity");
        let _ = line.remove_attribute("stroke-width");
        let _ = line.remove_attribute("vector-effect");
        let _ = line.remove_attribute("marker-end");
    }
}

fn edge_overlay_path(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    stroke_width: f32,
) -> Option<String> {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let length = (dx * dx + dy * dy).sqrt();
    if length < 3.0 {
        return None;
    }

    let shaft_half = stroke_width * 0.5;
    let arrow_len = (stroke_width * 5.4).clamp(14.0, 20.0).min(length * 0.45);
    let arrow_half = (stroke_width * 2.35).clamp(6.0, 9.5).min(length * 0.35);
    let shaft_len = length - arrow_len;
    if shaft_len <= 1.0 {
        return None;
    }

    let ux = dx / length;
    let uy = dy / length;
    let nx = -uy;
    let ny = ux;
    let shaft_end_x = x2 - ux * arrow_len;
    let shaft_end_y = y2 - uy * arrow_len;

    let start_upper_x = x1 + nx * shaft_half;
    let start_upper_y = y1 + ny * shaft_half;
    let shaft_upper_x = shaft_end_x + nx * shaft_half;
    let shaft_upper_y = shaft_end_y + ny * shaft_half;
    let base_upper_x = shaft_end_x + nx * arrow_half;
    let base_upper_y = shaft_end_y + ny * arrow_half;
    let base_lower_x = shaft_end_x - nx * arrow_half;
    let base_lower_y = shaft_end_y - ny * arrow_half;
    let shaft_lower_x = shaft_end_x - nx * shaft_half;
    let shaft_lower_y = shaft_end_y - ny * shaft_half;
    let start_lower_x = x1 - nx * shaft_half;
    let start_lower_y = y1 - ny * shaft_half;

    Some(format!(
        "M {:.2} {:.2} L {:.2} {:.2} L {:.2} {:.2} L {:.2} {:.2} L {:.2} {:.2} L {:.2} {:.2} L {:.2} {:.2} Z",
        start_upper_x,
        start_upper_y,
        shaft_upper_x,
        shaft_upper_y,
        base_upper_x,
        base_upper_y,
        x2,
        y2,
        base_lower_x,
        base_lower_y,
        shaft_lower_x,
        shaft_lower_y,
        start_lower_x,
        start_lower_y,
    ))
}

fn edge_overlay_style(
    layout: &Layout3D,
    active_focus: Option<(&str, f32)>,
    edge: &EdgeRef3D,
    from_id: &str,
    to_id: &str,
) -> (f32, f32) {
    if let Some((focus_id, focus_flag)) = active_focus {
        if focus_id == from_id || focus_id == to_id {
            if focus_flag == EDGE_FLAG_SELECTED {
                return (3.2, 0.98);
            }
            return (2.8, 0.90);
        }
        return (1.3, 0.26);
    }

    let long_edge = if let (Some(a), Some(b)) = (
        layout.nodes.get(edge.from_idx),
        layout.nodes.get(edge.to_idx),
    ) {
        let dx = a.x - b.x;
        let dy = a.y - b.y;
        let dz = a.z - b.z;
        (dx * dx + dy * dy + dz * dz).sqrt() > 3.5
    } else {
        false
    };
    let (long_width, long_opacity, short_width, short_opacity) =
        match layout.node_card_profile {
            NodeCardProfile::Compact => (2.4, 0.82, 1.9, 0.64),
            NodeCardProfile::TicketWide => (2.6, 0.78, 2.1, 0.60),
        };
    if long_edge {
        (long_width, long_opacity)
    } else {
        (short_width, short_opacity)
    }
}

pub(crate) fn render_frame(
    state: &mut RenderState,
    frame: &crate::effects::FrameContext,
) {
    crate::profile_scope!("graph3d::render_frame");
    let window = web_sys::window();
    let Some(doc) = window.as_ref().and_then(|w| w.document()) else {
        return;
    };

    let dt = state
        .last_frame_time
        .map(|last| (frame.time_s - last).clamp(0.0, 0.1))
        .unwrap_or(1.0 / 60.0);
    state.last_frame_time = Some(frame.time_s);

    if let Some(goal) = state.camera_goal.clone() {
        if !animate_camera(&mut state.camera, &goal, dt, CAMERA_ANIMATION_SPEED)
        {
            state.camera_goal = None;
        }
    }
    if animate_layout_nodes(
        &mut state.layout,
        &state.target_layout,
        dt,
        LAYOUT_ANIMATION_SPEED,
    ) {
        state.dirty_layout = true;
    }

    // The overlay-driven loop already resized the canvas backing store and
    // hands us the current frame's swap-chain view. We only need to make
    // sure our depth texture matches the new size; CSS pixel size is
    // recomputed from `frame.canvas_w/h` divided by DPR for DOM positioning.
    let dpr = window
        .as_ref()
        .map(|w| w.device_pixel_ratio().clamp(1.0, 4.0))
        .unwrap_or(1.0) as f32;
    let css_w = (frame.canvas_w as f32) / dpr;
    let css_h = (frame.canvas_h as f32) / dpr;
    if frame.canvas_w != state.gpu.canvas_w
        || frame.canvas_h != state.gpu.canvas_h
    {
        state.gpu.depth_view = create_depth_view(
            &state.gpu.device,
            frame.canvas_w,
            frame.canvas_h,
        );
        state.gpu.canvas_w = frame.canvas_w;
        state.gpu.canvas_h = frame.canvas_h;
    }

    // Resolve the graph container's bounding rect so the camera and DOM
    // projection are both centred on the container, not the full canvas.
    let (cont_x_css, cont_y_css, cont_w_css, cont_h_css) = doc
        .get_element_by_id(&state.container_id)
        .map(|el| {
            let _ = el.set_attribute(
                "data-camera-distance",
                &format!("{:.4}", state.camera.distance),
            );
            let _ = el.set_attribute(
                "data-camera-target",
                &format!(
                    "{:.4},{:.4},{:.4}",
                    state.camera.target[0],
                    state.camera.target[1],
                    state.camera.target[2],
                ),
            );
            let r = el.get_bounding_client_rect();
            (
                r.left() as f32,
                r.top() as f32,
                r.width() as f32,
                r.height() as f32,
            )
        })
        .unwrap_or((0.0, 0.0, css_w, css_h));
    let (viewport_x_css, viewport_y_css, viewport_w_css, viewport_h_css) =
        resolve_viewport_rect(cont_w_css, cont_h_css, state.viewport_insets);
    // Physical-pixel viewport coordinates for setViewport / setScissorRect.
    let vp_x = ((cont_x_css + viewport_x_css) * dpr).round() as u32;
    let vp_y = ((cont_y_css + viewport_y_css) * dpr).round() as u32;
    let vp_w = ((viewport_w_css * dpr).round() as u32).max(1);
    let vp_h = ((viewport_h_css * dpr).round() as u32).max(1);
    let render_layout =
        current_render_layout(state, viewport_w_css, viewport_h_css);
    let uses_dynamic_layout = render_layout.is_some();

    // Re-upload per-instance buffers if a node moved this frame.
    if uses_dynamic_layout || state.dirty_layout || state.dirty_edges {
        let render_layout = render_layout.as_ref().unwrap_or(&state.layout);
        let (edge_data, edge_count) = render_layout.build_gpu_edge_instances();
        if !edge_data.is_empty() {
            write_buffer(&state.gpu.device, &state.edge_buf, &edge_data);
        }
        state.edge_count = edge_count;
        state.dirty_edges = false;
    }

    if uses_dynamic_layout || state.dirty_layout {
        let render_layout = render_layout.as_ref().unwrap_or(&state.layout);
        let (node_data, node_count) = render_layout.build_node_quads();
        if !node_data.is_empty() {
            write_buffer(&state.gpu.device, &state.node_quad_buf, &node_data);
        }
        state.node_count = node_count;
        state.dirty_layout = false;
    }

    let gpu = &state.gpu;

    // Camera uniform — use container aspect ratio so the projection centres
    // the graph on the container, not the full canvas.
    let eye = state.camera.eye();
    let aspect = viewport_w_css / viewport_h_css.max(1.0);
    let proj = match state.projection {
        Projection::Perspective =>
            math::perspective(CAMERA_FOV, aspect, CAMERA_NEAR, CAMERA_FAR),
        Projection::Orthographic => {
            let half_h = state.camera.distance * (CAMERA_FOV * 0.5).tan();
            math::orthographic(half_h, aspect, CAMERA_NEAR, CAMERA_FAR)
        },
    };
    let view = math::look_at(eye, state.camera.target, [0.0, 1.0, 0.0]);
    let vp_mat = math::mul(proj, view);

    let time = web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now() as f32 / 1000.0)
        .unwrap_or(0.0);

    let mut cam_data = [0.0f32; CAM_UNIFORM_FLOATS];
    cam_data[..16].copy_from_slice(&vp_mat);
    cam_data[16] = eye[0];
    cam_data[17] = eye[1];
    cam_data[18] = eye[2];
    cam_data[19] = 1.0;
    cam_data[20] = time;
    cam_data[21] = vp_w as f32;
    cam_data[22] = vp_h as f32;
    write_buffer(&gpu.device, &gpu.cam_buf, &cam_data);

    // Use the swap-chain view supplied by the overlay's per-frame callback.
    // `loadOp: "load"` preserves whatever the overlay's smoke / particle
    // pass already drew underneath, so the graph composites on top.
    let tex_view = frame.frame_view.clone();

    // Render pass descriptor (colour LOADs the existing overlay frame, depth
    // is cleared because we own it exclusively).
    let color_att = obj();
    set(&color_att, "view", &tex_view);
    set(&color_att, "loadOp", &js_str("load"));
    set(&color_att, "storeOp", &js_str("store"));
    let color_atts = Array::new();
    color_atts.push(&JsValue::from(color_att));
    let rp_desc = obj();
    set(&rp_desc, "colorAttachments", &JsValue::from(color_atts));

    let ds_att = obj();
    set(&ds_att, "view", &gpu.depth_view);
    set(&ds_att, "depthClearValue", &js_f64(1.0));
    set(&ds_att, "depthLoadOp", &js_str("clear"));
    set(&ds_att, "depthStoreOp", &js_str("store"));
    set(&rp_desc, "depthStencilAttachment", &JsValue::from(ds_att));

    let encoder = gpu.device.create_command_encoder();
    let encoder_js: JsValue = encoder.into();
    let pass_desc =
        web_sys::GpuRenderPassDescriptor::from(JsValue::from(rp_desc));
    let enc_typed: web_sys::GpuCommandEncoder =
        encoder_js.clone().dyn_into().unwrap();
    let Ok(pass_enc) = enc_typed.begin_render_pass(&pass_desc) else {
        return;
    };
    let pass: JsValue = JsValue::from(pass_enc);

    // Restrict GPU rendering to the container region so edges and node quads
    // don't bleed into the content panel on the right.
    // setViewport(x, y, width, height, minDepth, maxDepth) — 6 args.
    if let Ok(f) =
        js_sys::Reflect::get(&pass, &super::interop::js_str("setViewport"))
            .and_then(|v| v.dyn_into::<js_sys::Function>())
    {
        let vp_args = Array::new();
        vp_args.push(&js_f64(vp_x as f64));
        vp_args.push(&js_f64(vp_y as f64));
        vp_args.push(&js_f64(vp_w as f64));
        vp_args.push(&js_f64(vp_h as f64));
        vp_args.push(&js_f64(0.0));
        vp_args.push(&js_f64(1.0));
        let _ = f.apply(&pass, &vp_args);
    }
    // setScissorRect(x, y, width, height) — 4 args.
    if let Ok(f) =
        js_sys::Reflect::get(&pass, &super::interop::js_str("setScissorRect"))
            .and_then(|v| v.dyn_into::<js_sys::Function>())
    {
        let sc_args = Array::new();
        sc_args.push(&js_f64(vp_x as f64));
        sc_args.push(&js_f64(vp_y as f64));
        sc_args.push(&js_f64(vp_w as f64));
        sc_args.push(&js_f64(vp_h as f64));
        let _ = f.apply(&pass, &sc_args);
    }

    // Node occluder quads are intentionally skipped: all nodes are on the
    // flat z=0 plane and DOM cards render on top of the GPU canvas anyway,
    // so writing depth causes more edge clipping than it prevents.

    // Edges: depth-test only.
    if state.edge_count > 0 {
        js_call(&pass, "setPipeline", &[&gpu.edge_pipeline.clone().into()]);
        js_call(&pass, "setBindGroup", &[&js_f64(0.0), &gpu.bind_group]);
        js_call(
            &pass,
            "setVertexBuffer",
            &[&js_f64(0.0), &gpu.quad_buf.clone().into()],
        );
        js_call(
            &pass,
            "setVertexBuffer",
            &[&js_f64(1.0), &state.edge_buf.clone().into()],
        );
        js_call(
            &pass,
            "draw",
            &[&js_f64(4.0), &js_f64(state.edge_count as f64)],
        );
    }

    js_call(&pass, "end", &[]);

    let cmd_buf: JsValue = Reflect::get(&encoder_js, &js_str("finish"))
        .and_then(|f| f.dyn_into::<Function>())
        .ok()
        .and_then(|f| f.call0(&encoder_js).ok())
        .unwrap_or(JsValue::UNDEFINED);
    let bufs = Array::new();
    bufs.push(&cmd_buf);
    js_call(frame.queue, "submit", &[&JsValue::from(bufs)]);

    let render_layout = render_layout.as_ref().unwrap_or(&state.layout);

    position_dom_nodes(
        &doc,
        state,
        render_layout,
        &vp_mat,
        viewport_x_css,
        viewport_y_css,
        viewport_w_css,
        viewport_h_css,
    );

    let node_rects = compute_node_screen_rects(
        state,
        render_layout,
        &vp_mat,
        viewport_x_css,
        viewport_y_css,
        viewport_w_css,
        viewport_h_css,
    );

    position_dom_layout_anchors(
        &doc,
        state,
        render_layout,
        &vp_mat,
        &node_rects,
        viewport_x_css,
        viewport_y_css,
        viewport_w_css,
        viewport_h_css,
    );
    position_dom_edges(
        &doc,
        state,
        render_layout,
        &vp_mat,
        viewport_x_css,
        viewport_y_css,
        viewport_w_css,
        viewport_h_css,
    );
}

fn current_render_layout(
    state: &RenderState,
    viewport_width: f32,
    viewport_height: f32,
) -> Option<Layout3D> {
    if state.node_view_transform.is_active() {
        Some(apply_node_view_transform(
            &state.layout,
            &state.camera,
            viewport_width,
            viewport_height,
            state.node_view_transform,
        ))
    } else {
        None
    }
}

fn edge_visual_state(state: &RenderState) -> EdgeVisualState<'_> {
    EdgeVisualState {
        selected_node_id: state.selected_node_id.as_deref(),
        hovered_node_id: state.hovered_node_id.as_deref(),
    }
}
