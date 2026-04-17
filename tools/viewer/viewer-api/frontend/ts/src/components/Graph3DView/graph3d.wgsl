// ── Hypergraph 3D View – shaders ──
//
// Concatenated after: palette.wgsl
// Effects operate in 3D world space.

struct Camera {
    viewProj : mat4x4<f32>,
    eye      : vec4<f32>,
    time     : vec4<f32>,   // x=time, y=beam_height, z=0, w=0
};

@group(0) @binding(0) var<uniform> cam : Camera;
@group(0) @binding(1) var<uniform> palette : ThemePalette;

// ══════════════════════════════════════════════════════
//  NODE RENDERING  (instanced billboard impostor spheres)
// ══════════════════════════════════════════════════════

struct NodeInstance {
    @location(2) center : vec3<f32>,    // world position
    @location(3) radius : f32,          // sphere radius
    @location(4) color  : vec4<f32>,    // base color + alpha
    @location(5) flags  : vec4<f32>,    // x=selected, y=hovered, z=isAtom, w=0
};

struct NodeVsOut {
    @builtin(position) pos   : vec4<f32>,
    @location(0) uv          : vec2<f32>,
    @location(1) worldCenter : vec3<f32>,
    @location(2) radius      : f32,
    @location(3) color       : vec4<f32>,
    @location(4) flags       : vec4<f32>,
};

@vertex
fn vs_node(
    @location(0) quadPos : vec2<f32>,   // −1..1 billboard quad
    inst : NodeInstance,
) -> NodeVsOut {
    // Build billboard in view space
    let right = normalize(vec3(cam.viewProj[0][0], cam.viewProj[1][0], cam.viewProj[2][0]));
    let up    = normalize(vec3(cam.viewProj[0][1], cam.viewProj[1][1], cam.viewProj[2][1]));

    let expand = 1.3;  // padding for AA edge
    let worldPos = inst.center
        + right * quadPos.x * inst.radius * expand
        + up    * quadPos.y * inst.radius * expand;

    var out: NodeVsOut;
    out.pos         = cam.viewProj * vec4(worldPos, 1.0);
    out.uv          = quadPos;
    out.worldCenter = inst.center;
    out.radius      = inst.radius;
    out.color       = inst.color;
    out.flags       = inst.flags;
    return out;
}

@fragment
fn fs_node(in: NodeVsOut) -> @location(0) vec4<f32> {
    let d = length(in.uv);
    if (d > 1.0) { discard; }

    // Sphere normal from billboard UV
    let z = sqrt(max(1.0 - d * d, 0.0));
    let right = normalize(vec3(cam.viewProj[0][0], cam.viewProj[1][0], cam.viewProj[2][0]));
    let up    = normalize(vec3(cam.viewProj[0][1], cam.viewProj[1][1], cam.viewProj[2][1]));
    let fwd   = normalize(cross(right, up));
    let N = normalize(right * in.uv.x + up * in.uv.y + fwd * z);

    let L = normalize(vec3(0.4, 0.8, 0.3));
    let V = normalize(cam.eye.xyz - in.worldCenter);
    let H = normalize(L + V);

    let ambient  = 0.18;
    let diffuse  = max(dot(N, L), 0.0) * 0.55;
    let spec     = pow(max(dot(N, H), 0.0), 40.0) * 0.35;
    let rim      = pow(1.0 - max(dot(N, V), 0.0), 3.0) * 0.15;
    let fresnel  = pow(1.0 - max(dot(N, V), 0.0), 4.0) * 0.25;

    var base = in.color.rgb;

    // Selected: glow ring
    if (in.flags.x > 0.5) {
        base = mix(base, vec3(1.0, 0.9, 0.4), 0.25);
        let ring = smoothstep(0.7, 0.85, d) * smoothstep(1.0, 0.92, d);
        let glow = ring * 0.6 * (0.7 + 0.3 * sin(cam.time.x * 3.0));
        let lit = ambient + diffuse + spec + rim + fresnel;
        return vec4(base * lit + vec3(glow * 0.8, glow * 0.6, glow * 0.1) + vec3(spec * 0.15), 1.0);
    }

    // Hovered: brightening
    if (in.flags.y > 0.5) {
        base = mix(base, vec3(1.0), 0.15);
    }

    let lit = ambient + diffuse + spec + rim;
    let aa = 1.0 - smoothstep(0.92, 1.0, d);
    return vec4((base * lit + vec3(spec * 0.12)) * aa, aa);
}


// ══════════════════════════════════════════════════════
//  EDGE RENDERING  (instanced energy beams between nodes)
// ══════════════════════════════════════════════════════

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

