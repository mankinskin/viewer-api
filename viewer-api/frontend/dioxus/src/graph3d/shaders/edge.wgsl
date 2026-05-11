// graph3d_edge.wgsl — Edge shader for ticket-viewer 3D graph.
//
// Composed from:
//   palette.wgsl (ThemePalette struct)
//   graph3d.wgsl (edge rendering only — no node impostor code)

struct ThemePalette {
    spark_core     : vec4f,
    spark_ember    : vec4f,
    spark_steel    : vec4f,
    ember_hot      : vec4f,
    beam_center    : vec4f,
    beam_edge      : vec4f,
    glitter_warm   : vec4f,
    glitter_cool   : vec4f,
    cinder_ember   : vec4f,
    cinder_gold    : vec4f,
    cinder_ash     : vec4f,
    cinder_vine    : vec4f,
    smoke_cool     : vec4f,
    smoke_warm     : vec4f,
    smoke_moss     : vec4f,
    kind_structural : vec4f,
    kind_error      : vec4f,
    kind_warn       : vec4f,
    kind_info       : vec4f,
    kind_debug      : vec4f,
    kind_span       : vec4f,
    kind_selected   : vec4f,
    kind_panic      : vec4f,
    _pad            : vec4f,
};

struct Camera {
    viewProj : mat4x4<f32>,
    eye      : vec4<f32>,
    time     : vec4<f32>,
};

@group(0) @binding(0) var<uniform> cam : Camera;
@group(0) @binding(1) var<uniform> palette : ThemePalette;

// ── Edge rendering (instanced directed edges between nodes) ──

struct EdgeVsOut {
    @builtin(position) pos : vec4<f32>,
    @location(0) color     : vec4<f32>,
    @location(1) edgeUV    : vec2<f32>,
    @location(2) flags     : f32,
    @location(3) edgeType  : f32,
    @location(4) edgeLen   : f32,
    // World-space t where the shaft exits the source / enters the destination card.
    @location(5) srcExitT  : f32,
    @location(6) dstExitT  : f32,
    @location(7) eyeDist   : f32,
};

fn screen_fraction_to_world_t(s: f32, inv_w_a: f32, inv_w_b: f32) -> f32 {
    let denom = (1.0 - s) * inv_w_a + s * inv_w_b;
    if (denom <= 0.000001) {
        return clamp(s, 0.0, 1.0);
    }
    return clamp((s * inv_w_b) / denom, 0.0, 1.0);
}

fn wrapped_pulse(coord: f32, center: f32, width: f32) -> f32 {
    let delta = abs(coord - center);
    let wrapped = min(delta, 1.0 - delta);
    return 1.0 - smoothstep(width, width * 2.3, wrapped);
}

// edgeType encoding:
//   0 = grid / simple
//   1 = directed edge with compact shared graph cards
//   2 = directed edge with wide ticket-viewer cards

