// graph3d_edge.wgsl — Edge energy-beam shader for ticket-viewer 3D graph.
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

// ── Procedural noise for energy beam effects ──

fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3(p.xyx) * vec3(0.1031, 0.1030, 0.0973));
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn noise2d(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(hash21(i), hash21(i + vec2(1.0, 0.0)), u.x),
        mix(hash21(i + vec2(0.0, 1.0)), hash21(i + vec2(1.0, 1.0)), u.x),
        u.y
    );
}

fn fbm2(p: vec2<f32>) -> f32 {
    var val = 0.0;
    var amp = 0.5;
    var pos = p;
    for (var i = 0; i < 3; i++) {
        val += amp * noise2d(pos);
        pos *= 2.1;
        amp *= 0.5;
    }
    return val;
}

// ── Edge rendering (instanced energy beams between nodes) ──

struct EdgeVsOut {
    @builtin(position) pos : vec4<f32>,
    @location(0) color     : vec4<f32>,
    @location(1) edgeUV    : vec2<f32>,
    @location(2) flags     : f32,
    @location(3) edgeType  : f32,
    @location(4) edgeLen   : f32,
    // Normalised XY direction (posB - posA) for bounding-box endpoint trim.
    @location(5) edgeDir   : vec2<f32>,
};

// edgeType encoding:
//   0 = grid / simple (no animation)
//   1 = normal edge (subtle energy flow)

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

    var halfWidth: f32;
    if (edgeType < 0.5) {
        halfWidth = select(0.015, 0.035, flags > 0.5);
    } else if (edgeType < 1.5) {
        halfWidth = select(0.04, 0.06, flags > 0.5);
    } else {
        halfWidth = select(0.06, 0.08, flags > 0.5);
    }

    // ── Enforce a minimum screen-space pixel width ─────────────────────
    let viewport_h = max(cam.time.z, 1.0);
    let center_clip = cam.viewProj * vec4(center, 1.0);
    let depth_w = max(abs(center_clip.w), 0.0001);
    let world_per_px = (2.0 * 0.41421356 * depth_w) / viewport_h;
    let min_px = select(1.25, 1.0, edgeType < 0.5);
    let min_world = world_per_px * min_px;
    halfWidth = max(halfWidth, min_world);

    // ── Arrowhead vertex expansion (directed edges only) ────────────────────
    // Constant world-space arrowhead: ARROW_LEN world units long, ARROW_MULT × beam wide.
    // Node half-extents match fragment trim constants.
    const NODE_HALF_W_VS : f32 = 1.1;
    const NODE_HALF_H_VS : f32 = 0.30;
    const ARROW_LEN      : f32 = 0.55;   // world-space arrowhead length (constant)
    const ARROW_MULT     : f32 = 3.8;    // how many times wider than the beam at base

    if (edgeType > 0.5 && edgeLength > 0.001) {
        // World-space distance from posB at current vertex.
        let dist_from_B = (1.0 - pos01) * edgeLength;
        // Compute t_exit in world units: how far along the edge (in wu) the
        // edge enters/exits the node bounding box at each endpoint.
        let ndir_x = dir.x / edgeLength;
        let ndir_y = dir.y / edgeLength;
        let t_x = select(1.0e6, NODE_HALF_W_VS / max(abs(ndir_x), 0.0001), abs(ndir_x) > 0.001);
        let t_y = select(1.0e6, NODE_HALF_H_VS / max(abs(ndir_y), 0.0001), abs(ndir_y) > 0.001);
        let t_exit_world = clamp(min(t_x, t_y), 0.0, edgeLength * 0.38);
        // Expand from (t_exit_world + ARROW_LEN) down to t_exit_world at tip.
        let arrow_base_dist = t_exit_world + ARROW_LEN;
        if (dist_from_B < arrow_base_dist) {
            // tent: ramps up at base, back down at tip
            let arrow_t = clamp(dist_from_B / max(t_exit_world + 0.001, 0.001), 0.0, 1.0);
            let env = clamp((arrow_base_dist - dist_from_B) / max(ARROW_LEN, 0.001), 0.0, 1.0);
            halfWidth = max(halfWidth * (1.0 + (ARROW_MULT - 1.0) * env), min_world);
        }
    }

    let worldPos = center + side * quadPos.y * halfWidth;

    var out: EdgeVsOut;
    out.pos      = cam.viewProj * vec4(worldPos, 1.0);
    out.color    = color;
    out.edgeUV   = quadPos;
    out.flags    = flags;
    out.edgeType = edgeType;
    out.edgeLen  = edgeLength;
    out.edgeDir  = select(dir.xy / edgeLength, vec2(1.0, 0.0), edgeLength < 0.0001);
    return out;
}

