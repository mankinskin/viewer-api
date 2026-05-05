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
    // Normalised XY direction (posB - posA) for bounding-box endpoint trim.
    @location(5) edgeDir   : vec2<f32>,
};

// edgeType encoding:
//   0 = grid / simple
//   1 = normal directed edge

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
    } else {
        // Wider quad to accommodate arrowhead; constant for the entire edge.
        halfWidth = select(0.07, 0.10, flags > 0.5);
    }

    // ── Enforce a minimum screen-space pixel width ─────────────────────
    let viewport_h = max(cam.time.z, 1.0);
    let center_clip = cam.viewProj * vec4(center, 1.0);
    let depth_w = max(abs(center_clip.w), 0.0001);
    let world_per_px = (2.0 * 0.41421356 * depth_w) / viewport_h;
    let min_px = select(1.25, 1.5, edgeType < 0.5);
    let min_world = world_per_px * min_px;
    halfWidth = max(halfWidth, min_world);

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

    // ── Directed edge (edgeType >= 1) ──
    //
    // Layout (t = 0..1 along the edge, posA→posB):
    //   [t_src_exit .. arrow_start_t]  = shaft (constant width)
    //   [arrow_start_t .. t_dst_exit]  = arrowhead (triangle, widens then tapers to point)
    //
    // Node bounding-box trim:
    //   NODE_HALF_W = 260px * 0.5 * (1/100 scale) = 1.3 world units
    //   NODE_HALF_H =  70px * 0.5 * (1/100 scale) = 0.35 world units

    const NODE_HALF_W : f32 = 1.3;
    const NODE_HALF_H : f32 = 0.35;
    const ARROW_LEN_F : f32 = 0.45;   // world-space arrowhead length
    const SHAFT_HALF  : f32 = 0.22;   // shaft half-width in quad-UV space (y in -1..1)
    const ARROW_HALF  : f32 = 0.90;   // arrowhead max half-width at base (quad-UV)
    const AA          : f32 = 0.04;   // anti-alias softness

    // How far along t the edge is inside the node bounding box (trim both ends).
    let dx_abs = abs(in.edgeDir.x);
    let dy_abs = abs(in.edgeDir.y);
    let t_x = select(1.0e6, NODE_HALF_W / max(dx_abs * in.edgeLen, 0.0001), dx_abs > 0.001);
    let t_y = select(1.0e6, NODE_HALF_H / max(dy_abs * in.edgeLen, 0.0001), dy_abs > 0.001);
    let t_exit = clamp(min(t_x, t_y), 0.0, 0.38);

    // Discard inside node cards.
    if (t < t_exit || t > (1.0 - t_exit)) { discard; }

    let visible_end   = 1.0 - t_exit;
    let arrow_start_t = visible_end - ARROW_LEN_F / max(in.edgeLen, 0.001);

    // ── Determine inside/outside for the current region ──────────────────
    var inside = false;

    if (t < arrow_start_t) {
        // Shaft region — constant-width rectangle.
        inside = across <= SHAFT_HALF + AA;
    } else {
        // Arrowhead region — triangle that tapers to a point at visible_end.
        let arrow_t = clamp((t - arrow_start_t) / max(visible_end - arrow_start_t, 0.0001), 0.0, 1.0);
        let tri_edge = ARROW_HALF * (1.0 - arrow_t);
        inside = across <= tri_edge + AA;
    }

    if (!inside) { discard; }

    // ── Compute alpha ─────────────────────────────────────────────────────
    var a: f32;
    if (t < arrow_start_t) {
        let d = across - SHAFT_HALF;
        a = 1.0 - smoothstep(-AA, AA, d);
    } else {
        let arrow_t = clamp((t - arrow_start_t) / max(visible_end - arrow_start_t, 0.0001), 0.0, 1.0);
        let tri_edge = ARROW_HALF * (1.0 - arrow_t);
        let d = across - tri_edge;
        a = 1.0 - smoothstep(-AA, AA, d);
    }

    // Short fade-in from the source node edge to avoid a hard clip.
    let srcFade = smoothstep(0.0, 0.05, t - t_exit);
    a *= srcFade * in.color.a;

    if (a < 0.002) { discard; }

    var col = in.color.rgb;
    if (in.flags > 0.5) {
        col = mix(col, vec3(1.0), 0.25);
    }
    return vec4(col * a, a);
}