@vertex
fn vs_edge(
    @location(0) quadPos  : vec2<f32>,
    @location(6) posA     : vec3<f32>,
    @location(7) posB     : vec3<f32>,
    @location(8) color    : vec4<f32>,
    @location(9) flags    : f32,
    @location(10) edgeType : f32,
) -> EdgeVsOut {
    let dir = posB - posA;
    let edgeLength = length(dir);
    let pos01 = quadPos.x * 0.5 + 0.5;
    let center = mix(posA, posB, pos01);

    let viewDir = normalize(cam.eye.xyz - center);
    let lineDir = normalize(dir);
    let side = normalize(cross(lineDir, viewDir));
    let is_selected = flags > 1.5;
    let is_hovered = flags > 0.5 && !is_selected;
    let is_dimmed = flags < -0.5;

    var halfWidth: f32;
    if (edgeType < 0.5) {
        halfWidth = 0.008;
        if (flags > 0.5) {
            halfWidth = 0.020;
        }
    } else {
        halfWidth = 0.16;
        if (is_hovered) {
            halfWidth = 0.20;
        }
        if (is_selected) {
            halfWidth = 0.24;
        }
        if (is_dimmed) {
            halfWidth = 0.12;
        }
    }

    // ── Enforce a minimum screen-space pixel width ─────────────────────
    let viewport_h = max(cam.time.z, 1.0);
    let center_clip = cam.viewProj * vec4(center, 1.0);
    let depth_w = max(abs(center_clip.w), 0.0001);
    let world_per_px = (2.0 * 0.41421356 * depth_w) / viewport_h;
    var min_px = 3.8;
    if (edgeType < 0.5) {
        min_px = 0.75;
    }
    if (is_dimmed && edgeType >= 0.5) {
        min_px = 2.8;
    }
    if (is_hovered) {
        min_px = 4.6;
    }
    if (is_selected) {
        min_px = 5.2;
    }
    let min_world = world_per_px * min_px;
    halfWidth = max(halfWidth, min_world);

    let clip_a = cam.viewProj * vec4(posA, 1.0);
    let clip_b = cam.viewProj * vec4(posB, 1.0);
    let inv_w_a = 1.0 / max(abs(clip_a.w), 0.0001);
    let inv_w_b = 1.0 / max(abs(clip_b.w), 0.0001);
    let ndc_a = clip_a.xy * inv_w_a;
    let ndc_b = clip_b.xy * inv_w_b;
    let screen_a = vec2(
        (ndc_a.x + 1.0) * 0.5 * cam.time.y,
        (1.0 - ndc_a.y) * 0.5 * cam.time.z,
    );
    let screen_b = vec2(
        (ndc_b.x + 1.0) * 0.5 * cam.time.y,
        (1.0 - ndc_b.y) * 0.5 * cam.time.z,
    );
    let screen_dir = screen_b - screen_a;
    let screen_len = max(length(screen_dir), 0.0001);
    let screen_unit = screen_dir / screen_len;
    let dx_abs = abs(screen_unit.x);
    let dy_abs = abs(screen_unit.y);
    let card_w_px = select(170.0, 260.0, edgeType > 1.5);
    let card_h_px = select(44.0, 56.0, edgeType > 1.5);

    let dist_a = max(length(cam.eye.xyz - posA), 0.1);
    let dist_b = max(length(cam.eye.xyz - posB), 0.1);
    let pixel_scale_a = clamp(22.0 / dist_a, 0.14, 3.5);
    let pixel_scale_b = clamp(22.0 / dist_b, 0.14, 3.5);
    let half_w_a = card_w_px * pixel_scale_a * 0.5;
    let half_h_a = card_h_px * pixel_scale_a * 0.5;
    let half_w_b = card_w_px * pixel_scale_b * 0.5;
    let half_h_b = card_h_px * pixel_scale_b * 0.5;
    let src_exit_px_x = select(1.0e6, half_w_a / max(dx_abs, 0.0001), dx_abs > 0.001);
    let src_exit_px_y = select(1.0e6, half_h_a / max(dy_abs, 0.0001), dy_abs > 0.001);
    let dst_exit_px_x = select(1.0e6, half_w_b / max(dx_abs, 0.0001), dx_abs > 0.001);
    let dst_exit_px_y = select(1.0e6, half_h_b / max(dy_abs, 0.0001), dy_abs > 0.001);
    let src_exit_s = clamp(min(src_exit_px_x, src_exit_px_y) / screen_len, 0.0, 0.49);
    let dst_exit_s = clamp(min(dst_exit_px_x, dst_exit_px_y) / screen_len, 0.0, 0.49);
    let src_exit_t = screen_fraction_to_world_t(src_exit_s, inv_w_a, inv_w_b);
    let dst_exit_t = screen_fraction_to_world_t(1.0 - dst_exit_s, inv_w_a, inv_w_b);

    let worldPos = center + side * quadPos.y * halfWidth;

    var out: EdgeVsOut;
    out.pos      = cam.viewProj * vec4(worldPos, 1.0);
    out.color    = color;
    out.edgeUV   = quadPos;
    out.flags    = flags;
    out.edgeType = edgeType;
    out.edgeLen  = edgeLength;
    out.srcExitT = src_exit_t;
    out.dstExitT = dst_exit_t;
    out.eyeDist  = distance(cam.eye.xyz, center);
    return out;
}

