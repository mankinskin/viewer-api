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

    let worldPos = center + side * quadPos.y * halfWidth;

    var out: EdgeVsOut;
    out.pos      = cam.viewProj * vec4(worldPos, 1.0);
    out.color    = color;
    out.edgeUV   = quadPos;
    out.flags    = flags;
    out.edgeType = edgeType;
    out.edgeLen  = edgeLength;
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
        let alpha = 1.0 - smoothstep(0.6, 1.0, across);
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

    let endFadeA = smoothstep(0.0, 0.06, t);
    let endFadeB = smoothstep(0.0, 0.06, 1.0 - t);
    intensity *= min(endFadeA, endFadeB);

    let a = clamp(intensity * in.color.a * 1.6, 0.0, 1.0);
    return vec4(col * a, a);
}