struct EdgeInstance {
    @location(2) posA   : vec3<f32>,    // start point
    @location(3) posB_x : f32,
    @location(4) posB_yz_color : vec4<f32>,  // yz = posB.yz, zw = color.rg
    @location(5) color_ba_flags : vec4<f32>, // xy = color.ba, z = flags, w = patternIdx
};

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
//   2 = search-path start (uniform teal, arrow toward A / parent)
//   3 = search-path root (gold, radiant bidirectional) [legacy]
//   4 = search-path end (uniform teal, arrow toward B / child)
//   5 = trace path (gentle flow)
//   6 = candidate edge (muted violet, transparent)
//   7 = insert edge
//   8 = search-path root entry (gold, arrow toward A / root)
//   9 = search-path root exit (gold, arrow toward B / child)

@vertex
fn vs_edge(
    @location(0) quadPos  : vec2<f32>,
    @location(6) posA     : vec3<f32>,
    @location(7) posB     : vec3<f32>,
    @location(8) color    : vec4<f32>,
    @location(9) flags    : f32,   // highlighted flag
    @location(10) edgeType : f32,  // beam type
) -> EdgeVsOut {
    let dir = posB - posA;
    let edgeLength = length(dir);
    let pos01 = quadPos.x * 0.5 + 0.5;  // 0..1 along line
    let center = mix(posA, posB, pos01);

    let viewDir = normalize(cam.eye.xyz - center);
    let lineDir = normalize(dir);
    let side = normalize(cross(lineDir, viewDir));

    // Width varies by edge type
    var halfWidth: f32;
    if (edgeType < 0.5) {
        // Grid: thin line
        halfWidth = select(0.015, 0.035, flags > 0.5);
    } else if (edgeType < 1.5) {
        // Normal edge: subtle beam
        halfWidth = select(0.03, 0.05, flags > 0.5);
    } else if (edgeType > 5.5 && edgeType < 6.5) {
        // Candidate edge (type 6): medium-thin
        halfWidth = 0.035;
    } else if ((edgeType > 1.5 && edgeType < 4.5) || (edgeType > 7.5 && edgeType < 9.5)) {
        // Search path edges with arrows (types 2,3,4,8,9): extra wide for arrowhead room
        halfWidth = select(0.10, 0.12, flags > 0.5);
    } else {
        // Trace path (type 5): moderate beam
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

// ── Arrow helper: computes arrowhead intensity ──
// arrowDir: 0.0 = arrow at A (low t), 1.0 = arrow at B (high t)
// edgeLen: world-space length of the edge, used for constant-size arrowheads
fn arrowHead(t_in: f32, across: f32, arrowDir: f32, edgeLen: f32) -> f32 {
    // Flip t so the arrow always points toward high values
    let t_a = select(1.0 - t_in, t_in, arrowDir > 0.5);

    // Fixed world-space arrow length (~0.25 units), clamped to at most 40% of edge
    let arrowFrac = clamp(0.25 / max(edgeLen, 0.01), 0.04, 0.40);
    let arrowStart = 1.0 - arrowFrac;
    let fadeWidth = arrowFrac * 0.1;  // soft transition at base
    let inArrow = smoothstep(arrowStart - fadeWidth, arrowStart, t_a);
    let arrowProgress = clamp((t_a - arrowStart) / arrowFrac, 0.0, 1.0);

    // Triangle shape: wide at base, narrows to point
    // across is 0..1 from centre; triangle boundary shrinks linearly
    let triangleBound = (1.0 - arrowProgress) * 0.85;
    let arrowShape = smoothstep(0.04, 0.0, across - triangleBound) * inArrow;

    // Bright edge outline of the triangle
    let edgeDist = abs(across - triangleBound);
    let arrowEdge = exp(-edgeDist * edgeDist * 800.0) * inArrow * 0.6;

    return arrowShape * 0.7 + arrowEdge;
}

@fragment
fn fs_edge(in: EdgeVsOut) -> @location(0) vec4<f32> {
    let t = in.edgeUV.x * 0.5 + 0.5;       // 0..1 along beam (A=0, B=1)
    let across = abs(in.edgeUV.y);           // 0..1 from center outward
    let side_sign = in.edgeUV.y;             // signed lateral position
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

    // ── Candidate edges (edgeType 6): muted, transparent, gentle pulse ──
    if (in.edgeType > 5.5 && in.edgeType < 6.5) {
        let core_c = exp(-across * across * 12.0);
        let glow_c = exp(-across * across * 3.5);
        let gentlePulse = 0.5 + 0.5 * sin(t * 8.0 - time * 1.0);
        let intensity_c = core_c * 0.45 + glow_c * 0.15 * gentlePulse;
        var col_c = in.color.rgb;
        col_c = mix(col_c, vec3(0.7, 0.6, 0.9), 0.2);  // slight violet tint
        let endFade_c = smoothstep(0.0, 0.08, min(t, 1.0 - t));
        let a_c = clamp(intensity_c * in.color.a * endFade_c, 0.0, 1.0);
        return vec4(col_c * a_c, a_c);
    }

    // ── Energy beam rendering (edgeType 1-5) ──

    // For path edges (2,3,4,8,9), narrow the beam core relative to the wide quad
    // so the arrowhead can spread wider than the beam body.
    let isArrowType = (in.edgeType > 1.5 && in.edgeType < 4.5) || (in.edgeType > 7.5 && in.edgeType < 9.5);
    // Beam occupies the central ~40% of quad width for arrow types
    let beamScale = select(1.0, 2.2, isArrowType);
    let beamAcross = across * beamScale;  // stretched so core stays narrow

    // Radial beam profiles (using beamAcross for arrow types)
    let core     = exp(-beamAcross * beamAcross * 18.0);           // tight hot center
    let innerGlow = exp(-beamAcross * beamAcross * 5.0);           // medium glow
    let outerGlow = exp(-beamAcross * beamAcross * 1.8);           // soft halo

    // Animated noise layers (flow from A→B)
    let flowSpeed = select(1.2, 2.5, in.edgeType > 1.5);
    let n1 = noise2d(vec2(t * 10.0 - time * flowSpeed, beamAcross * 5.0));
    let n2 = noise2d(vec2(t * 7.0 - time * flowSpeed * 0.6, beamAcross * 3.0 + 7.7));
    let plasma = n1 * 0.6 + n2 * 0.4;

    // FBM turbulence for wispy tendrils
    let turb = fbm2(vec2(t * 6.0 - time * flowSpeed * 0.8, side_sign * 3.0 + time * 0.3));

    // Traveling pulse waves (A→B direction)
    let pulse1 = pow(0.5 + 0.5 * sin((t * 6.28318 * 3.0) - time * 4.0), 3.0);
    let pulse2 = pow(0.5 + 0.5 * sin((t * 6.28318 * 2.0) - time * 2.5 + 1.5), 2.0);

    // Asymmetric endpoint glow
    let sourceGlow = exp(-t * t * 6.0);             // peaks at A
    let targetGlow = exp(-(1.0 - t) * (1.0 - t) * 8.0);  // peaks at B

    // ── Compose base intensity ──
    var intensity = core * 0.7
        + innerGlow * 0.2 * (0.6 + 0.4 * plasma)
        + outerGlow * 0.08 * (0.5 + 0.5 * turb)
        + core * pulse1 * 0.25
        + innerGlow * pulse2 * 0.1;

    var col = in.color.rgb;
    var hotCenter = vec3(1.0);  // white-hot for core brightening

    // ── Per-type effects ──
    if (in.edgeType > 1.5 && in.edgeType < 2.5) {
        // ═══ SP START (type 2): uniform teal, arrow at A (toward parent) ═══
        // Arrow points toward posA (parent). Flow travels B→A (child→parent).
        let flowPulse = pow(0.5 + 0.5 * sin(t * 20.0 + time * 5.0), 3.0);  // reversed flow
        intensity += core * flowPulse * 0.25;
        intensity += sourceGlow * innerGlow * 0.3;
        // Arrowhead at A end (arrowDir = 0.0)
        let arrow = arrowHead(t, across, 0.0, in.edgeLen);
        intensity += arrow * 0.8;
        hotCenter = vec3(0.85, 1.0, 1.0);

    } else if (in.edgeType > 2.5 && in.edgeType < 3.5) {
        // ═══ SP ROOT legacy (type 3): golden radiance, bidirectional ═══
        let centerDist = abs(t - 0.5);
        let biPulse = pow(0.5 + 0.5 * sin(centerDist * 20.0 - time * 5.0), 3.0);
        intensity += core * biPulse * 0.4;
        let center_glow = exp(-centerDist * centerDist * 12.0);
        intensity += center_glow * innerGlow * 0.5;
        let shimmer = noise2d(vec2(t * 15.0, time * 3.0));
        intensity += core * shimmer * 0.15;
        hotCenter = vec3(1.0, 0.95, 0.75);
        col = mix(col, vec3(1.0, 0.9, 0.5), center_glow * 0.3);

    } else if (in.edgeType > 3.5 && in.edgeType < 4.5) {
        // ═══ SP END (type 4): uniform teal, arrow at B (toward child) ═══
        // Arrow points toward posB (child). Flow travels A→B (parent→child).
        let flowPulse = pow(0.5 + 0.5 * sin(t * 20.0 - time * 5.0), 3.0);
        intensity += core * flowPulse * 0.25;
        intensity += targetGlow * innerGlow * 0.3;
        // Arrowhead at B end (arrowDir = 1.0)
        let arrow = arrowHead(t, across, 1.0, in.edgeLen);
        intensity += arrow * 0.8;
        hotCenter = vec3(0.85, 1.0, 1.0);

    } else if (in.edgeType > 4.5 && in.edgeType < 5.5) {
        // ═══ TRACE PATH (type 5): gentle teal flow ═══
        let gentlePulse = 0.5 + 0.5 * sin(t * 12.56 - time * 2.0);
        intensity += core * gentlePulse * 0.15;
        intensity += sourceGlow * outerGlow * 0.2;
        intensity += targetGlow * outerGlow * 0.2;
        hotCenter = vec3(0.85, 1.0, 1.0);

    } else if (in.edgeType > 7.5 && in.edgeType < 8.5) {
        // ═══ SP ROOT ENTRY (type 8): golden radiance, arrow at A (toward root) ═══
        // Search arrived upward at the root node from the start path.
        let centerDist_re = abs(t - 0.5);
        let center_glow_re = exp(-centerDist_re * centerDist_re * 12.0);
        let shimmer_re = noise2d(vec2(t * 15.0, time * 3.0));
        intensity += center_glow_re * innerGlow * 0.35;
        intensity += core * shimmer_re * 0.12;
        // Flow travels B→A (child→parent / upward toward root)
        let flowPulse_re = pow(0.5 + 0.5 * sin(t * 20.0 + time * 5.0), 3.0);
        intensity += core * flowPulse_re * 0.25;
        intensity += sourceGlow * innerGlow * 0.3;
        // Arrowhead at A end (toward root/parent, arrowDir = 0.0)
        let arrow_re = arrowHead(t, across, 0.0, in.edgeLen);
        intensity += arrow_re * 0.8;
        hotCenter = vec3(1.0, 0.95, 0.75);
        col = mix(col, vec3(1.0, 0.9, 0.5), center_glow_re * 0.2);

    } else if (in.edgeType > 8.5 && in.edgeType < 9.5) {
        // ═══ SP ROOT EXIT (type 9): golden radiance, arrow at B (toward child) ═══
        // Search continues downward from root into the end path.
        let centerDist_rx = abs(t - 0.5);
        let center_glow_rx = exp(-centerDist_rx * centerDist_rx * 12.0);
        let shimmer_rx = noise2d(vec2(t * 15.0, time * 3.0));
        intensity += center_glow_rx * innerGlow * 0.35;
        intensity += core * shimmer_rx * 0.12;
        // Flow travels A→B (parent→child / downward from root)
        let flowPulse_rx = pow(0.5 + 0.5 * sin(t * 20.0 - time * 5.0), 3.0);
        intensity += core * flowPulse_rx * 0.25;
        intensity += targetGlow * innerGlow * 0.3;
        // Arrowhead at B end (toward child, arrowDir = 1.0)
        let arrow_rx = arrowHead(t, across, 1.0, in.edgeLen);
        intensity += arrow_rx * 0.8;
        hotCenter = vec3(1.0, 0.95, 0.75);
        col = mix(col, vec3(1.0, 0.9, 0.5), center_glow_rx * 0.2);

    } else {
        // ═══ NORMAL edge (type 1): subtle energy ═══
        intensity *= 0.8;
        let subtlePulse = 0.5 + 0.5 * sin(t * 8.0 - time * 1.5);
        intensity += core * subtlePulse * 0.08;
    }

    // ── Hot-core brightening (white center like a plasma arc) ──
    col = mix(col, hotCenter, core * 0.4);

    // Highlight whitening
    if (in.flags > 0.5) {
        col = mix(col, vec3(1.0), 0.15 * core);
        intensity *= 1.2;
    }

    // Endpoint fade (skip arrow tip end for arrow types)
    var endFadeA = smoothstep(0.0, 0.06, t);
    var endFadeB = smoothstep(0.0, 0.06, 1.0 - t);
    // Don't fade the arrow tip — let the arrowhead geometry define the end
    if (in.edgeType > 1.5 && in.edgeType < 2.5) {
        endFadeA = 1.0;  // type 2: arrow at A end, don't fade there
    }
    if (in.edgeType > 7.5 && in.edgeType < 8.5) {
        endFadeA = 1.0;  // type 8: root entry arrow at A end, don't fade there
    }
    if (in.edgeType > 3.5 && in.edgeType < 4.5) {
        endFadeB = 1.0;  // type 4: arrow at B end, don't fade there
    }
    if (in.edgeType > 8.5 && in.edgeType < 9.5) {
        endFadeB = 1.0;  // type 9: root exit arrow at B end, don't fade there
    }
    intensity *= min(endFadeA, endFadeB);

    // Final output with premultiplied alpha
    let a = clamp(intensity * in.color.a * 1.6, 0.0, 1.0);
    return vec4(col * a, a);
}


// ══════════════════════════════════════════════════════
//  LABEL TEXT  (reserved for future: SDF text rendering)
// ══════════════════════════════════════════════════════