@fragment
fn fs_edge(in: EdgeVsOut) -> @location(0) vec4<f32> {
    let t = in.edgeUV.x * 0.5 + 0.5;
    let across = abs(in.edgeUV.y);

    // ── Grid / simple edges (edgeType 0) ──
    if (in.edgeType < 0.5) {
        let alpha = 1.0 - smoothstep(0.92, 1.0, across);
        var col = in.color.rgb;
        var a = in.color.a * alpha;
        if (in.flags > 0.5) {
            col = mix(col, vec3(1.0), 0.3);
            a *= 1.4;
        }
        let endFade = smoothstep(0.0, 0.08, 0.5 - abs(in.edgeUV.x));
        a *= endFade * 0.65;
        return vec4(col * a, a);
    }

    // ── Directed edge (edgeType >= 1) ──
    //
    // Layout (t = 0..1 along the edge, posA→posB):
    //   [t_src_exit .. arrow_start_t]  = shaft (constant width)
    //   [arrow_start_t .. t_dst_exit]  = arrowhead (triangle, widens then tapers to point)
    //
    const ARROW_LEN_F : f32 = 0.45;   // world-space arrowhead length
    const SHAFT_HALF  : f32 = 0.30;   // shaft half-width in quad-UV space (y in -1..1)
    const ARROW_HALF  : f32 = 0.90;   // arrowhead max half-width at base (quad-UV)
    const AA          : f32 = 0.04;   // anti-alias softness
    let visible_start = min(in.srcExitT, in.dstExitT);
    let visible_end = max(in.srcExitT, in.dstExitT);

    // Discard inside node cards.
    if (t < visible_start || t > visible_end) { discard; }

    let arrow_start_t = clamp(visible_end - ARROW_LEN_F / max(in.edgeLen, 0.001), visible_start, visible_end);

    // ── Determine inside/outside for the current region ──────────────────
    var inside = false;
    var shape_half = SHAFT_HALF;

    if (t < arrow_start_t) {
        // Shaft region — constant-width rectangle.
        inside = across <= SHAFT_HALF + AA;
    } else {
        // Arrowhead region — triangle that tapers to a point at visible_end.
        let arrow_t = clamp((t - arrow_start_t) / max(visible_end - arrow_start_t, 0.0001), 0.0, 1.0);
        let tri_edge = ARROW_HALF * (1.0 - arrow_t);
        shape_half = tri_edge;
        inside = across <= tri_edge + AA;
    }

    if (!inside) { discard; }

    // ── Compute alpha ─────────────────────────────────────────────────────
    var a: f32;
    if (t < arrow_start_t) {
        let d = across - SHAFT_HALF;
        shape_half = SHAFT_HALF;
        a = 1.0 - smoothstep(-AA, AA, d);
    } else {
        let arrow_t = clamp((t - arrow_start_t) / max(visible_end - arrow_start_t, 0.0001), 0.0, 1.0);
        let tri_edge = ARROW_HALF * (1.0 - arrow_t);
        shape_half = tri_edge;
        let d = across - tri_edge;
        a = 1.0 - smoothstep(-AA, AA, d);
    }

    // Short fade-in from the source node edge to avoid a hard clip.
    let srcFade = smoothstep(0.0, 0.05, t - visible_start);
    a *= srcFade * in.color.a;

    if (a < 0.002) { discard; }

    let visible_len = max(visible_end - visible_start, 0.0001);
    let norm_t = clamp((t - visible_start) / visible_len, 0.0, 1.0);
    let profile = clamp(across / max(shape_half + AA, 0.0001), 0.0, 1.0);
    let core_mask = 1.0 - smoothstep(0.0, 0.25, profile);
    let inner_mask = 1.0 - smoothstep(0.18, 0.72, profile);
    let halo_mask = 1.0 - smoothstep(0.55, 1.0, profile);
    let direction_bias = 0.82 + 0.18 * smoothstep(0.10, 0.95, norm_t);
    let depth_soft = clamp(1.36 - in.eyeDist / 135.0, 0.84, 1.15);
    let is_selected = in.flags > 1.5;
    let is_hovered = in.flags > 0.5 && !is_selected;
    let is_dimmed = in.flags < -0.5;

    var edge_rgb = mix(in.color.rgb, palette.beam_edge.rgb, 0.12);
    var halo_rgb = mix(edge_rgb, palette.glitter_cool.rgb, 0.16);
    var core_rgb = mix(edge_rgb, palette.beam_center.rgb, 0.22);
    var packet_rgb = palette.spark_core.rgb;
    var alpha_scale = 2.35;
    var packet_strength = 0.0;

    if (is_selected) {
        edge_rgb = mix(palette.kind_selected.rgb, palette.beam_center.rgb, 0.28);
        halo_rgb = mix(palette.spark_ember.rgb, edge_rgb, 0.45);
        core_rgb = mix(edge_rgb, palette.spark_core.rgb, 0.42);
        packet_rgb = mix(palette.spark_core.rgb, palette.beam_center.rgb, 0.40);
        alpha_scale = 2.75;
        packet_strength = 1.0;
    } else if (is_hovered) {
        edge_rgb = mix(palette.kind_info.rgb, palette.glitter_cool.rgb, 0.35);
        halo_rgb = mix(palette.smoke_cool.rgb, edge_rgb, 0.60);
        core_rgb = mix(edge_rgb, palette.glitter_cool.rgb, 0.36);
        packet_rgb = mix(palette.glitter_cool.rgb, palette.spark_core.rgb, 0.28);
        alpha_scale = 2.30;
        packet_strength = 0.72;
    } else if (is_dimmed) {
        edge_rgb = mix(in.color.rgb, palette.cinder_ash.rgb, 0.68);
        halo_rgb = mix(edge_rgb, palette.smoke_cool.rgb, 0.76);
        core_rgb = mix(edge_rgb, halo_rgb, 0.25);
        alpha_scale = 0.95;
    }

    var lit_rgb = halo_rgb * (0.26 + 0.32 * halo_mask);
    lit_rgb += edge_rgb * (0.55 + 0.85 * inner_mask) * direction_bias;
    lit_rgb += core_rgb * (0.72 + 1.18 * core_mask);

    var packet = 0.0;
    if (packet_strength > 0.0) {
        let seed = fract(in.edgeLen * 0.173 + in.color.r * 0.37 + in.color.g * 0.53);
        let packet_coord = fract(norm_t * 3.0 - cam.time.x * (0.55 + 0.35 * packet_strength) + seed);
        packet = wrapped_pulse(packet_coord, 0.5, 0.18) * core_mask;
        lit_rgb += packet_rgb * packet * (1.0 + packet_strength);
    }

    var opacity = a * depth_soft * alpha_scale;
    opacity *= 1.08 + 0.38 * inner_mask + 0.32 * core_mask;
    opacity = clamp(opacity + packet * packet_strength * 0.22, 0.0, 1.0);

    if (opacity < 0.002) { discard; }
    return vec4(lit_rgb * opacity, opacity);
}
