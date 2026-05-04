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
    // Project the centre to clip space to figure out how many world units
    // map to one pixel at this depth, then bump halfWidth so the line is
    // never thinner than ~1 device pixel. This is the standard fix for
    // sub-pixel edge aliasing in 3-D line renderers.
    let viewport_h = max(cam.time.z, 1.0);
    // Vertical FOV is 45° (FRAC_PI_4) → tan(fov/2) ≈ 0.4142.
    let center_clip = cam.viewProj * vec4(center, 1.0);
    let depth_w = max(abs(center_clip.w), 0.0001);
    // World units per pixel at this depth.
    let world_per_px = (2.0 * 0.41421356 * depth_w) / viewport_h;
    // Minimum half-width: 1.0 px (subtle) for grid, 1.25 px for energy beams.
    let min_px = select(1.25, 1.0, edgeType < 0.5);
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
    // Normalised 2-D direction (flat graph lies on z=0 so z component is ~0).
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
        // Tight AA band (matches TS hypergraph.wgsl reference): keep core
        // fully opaque and only fade the outermost few % of the line so
        // sub-pixel-thin edges stay crisp at any distance.
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

    let sourceGlow = exp(-t * t * 6.0);
    let targetGlow = exp(-(1.0 - t) * (1.0 - t) * 8.0);

    var intensity = core * 0.7
        + innerGlow * 0.2 * (0.6 + 0.4 * plasma)
        + outerGlow * 0.08 * (0.5 + 0.5 * turb)
        + core * pulse1 * 0.25
        + innerGlow * pulse2 * 0.1;

    var col = in.color.rgb;
    var hotCenter = vec3(1.0);

    // Normal edge: subtle energy
    intensity *= 0.8;
    let subtlePulse = 0.5 + 0.5 * sin(t * 8.0 - time * 1.5);
    intensity += core * subtlePulse * 0.08;

    // Hot-core brightening
    col = mix(col, hotCenter, core * 0.4);

    if (in.flags > 0.5) {
        col = mix(col, vec3(1.0), 0.15 * core);
        intensity *= 1.2;
    }

    // ── Bounding-box endpoint trim ─────────────────────────────────────────
    // Discard fragments that fall inside the node card at either end of the
    // edge so the beam appears to connect at the card boundary rather than
    // running through the opaque DOM element.
    //
    // Card half-extents in world units (independent of camera distance D):
    //   half_w = (card_css_half_w × pixel_scale) / (pix_per_wu)
    //          = (110 × 15/D) / (viewport_h / (2 × 0.4142 × D))
    //          = 110 × 15 × 2 × 0.4142 / viewport_h
    //          = 1366 / viewport_h
    // pixel_scale constant 15 matches render.rs `(15.0 / dist).clamp(…)`.
    // card_css_half_w = 110 (half of 220 px).  card_css_half_h ≈ 32 px.
    let vp_h_2 = max(cam.time.z, 1.0);
    let card_half_w = 1366.0 / vp_h_2;   // world units
    let card_half_h =  394.0 / vp_h_2;   // world units  (32 px half-height)

    // Compute the normalised t value at which the edge exits the source card
    // (and by symmetry enters the destination card).
    let dx_abs = abs(in.edgeDir.x);
    let dy_abs = abs(in.edgeDir.y);
    let t_x = select(1.0e6, card_half_w / dx_abs, dx_abs > 0.001);
    let t_y = select(1.0e6, card_half_h / dy_abs, dy_abs > 0.001);
    // t_exit is in world units; divide by edgeLen to get a [0,1] parameter.
    let t_exit_raw = min(t_x, t_y) / max(in.edgeLen, 0.0001);
    // Clamp so at most 40% of each end is trimmed (avoids hiding very short
    // edges entirely when adjacent cards overlap in world space).
    let t_exit = clamp(t_exit_raw, 0.0, 0.40);

    if (t < t_exit || t > (1.0 - t_exit)) { discard; }

    // ── Endpoint fades ─────────────────────────────────────────────────────
    // ARROW_START is dynamic: place the arrowhead in the last 20% of the
    // *visible* edge portion so it always sits near the destination card.
    let visible_end = 1.0 - t_exit;
    let arrow_start = visible_end - max((visible_end - t_exit) * 0.20, 0.04);

    let endFadeA = smoothstep(0.0, 0.06, t - t_exit);
    // In the arrowhead zone suppress the tail-end fade so the tip stays bright.
    let endFadeB = select(smoothstep(0.0, 0.06, visible_end - t), 1.0, t > arrow_start);
    intensity *= min(endFadeA, endFadeB);

    // ── Directed arrowhead at posB (the dependency target) ────────────────
    // Tapers from full beam-width at the base to a sharp point at the visible
    // end.  `arrow_base_half = 1.0` makes the base exactly beam-width so it
    // looks like a natural continuation of the line.
    if t > arrow_start {
        let arrow_t = (t - arrow_start) / max(visible_end - arrow_start, 0.0001);
        let arrow_base_half = 1.0;   // 1.0 = beam half-width (edgeUV.y ∈ [-1,1])
        let allowed = arrow_base_half * (1.0 - arrow_t);
        if abs(in.edgeUV.y) > allowed { discard; }
        // Brighten the arrowhead so it pops against the background.
        intensity *= 1.5 + 0.8 * (1.0 - arrow_t);
    }

    let a = clamp(intensity * in.color.a * 1.6, 0.0, 1.0);
    return vec4(col * a, a);
}