@fragment
fn fs_edge(in: EdgeVsOut) -> @location(0) vec4<f32> {
    let t = in.edgeUV.x * 0.5 + 0.5;
    let across = abs(in.edgeUV.y);
    let side_sign = in.edgeUV.y;
    let time = cam.time.x;

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
        a *= endFade;
        return vec4(col * a, a);
    }

    // ── Energy beam rendering (edgeType >= 1) ──

    let beamAcross = across;

    let core      = exp(-beamAcross * beamAcross * 18.0);
    let innerGlow = exp(-beamAcross * beamAcross * 5.0);
    let outerGlow = exp(-beamAcross * beamAcross * 1.8);

    let flowSpeed = 1.2;
    let n1 = noise2d(vec2(t * 10.0 - time * flowSpeed, beamAcross * 5.0));
    let n2 = noise2d(vec2(t * 7.0 - time * flowSpeed * 0.6, beamAcross * 3.0 + 7.7));
    let plasma = n1 * 0.6 + n2 * 0.4;

    let turb = fbm2(vec2(t * 6.0 - time * flowSpeed * 0.8, side_sign * 3.0 + time * 0.3));

    let pulse1 = pow(0.5 + 0.5 * sin((t * 6.28318 * 3.0) - time * 4.0), 3.0);
    let pulse2 = pow(0.5 + 0.5 * sin((t * 6.28318 * 2.0) - time * 2.5 + 1.5), 2.0);

    var intensity = core * 0.7
        + innerGlow * 0.2 * (0.6 + 0.4 * plasma)
        + outerGlow * 0.08 * (0.5 + 0.5 * turb)
        + core * pulse1 * 0.25
        + innerGlow * pulse2 * 0.1;

    var col = in.color.rgb;
    var hotCenter = vec3(1.0);

    intensity *= 0.8;
    let subtlePulse = 0.5 + 0.5 * sin(t * 8.0 - time * 1.5);
    intensity += core * subtlePulse * 0.08;

    col = mix(col, hotCenter, core * 0.4);

    if (in.flags > 0.5) {
        col = mix(col, vec3(1.0), 0.15 * core);
        intensity *= 1.2;
    }

    // ── Fixed world-space bounding-box endpoint trim (connection points) ──────
    // Trim the edge at the node card surface so it appears to connect exactly
    // at the node boundary.  Both source (t≈0) and target (t≈1) are trimmed.
    //   NODE_HALF_W = 220px * 0.5 * scale(1/100) = 1.1 world units
    //   NODE_HALF_H =  60px * 0.5 * scale(1/100) = 0.30 world units
    const NODE_HALF_W: f32 = 1.1;
    const NODE_HALF_H: f32 = 0.30;
    // How far along t the edge enters/exits the node bounding box.
    let dx_abs = abs(in.edgeDir.x);
    let dy_abs = abs(in.edgeDir.y);
    let t_x = select(1.0e6, NODE_HALF_W / max(dx_abs * in.edgeLen, 0.0001), dx_abs > 0.001);
    let t_y = select(1.0e6, NODE_HALF_H / max(dy_abs * in.edgeLen, 0.0001), dy_abs > 0.001);
    let t_exit = clamp(min(t_x, t_y), 0.0, 0.38);

    if (t < t_exit || t > (1.0 - t_exit)) { discard; }

    // ── Constant world-space arrowhead at posB end ────────────────────────
    // Arrow: ARROW_LEN world units, starts just past the node boundary.
    const ARROW_LEN_F: f32 = 0.55;
    let arrow_start_t = 1.0 - t_exit - ARROW_LEN_F / max(in.edgeLen, 0.001);
    let visible_end = 1.0 - t_exit;

    // Smooth entry fade for both ends (avoids hard clip at connection point).
    let srcFade = smoothstep(0.0, 0.06, t - t_exit);
    let dstFade = smoothstep(0.0, 0.06, visible_end - t);

    // ── Connection point energy balls ─────────────────────────────────────
    // Compute glow contributions BEFORE any per-region discards so the ball
    // can extend slightly beyond the beam width at the endpoints.
    let ball_r2 = 0.0064;  // ball radius² in UV space (r = 0.08)
    let src_dt = t - t_exit;
    let dst_dt = t - visible_end;
    let src_d2 = src_dt * src_dt + in.edgeUV.y * in.edgeUV.y;
    let dst_d2 = dst_dt * dst_dt + in.edgeUV.y * in.edgeUV.y;
    let ball_glow = (exp(-src_d2 / ball_r2) + exp(-dst_d2 / ball_r2)) * 1.8;
    let ball_pulse = 0.82 + 0.18 * sin(time * 3.0 + t * 8.0);
    let ball_a = clamp(ball_glow * ball_pulse * in.color.a, 0.0, 1.0);
    let ball_col = mix(in.color.rgb, vec3(1.0), 0.80);

    // ── Beam / arrowhead intensity (non-discarding, clipped to 0 instead) ─
    var intensity_final = 0.0;

    if (t < arrow_start_t) {
        // Beam region: thin strip with soft AA edges.
        const BEAM_HALF: f32 = 0.32;
        let d = abs(in.edgeUV.y) - BEAM_HALF;
        if (d <= 0.10) {
            intensity_final = intensity * min(srcFade, dstFade)
                * (1.0 - smoothstep(0.0, 0.10, d));
        }
    } else {
        // Arrowhead: crisp triangle SDF.
        let arrow_t = clamp((t - arrow_start_t) / max(visible_end - arrow_start_t, 0.0001), 0.0, 1.0);
        const ARROW_HALF: f32 = 0.98;
        let tri_edge = ARROW_HALF * (1.0 - arrow_t);
        let d_arrow = abs(in.edgeUV.y) - tri_edge;
        const AA: f32 = 0.025;
        if (d_arrow <= AA) {
            intensity_final = intensity * srcFade
                * (1.0 - smoothstep(-AA * 0.5, AA, d_arrow))
                * (1.8 + 0.6 * (1.0 - arrow_t));
        }
    }

    let beam_a   = clamp(intensity_final * in.color.a * 1.6, 0.0, 1.0);
    let final_a  = clamp(beam_a + ball_a, 0.0, 1.0);
    if (final_a < 0.0008) { discard; }
    let final_col = (col * beam_a + ball_col * ball_a) / max(final_a, 0.001);
    return vec4(final_col * final_a, final_a);
}
