// background.wgsl — fullscreen scene rendering (Dark Souls cinder theme)
//
// Concatenated after: types.wgsl + noise.wgsl
// Contains: fullscreen quad VS, scene element rendering, smoky background,
//           CRT post-processing, fragment entry point.
//
// Canvas sits BEHIND HTML (z-index -1, opaque).  HTML backgrounds are
// transparent so the dark texture shows through.

// ---- bindings (render pass — read-only) -------------------------------------

@group(0) @binding(0) var<uniform>       u         : Uniforms;
@group(0) @binding(1) var<storage, read> elems     : array<ElemRect>;
@group(0) @binding(2) var<storage, read> particles : array<Particle>;

// ---- fullscreen quad vertex shader ------------------------------------------

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4f {
    var pos = array<vec2f, 6>(
        vec2f(-1.0, -1.0), vec2f( 1.0, -1.0), vec2f(-1.0,  1.0),
        vec2f( 1.0, -1.0), vec2f( 1.0,  1.0), vec2f(-1.0,  1.0),
    );
    return vec4f(pos[vi], 0.0, 1.0);
}

// ---- edge helpers -----------------------------------------------------------

fn edge_dist(px: vec2f, ex: f32, ey: f32, ew: f32, eh: f32) -> f32 {
    let dx = min(px.x - ex, ex + ew - px.x);
    let dy = min(px.y - ey, ey + eh - px.y);
    return min(dx, dy);
}

fn perimeter_t(px: vec2f, ex: f32, ey: f32, ew: f32, eh: f32) -> f32 {
    let perim = 2.0 * (ew + eh);
    let lx = px.x - ex;
    let ly = px.y - ey;
    if ly < lx && ly < (eh - ly) && ly < (ew - lx) { return lx / perim; }
    if (ew - lx) < ly && (ew - lx) < (eh - ly)     { return (ew + ly) / perim; }
    if (eh - ly) < lx && (eh - ly) < (ew - lx)      { return (ew + eh + (ew - lx)) / perim; }
    return (2.0 * ew + eh + (eh - ly)) / perim;
}

// ---- rounded-rect SDF ------------------------------------------------------

fn rounded_rect_sdf(px: vec2f, ex: f32, ey: f32, ew: f32, eh: f32, radius: f32) -> f32 {
    let center = vec2f(ex + ew * 0.5, ey + eh * 0.5);
    let half   = vec2f(ew * 0.5, eh * 0.5);
    let rel    = abs(px - center) - half + vec2f(radius);
    return length(max(rel, vec2f(0.0))) + min(max(rel.x, rel.y), 0.0) - radius;
}

// ---- hover proximity --------------------------------------------------------

fn hover_proximity(ex: f32, ey: f32, ew: f32, eh: f32) -> f32 {
    let mouse  = vec2f(u.mouse_x, u.mouse_y);
    let center = vec2f(ex + ew * 0.5, ey + eh * 0.5);
    let dist   = length(mouse - center);
    return smoothstep(max(ew, eh) * 0.8, 0.0, dist);
}

// ---- graph node (kind 8) — dark stone / iron slab ---------------------------

fn graph_node(px: vec2f, ex: f32, ey: f32, ew: f32, eh: f32,
              hue: f32, node_type: f32, t: f32, prox: f32) -> vec4f {
    let radius = 6.0;
    let sdf = rounded_rect_sdf(px, ex, ey, ew, eh, radius);
    if sdf > 4.0 { return vec4f(0.0); }

    let body_mask = smoothstep(0.5, -0.5, sdf);
    let nx = (px.x - ex) / ew;
    let ny = (px.y - ey) / eh;

    // Stone material: dark grey with subtle noise grain
    let stone_noise = smooth_noise(px * 0.15) * 0.08;
    let stone_base  = vec3f(0.16, 0.15, 0.14) + vec3f(stone_noise);
    let stone_top   = stone_base + vec3f(0.06, 0.05, 0.04);
    let stone_bot   = stone_base * 0.7;
    var fill_rgb    = mix(stone_top, stone_bot, ny);

    // Subtle vine veins
    let vine_n = smooth_noise(px * 0.08 + vec2f(3.7, 1.2));
    let vine_streak = smoothstep(0.48, 0.52, vine_n) * 0.3;
    fill_rgb = mix(fill_rgb, vec3f(0.12, 0.30, 0.10), vine_streak);

    // Mouse-based torch lighting
    let mouse  = vec2f(u.mouse_x, u.mouse_y);
    let center = vec2f(ex + ew * 0.5, ey + eh * 0.5);
    let to_mouse = normalize(mouse - center + vec2f(0.001));
    let normal   = vec2f((nx - 0.5) * 0.3, (ny - 0.5) * 0.5);
    let diffuse  = max(0.0, dot(normalize(normal + vec2f(0.0, -0.3)), to_mouse));
    let torch_col = vec3f(0.9, 0.5, 0.15);
    let lb = prox * 0.7;
    fill_rgb = fill_rgb + torch_col * diffuse * (0.10 + lb * 0.4);

    // Specular — dull metal sheen
    let spec_pos = vec2f(nx - 0.5, ny - 0.3);
    let spec = pow(max(0.0, 1.0 - length(spec_pos - to_mouse * 0.2) * 2.5), 12.0);
    fill_rgb = fill_rgb + vec3f(0.6, 0.5, 0.3) * spec * (0.05 + lb * 0.15);

    // Chiselled top edge gleam
    fill_rgb = fill_rgb + vec3f(0.5, 0.45, 0.35) * smoothstep(0.12, 0.0, ny) * 0.15;
    fill_rgb = fill_rgb * (1.0 - smoothstep(0.88, 1.0, ny) * 0.2);

    // Node type accent — vine (enter) or blood (exit)
    let ntype = u32(node_type);
    if ntype == 1u {
        let bar = smoothstep(3.0, 0.0, px.x - ex) * smoothstep(-1.0, 0.0, sdf);
        fill_rgb = mix(fill_rgb, vec3f(0.15, 0.40, 0.10), bar * 0.7);
    } else if ntype == 2u {
        let bar = smoothstep(3.0, 0.0, (ex + ew) - px.x) * smoothstep(-1.0, 0.0, sdf);
        fill_rgb = mix(fill_rgb, vec3f(0.55, 0.08, 0.05), bar * 0.7);
    }

    // Iron border with ember glow on hover
    let border_band = smoothstep(-1.5, 0.0, sdf) * smoothstep(2.0, 0.5, sdf);
    let ember_pulse = 0.5 + 0.5 * sin(t * 2.0 + nx * 6.28);
    let border_rgb  = mix(vec3f(0.10, 0.09, 0.08), vec3f(0.7, 0.25, 0.05), prox * ember_pulse * 0.6);
    fill_rgb = fill_rgb + border_rgb * border_band * 0.5;

    // Deep shadow beneath
    let outer       = smoothstep(4.0, 0.0, sdf) * (1.0 - body_mask);
    let outer_alpha = outer * 0.15;
    let shadow_sdf  = rounded_rect_sdf(px - vec2f(-1.0, 4.0 + prox * 3.0), ex, ey, ew, eh, radius);
    let shadow_mask = smoothstep(0.0, 10.0, -shadow_sdf) * (1.0 - body_mask);
    let shadow_alpha = shadow_mask * 0.35 * (0.6 + prox * 0.4);

    let body_alpha = body_mask * 0.80;
    let total_rgb  = fill_rgb * body_alpha + vec3f(0.08, 0.06, 0.04) * outer_alpha;
    let total_a    = body_alpha + outer_alpha + shadow_alpha;

    return vec4f(total_rgb, total_a);
}

// ---- GPU cursor rendering ---------------------------------------------------

// Signed distance to a line segment (a → b), returns distance from point p
fn sd_segment(p: vec2f, a: vec2f, b: vec2f) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

// Arrow cursor SDF — proper pointer shape with shaft notch
fn cursor_arrow_sdf(p: vec2f) -> f32 {
    // 7-vertex polygon:  tip → right diagonal → notch-in → shaft-bottom-right
    //                   → shaft-bottom-left → notch-left → left edge
    let v0 = vec2f(0.0,  0.0);    // tip
    let v1 = vec2f(16.0, 16.8);   // right diagonal end
    let v2 = vec2f(6.8,  14.0);   // notch inward
    let v3 = vec2f(11.0, 24.0);   // shaft bottom-right
    let v4 = vec2f(5.5,  24.0);   // shaft bottom-left
    let v5 = vec2f(4.0,  16.0);   // notch left
    let v6 = vec2f(0.0,  22.0);   // left edge bottom

    // Winding-number polygon SDF (7 edges)
    var d = dot(p - v0, p - v0);
    var s = 1.0;

    // Helper: per-edge distance + winding update
    // Edge v0→v6
    var e = v6 - v0; var w = p - v0;
    var b2 = w - e * clamp(dot(w, e) / dot(e, e), 0.0, 1.0);
    d = min(d, dot(b2, b2));
    var c0 = w.y >= 0.0; var c1 = e.y * w.x > e.x * w.y; var c2 = w.y >= e.y;
    if ((c0 && c1 && !c2) || (!c0 && !c1 && c2)) { s *= -1.0; }

    // Edge v6→v5
    e = v5 - v6; w = p - v6;
    b2 = w - e * clamp(dot(w, e) / dot(e, e), 0.0, 1.0);
    d = min(d, dot(b2, b2));
    c0 = w.y >= 0.0; c1 = e.y * w.x > e.x * w.y; c2 = w.y >= e.y;
    if ((c0 && c1 && !c2) || (!c0 && !c1 && c2)) { s *= -1.0; }

    // Edge v5→v4
    e = v4 - v5; w = p - v5;
    b2 = w - e * clamp(dot(w, e) / dot(e, e), 0.0, 1.0);
    d = min(d, dot(b2, b2));
    c0 = w.y >= 0.0; c1 = e.y * w.x > e.x * w.y; c2 = w.y >= e.y;
    if ((c0 && c1 && !c2) || (!c0 && !c1 && c2)) { s *= -1.0; }

    // Edge v4→v3
    e = v3 - v4; w = p - v4;
    b2 = w - e * clamp(dot(w, e) / dot(e, e), 0.0, 1.0);
    d = min(d, dot(b2, b2));
    c0 = w.y >= 0.0; c1 = e.y * w.x > e.x * w.y; c2 = w.y >= e.y;
    if ((c0 && c1 && !c2) || (!c0 && !c1 && c2)) { s *= -1.0; }

    // Edge v3→v2
    e = v2 - v3; w = p - v3;
    b2 = w - e * clamp(dot(w, e) / dot(e, e), 0.0, 1.0);
    d = min(d, dot(b2, b2));
    c0 = w.y >= 0.0; c1 = e.y * w.x > e.x * w.y; c2 = w.y >= e.y;
    if ((c0 && c1 && !c2) || (!c0 && !c1 && c2)) { s *= -1.0; }

    // Edge v2→v1
    e = v1 - v2; w = p - v2;
    b2 = w - e * clamp(dot(w, e) / dot(e, e), 0.0, 1.0);
    d = min(d, dot(b2, b2));
    c0 = w.y >= 0.0; c1 = e.y * w.x > e.x * w.y; c2 = w.y >= e.y;
    if ((c0 && c1 && !c2) || (!c0 && !c1 && c2)) { s *= -1.0; }

    // Edge v1→v0
    e = v0 - v1; w = p - v1;
    b2 = w - e * clamp(dot(w, e) / dot(e, e), 0.0, 1.0);
    d = min(d, dot(b2, b2));
    c0 = w.y >= 0.0; c1 = e.y * w.x > e.x * w.y; c2 = w.y >= e.y;
    if ((c0 && c1 && !c2) || (!c0 && !c1 && c2)) { s *= -1.0; }

    return s * sqrt(d);
}

// ---- procedural texture helpers for cursors ---------------------------------

// Voronoi cell noise — returns (cell_distance, cell_id) for metallic grain
fn voronoi(p: vec2f) -> vec2f {
    let ip = floor(p);
    let fp = fract(p);
    var min_d = 8.0;
    var cell_id = 0.0;
    for (var j = -1; j <= 1; j++) {
        for (var i = -1; i <= 1; i++) {
            let neighbor = vec2f(f32(i), f32(j));
            let point = vec2f(hash2(ip + neighbor + vec2f(0.0, 0.0)),
                              hash2(ip + neighbor + vec2f(17.3, 31.7)));
            let diff = neighbor + point - fp;
            let dist = dot(diff, diff);
            if (dist < min_d) {
                min_d = dist;
                cell_id = hash2(ip + neighbor + vec2f(53.1, 97.3));
            }
        }
    }
    return vec2f(sqrt(min_d), cell_id);
}

// Medium-detail FBM for cursor texturing (3 octaves, rotated)
fn fbm3(p_in: vec2f) -> f32 {
    var val  = 0.0;
    var amp  = 0.5;
    var freq = 1.0;
    var p    = p_in;
    for (var i = 0; i < 3; i++) {
        val  += amp * smooth_noise(p * freq);
        amp  *= 0.5;
        freq *= 2.1;
        p = vec2f(p.x * 0.866 - p.y * 0.5, p.x * 0.5 + p.y * 0.866);
    }
    return val;
}

// ---- Metal cursor — forged dark iron with hammer marks, rust, patina --------

fn cursor_metal(px: vec2f, mouse: vec2f, t: f32) -> vec4f {
    let local = px - mouse;

    // Expanded bounding box for shadow
    if (local.x < -4.0 || local.x > 22.0 || local.y < -4.0 || local.y > 30.0) {
        return vec4f(0.0);
    }

    let sdf = cursor_arrow_sdf(local);

    // Anti-aliased edge (sub-pixel smooth)
    let aa = 1.0 - smoothstep(-1.2, 0.6, sdf);
    if (aa < 0.001) { return vec4f(0.0); }

    let uv = local / vec2f(16.0, 24.0);

    // ── Surface normal with height-field detail ──
    let dome = vec2f((uv.x - 0.4) * 0.5, (uv.y - 0.45) * 0.3);

    // Hammer-strike dents — single pass
    let dent_h = smooth_noise(local * 0.35 + vec2f(42.0, 17.0)) * 0.15;

    // Forged grain — directional, precomputed rotation
    let grain_p = vec2f(
        local.x * 0.9888 - local.y * 0.1494,
        local.x * 0.1494 + local.y * 0.9888
    );
    let grain = smooth_noise(vec2f(grain_p.x * 6.0, grain_p.y * 0.8 + 200.0)) * 0.06;

    // Fine micro-scratches (single anisotropic pass)
    let scratch1 = smooth_noise(vec2f(local.x * 12.0, local.y * 1.5 + 500.0)) * 0.025;

    // Simplified normal from height field
    let eps = 0.5;
    let h_center = dent_h + grain + scratch1;
    let h_right  = smooth_noise((local + vec2f(eps, 0.0)) * 0.35 + vec2f(42.0, 17.0)) * 0.09
                 + smooth_noise(vec2f((grain_p.x + eps) * 6.0, grain_p.y * 0.8 + 200.0)) * 0.06;
    let h_up     = smooth_noise((local + vec2f(0.0, eps)) * 0.35 + vec2f(42.0, 17.0)) * 0.09
                 + smooth_noise(vec2f(grain_p.x * 6.0, (grain_p.y + eps) * 0.8 + 200.0)) * 0.06;

    let normal = normalize(vec3f(
        dome.x + (h_center - h_right) * 3.0,
        dome.y + (h_center - h_up) * 3.0,
        1.0
    ));

    // ── Material: dark forged iron with rust + patina combined ──
    let base_iron = vec3f(0.32, 0.30, 0.28);

    // Single noise-based grain variation (replaces voronoi)
    let crystal_var = smooth_noise(local * 0.8 + vec2f(73.0, 11.0));
    let crystal_tint = mix(vec3f(0.30, 0.28, 0.26), vec3f(0.36, 0.34, 0.30), crystal_var);

    // Rust + patina in one fbm3 pass
    let weathering = fbm3(local * 0.3 + vec2f(15.0, 27.0));
    let rust_mask = smoothstep(0.35, 0.65, weathering);
    let rust_col = mix(vec3f(0.35, 0.18, 0.08), vec3f(0.50, 0.25, 0.10), weathering);
    let rust_amount = rust_mask * smoothstep(0.3, 0.7, dent_h / 0.15) * 0.45;
    let patina_amount = (1.0 - rust_mask) * 0.2 * (1.0 - rust_amount * 2.0);

    // Combine base material
    var metal_col = mix(crystal_tint, base_iron, grain * 8.0);
    metal_col = mix(metal_col, rust_col, rust_amount);
    metal_col = mix(metal_col, vec3f(0.15, 0.18, 0.25), patina_amount);

    // Brushed highlight streaks
    let brush_streak = pow(smooth_noise(vec2f(grain_p.x * 15.0, grain_p.y * 0.4 + 400.0)), 3.0) * 0.12;
    metal_col = metal_col + vec3f(brush_streak);

    // ── Lighting: PBR-ish with two lights ──
    let view = vec3f(0.0, 0.0, 1.0);

    // Key light: warm upper-left
    let light1 = normalize(vec3f(-0.5, -0.8, 1.0));
    let diff1 = max(dot(normal, light1), 0.0);
    let half1 = normalize(light1 + view);
    // Roughness varies: rust is rough, polished metal is sharp
    let roughness = mix(0.3, 0.9, rust_amount + patina_amount * 0.5);
    let spec_power = mix(80.0, 8.0, roughness);
    let spec1 = pow(max(dot(normal, half1), 0.0), spec_power) * mix(1.2, 0.15, roughness);

    // Fill light: cool from lower-right
    let light2 = normalize(vec3f(0.6, 0.3, 0.8));
    let diff2 = max(dot(normal, light2), 0.0) * 0.3;
    let half2 = normalize(light2 + view);
    let spec2 = pow(max(dot(normal, half2), 0.0), spec_power * 0.5) * mix(0.4, 0.05, roughness);

    // Ambient occlusion from SDF (edges darker)
    let ao = smoothstep(0.0, 5.0, -sdf) * 0.3 + 0.7;

    // Fresnel rim
    let fresnel = pow(1.0 - max(dot(normal, view), 0.0), 4.0);
    let rim_col = vec3f(0.4, 0.42, 0.5) * fresnel * 0.25;

    let ambient = vec3f(0.06, 0.055, 0.05);
    var col = metal_col * (ambient + (diff1 * vec3f(1.0, 0.95, 0.85) + diff2 * vec3f(0.7, 0.8, 1.0)) * ao)
            + vec3f(spec1) * vec3f(1.0, 0.95, 0.88) * (1.0 - rust_amount)
            + vec3f(spec2) * vec3f(0.7, 0.8, 1.0) * (1.0 - rust_amount)
            + rim_col;

    // Subtle heat shimmer near the tip (this IS a cinder theme)
    let tip_glow = exp(-length(local) * 0.15) * 0.08;
    col = col + vec3f(tip_glow * 0.8, tip_glow * 0.3, tip_glow * 0.05);

    // ── Dark forged border (bevelled edge) ──
    let bevel = smoothstep(0.5, -1.5, sdf);
    let bevel_light = max(dot(normalize(vec3f(-sign(sdf) * 0.5, -sign(sdf) * 0.3, 1.0)), light1), 0.0);
    col = mix(vec3f(0.05, 0.04, 0.03), col, bevel);
    col = col + vec3f(bevel_light * 0.1) * (1.0 - bevel);

    // ── Drop shadow ──
    let shadow_sdf = cursor_arrow_sdf(local - vec2f(2.0, 2.5));
    let shadow = (1.0 - smoothstep(-3.0, 2.0, shadow_sdf)) * 0.45;

    let shadow_result = vec4f(0.0, 0.0, 0.0, shadow);
    let cursor_result = vec4f(col, aa);
    let out_a = cursor_result.a + shadow_result.a * (1.0 - cursor_result.a);
    let out_rgb = (cursor_result.rgb * cursor_result.a + shadow_result.rgb * shadow_result.a * (1.0 - cursor_result.a)) / max(out_a, 0.001);
    return vec4f(out_rgb, out_a);
}

// ---- Glass cursor — crystal with internal fractures, caustics, dispersion ---

fn cursor_glass(px: vec2f, mouse: vec2f, t: f32) -> vec4f {
    let local = px - mouse;

    if (local.x < -6.0 || local.x > 24.0 || local.y < -6.0 || local.y > 34.0) {
        return vec4f(0.0);
    }

    let sdf = cursor_arrow_sdf(local);

    let aa = 1.0 - smoothstep(-1.2, 0.6, sdf);
    if (aa < 0.001) { return vec4f(0.0); }

    let uv = local / vec2f(16.0, 24.0);

    // ── Glass surface normals: thick convex lens ──
    let dome_strength = 0.7;
    let dome_x = (uv.x - 0.4) * dome_strength;
    let dome_y = (uv.y - 0.45) * dome_strength * 0.7;

    // Wavy imperfections in glass surface (2 layers instead of 3)
    let wave1 = smooth_noise(local * 0.6 + vec2f(t * 0.08, 7.0)) * 0.14;
    let wave2 = smooth_noise(local * 1.3 + vec2f(13.0, t * 0.06)) * 0.07;

    let eps = 0.4;
    let h_c = wave1 + wave2;
    let h_r = smooth_noise((local + vec2f(eps, 0.0)) * 0.6 + vec2f(t * 0.08, 7.0)) * 0.14
            + smooth_noise((local + vec2f(eps, 0.0)) * 1.3 + vec2f(13.0, t * 0.06)) * 0.07;
    let h_u = smooth_noise((local + vec2f(0.0, eps)) * 0.6 + vec2f(t * 0.08, 7.0)) * 0.14
            + smooth_noise((local + vec2f(0.0, eps)) * 1.3 + vec2f(13.0, t * 0.06)) * 0.07;

    let normal = normalize(vec3f(
        dome_x + (h_c - h_r) * 2.5,
        dome_y + (h_c - h_u) * 2.5,
        1.0
    ));

    // ── Internal structure: fractures, bubbles (single voronoi pass) ──
    let fracture_vor = voronoi(local * 0.5 + vec2f(100.0, 200.0));
    let fracture_lines = smoothstep(0.12, 0.08, fracture_vor.x) * 0.3;

    // Use hash for bubbles instead of second voronoi
    let bubble_h = smooth_noise(local * 1.5 + vec2f(300.0, 400.0));
    let bubbles = smoothstep(0.85, 0.95, bubble_h) * 0.5;

    // Simplified internal caustic (single smooth_noise instead of fbm3)
    let internal_caustic = smooth_noise(local * 0.15 + normal.xy * 3.0 + vec2f(t * 0.12, -t * 0.08));

    // ── Refraction: chromatic aberration (R/G/B refract differently) ──
    let refract_base = 10.0;
    let r_offset = normal.xy * (refract_base * 1.05);
    let g_offset = normal.xy * (refract_base * 1.00);
    let b_offset = normal.xy * (refract_base * 0.95);

    // Sample "background" at three offset positions (simulated via noise)
    let bg_scale = 0.008;
    let bg_r = smooth_noise((px + r_offset) * bg_scale + vec2f(t * 0.03, 0.0)) * 0.12 + 0.04;
    let bg_g = smooth_noise((px + g_offset) * bg_scale + vec2f(0.0, t * 0.03)) * 0.13 + 0.045;
    let bg_b = smooth_noise((px + b_offset) * bg_scale + vec2f(t * 0.02, t * 0.02)) * 0.14 + 0.05;
    var refracted = vec3f(bg_r, bg_g, bg_b);

    // Tint by glass body colour (very slight blue-green)
    let glass_tint = vec3f(0.85, 0.92, 0.95);
    refracted = refracted * glass_tint;

    // Add internal structures
    refracted = refracted + vec3f(fracture_lines * 0.7, fracture_lines * 0.8, fracture_lines);
    refracted = refracted + vec3f(bubbles * 0.8, bubbles * 0.9, bubbles);
    refracted = refracted + vec3f(internal_caustic * 0.04, internal_caustic * 0.05, internal_caustic * 0.06);

    // ── Fresnel: Schlick's approximation ──
    let view = vec3f(0.0, 0.0, 1.0);
    let n_dot_v = max(dot(normal, view), 0.0);
    let f0 = 0.04;  // glass IOR ~1.5
    let fresnel = f0 + (1.0 - f0) * pow(1.0 - n_dot_v, 5.0);

    // ── Reflection: environment approximation (multi-layer) ──
    let refl_dir = reflect(-view, normal);
    let refl_uv1 = refl_dir.xy * 5.0 + vec2f(t * 0.06, -t * 0.04);
    let refl_uv2 = refl_dir.xy * 12.0 + vec2f(-t * 0.03, t * 0.07);
    let refl1 = smooth_noise(refl_uv1) * 0.25 + 0.08;
    let refl2 = smooth_noise(refl_uv2) * 0.1;
    let reflection = vec3f(refl1 + refl2) * vec3f(0.9, 0.95, 1.0);

    // ── Specular highlights: two lights ──
    // Key: sharp point light upper-left
    let light1 = normalize(vec3f(-0.4, -0.7, 1.0));
    let half1 = normalize(light1 + view);
    let spec1 = pow(max(dot(normal, half1), 0.0), 128.0) * 1.5;

    // Fill: softer warm light from right
    let light2 = normalize(vec3f(0.7, -0.2, 0.9));
    let half2 = normalize(light2 + view);
    let spec2 = pow(max(dot(normal, half2), 0.0), 64.0) * 0.4;

    // ── Edge caustics: rainbow dispersion along borders ──
    let edge_d = abs(sdf);
    let edge_bright = smoothstep(3.5, 0.0, edge_d);

    // Travelling rainbow wave along the perimeter
    let perim_t = atan2(local.y - 12.0, local.x - 6.0); // angle around center
    let rainbow_phase = perim_t * 2.0 + t * 0.8 + sdf * 0.5;
    let caustic_r = sin(rainbow_phase) * 0.5 + 0.5;
    let caustic_g = sin(rainbow_phase + 2.094) * 0.5 + 0.5;
    let caustic_b = sin(rainbow_phase + 4.189) * 0.5 + 0.5;
    let caustic = vec3f(caustic_r, caustic_g, caustic_b) * edge_bright * 0.4;

    // Secondary: internal total-internal-reflection caustic bands
    let tir_bands = pow(sin(sdf * 1.2 + t * 0.3) * 0.5 + 0.5, 4.0) * edge_bright * 0.2;

    // ── Compose ──
    var col = mix(refracted, reflection, fresnel)
            + vec3f(spec1) * vec3f(1.0, 0.98, 0.95)
            + vec3f(spec2) * vec3f(1.0, 0.95, 0.85)
            + caustic
            + vec3f(tir_bands * 0.5, tir_bands * 0.7, tir_bands);

    // Glass alpha: mostly transparent body, opaque at edges (Fresnel)
    let body_alpha = 0.18 + fresnel * 0.55;

    // Bright crisp edge highlight (like polished glass bevels catching light)
    let edge_highlight = smoothstep(1.2, 0.0, edge_d) * 0.7;
    let edge_shadow_inner = smoothstep(0.0, 2.5, edge_d) * smoothstep(4.0, 2.5, edge_d) * 0.15;
    col = col + vec3f(edge_highlight * 0.9, edge_highlight * 0.95, edge_highlight);
    col = col - vec3f(edge_shadow_inner * 0.3);

    // ── Drop shadow (soft, slightly coloured by caustics) ──
    let shadow_sdf = cursor_arrow_sdf(local - vec2f(2.0, 3.0));
    let shadow_base = (1.0 - smoothstep(-4.0, 3.0, shadow_sdf)) * 0.25;
    // Caustic light leaking into shadow
    let shadow_caustic_phase = shadow_sdf * 0.6 + t * 0.4;
    let sc_r = sin(shadow_caustic_phase) * 0.5 + 0.5;
    let sc_g = sin(shadow_caustic_phase + 2.094) * 0.5 + 0.5;
    let sc_b = sin(shadow_caustic_phase + 4.189) * 0.5 + 0.5;
    let shadow_caustic_bright = smoothstep(3.0, 0.0, abs(shadow_sdf + 1.0)) * 0.12;
    let shadow_col_rgb = vec3f(sc_r, sc_g, sc_b) * shadow_caustic_bright;

    let cursor_a = clamp(aa * (body_alpha + edge_highlight * 0.3), 0.0, 1.0);
    let shadow_result = vec4f(shadow_col_rgb, shadow_base);
    let cursor_result = vec4f(col, cursor_a);
    let out_a = cursor_result.a + shadow_result.a * (1.0 - cursor_result.a);
    let out_rgb = (cursor_result.rgb * cursor_result.a + shadow_result.rgb * shadow_result.a * (1.0 - cursor_result.a)) / max(out_a, 0.001);
    return vec4f(out_rgb, out_a);
}

// Dispatch cursor rendering based on style uniform
fn gpu_cursor(px: vec2f, mouse: vec2f, style: f32, t: f32) -> vec4f {
    if (style < 0.5) { return vec4f(0.0); }       // 0 = default (no GPU cursor)
    if (style < 1.5) { return cursor_metal(px, mouse, t); } // 1 = metal
    return cursor_glass(px, mouse, t);                       // 2 = glass
}

// ---- CRT post-processing ---------------------------------------------------

fn crt_scanlines(py: f32, width: f32) -> f32 {
    // width 0.0 = thin crisp lines (freq ~pi), 1.0 = wide soft bands (freq ~0.3*pi)
    let freq = mix(3.14159, 0.9, width);
    let depth = mix(0.18, 0.35, width);  // wider lines = deeper modulation
    return (1.0 - depth) + depth * sin(py * freq);
}

fn crt_vertical_lines(px_x: f32, width: f32) -> f32 {
    let freq = mix(3.14159 * 0.6667, 0.6, width);
    let depth = mix(0.12, 0.25, width);
    return (1.0 - depth) + depth * sin(px_x * freq);
}

// Pixel-grid opacity effect — screen-door pattern (no colour shift)
fn crt_pixel_grid(px: vec2f) -> f32 {
    let cell = 3.0;
    // Horizontal gap between pixel cells
    let gx = smoothstep(0.0, 0.6, px.x % cell)
           * smoothstep(cell, cell - 0.6, px.x % cell);
    // Vertical gap between pixel cells
    let gy = smoothstep(0.0, 0.6, px.y % cell)
           * smoothstep(cell, cell - 0.6, px.y % cell);
    // Mix: mostly opaque, subtle grid darkening
    return mix(1.0, gx * gy, 0.22);
}

fn crt_edge_shadow(uv: vec2f) -> f32 {
    let d_left   = uv.x;
    let d_right  = 1.0 - uv.x;
    let d_top    = uv.y;
    let d_bottom = 1.0 - uv.y;
    let d = min(min(d_left, d_right), min(d_top, d_bottom));
    return smoothstep(0.0, 0.04, d) * (0.7 + 0.3 * smoothstep(0.0, 0.15, d));
}

// ---- static thin shadow for non-hovered elements ----------------------------

fn static_shadow(px: vec2f, ex: f32, ey: f32, ew: f32, eh: f32) -> f32 {
    let inside_x = px.x >= ex && px.x < ex + ew;
    let inside_y = px.y >= ey && px.y < ey + eh;
    if !(inside_x && inside_y) { return 0.0; }
    let dist = edge_dist(px, ex, ey, ew, eh);
    return smoothstep(3.0, 0.0, dist) * 0.20;
}

// ---- ember hover border — smouldering cracks along edges --------------------

fn hover_border(px: vec2f, ex: f32, ey: f32, ew: f32, eh: f32,
                hue: f32, t: f32, prox: f32) -> vec4f {
    let cs = max(u.cinder_size, 0.01);
    let margin = 4.0 * cs;
    let inside_x = px.x >= ex - margin && px.x < ex + ew + margin;
    let inside_y = px.y >= ey - margin && px.y < ey + eh + margin;
    if !(inside_x && inside_y) { return vec4f(0.0); }

    if !(inside_x && inside_y) { return vec4f(0.0); }

    let dist = edge_dist(px, ex, ey, ew, eh);
    let pt   = perimeter_t(px, ex, ey, ew, eh);

    if dist > 7.0 * cs { return vec4f(0.0); }

    // Crackling ember wave — irregular, like smouldering cracks
    let crack_n = smooth_noise(vec2f(pt * 40.0, t * 1.5));
    let crack   = pow(crack_n, 3.0);

    // Slow pulsing heat
    let pulse = 0.6 + 0.4 * sin(t * 1.5 + pt * 6.28);

    // Ember colour: deep orange → dull red, with vine-green flickers
    let ember_core = palette.cinder_ember.rgb;
    let ember_edge = palette.cinder_gold.rgb;
    let vine_flick = palette.cinder_vine.rgb;
    var ember_rgb = mix(ember_edge, ember_core, crack * pulse);
    let vine_f = smoothstep(0.7, 0.9, smooth_noise(vec2f(pt * 20.0 + 5.0, t * 0.5)));
    ember_rgb = mix(ember_rgb, vine_flick, vine_f * 0.4);

    // Border glow profile
    let glow = smoothstep(0.0, 0.5 * cs, dist) * smoothstep(6.0 * cs, 1.0 * cs, dist);

    // Impact: bonfire flare
    let impact = prox * prox * 0.5;

    let brightness = (crack * 0.7 + pulse * 0.3 + impact) * prox;
    let final_rgb = ember_rgb * glow * brightness * 1.2;
    let final_a   = glow * brightness * 0.85;

    return vec4f(final_rgb, final_a);
}

// ---- main scene compositing -------------------------------------------------

fn sample_scene(px: vec2f) -> vec4f {
    let hover_idx = i32(u.hover_elem);
    let cs = u.cinder_size;

    // FAST PATH: no hover and cinder disabled → skip entire loop
    if hover_idx < 0 && cs <= 0.0 {
        return vec4f(0.0);
    }

    var out = vec4f(0.0);
    let count = u32(u.element_count);

    // In 3D views (scene3d=4, hypergraph=5), skip cinder border on structural
    // elements (kind=0) like the view-container — effects should only apply
    // to actual scene objects, not the outer viewport frame.
    let is_3d_view = u.current_view >= 4.0 && u.current_view <= 5.0;

    // When hovering, only process the hovered element (skip full loop)
    if hover_idx >= 0 && cs > 0.0 {
        let i = u32(hover_idx);
        if i < count {
            let e = elems[i];
            // Skip structural elements (kind=0) in 3D views
            if is_3d_view && e.kind < 0.5 {
                return vec4f(0.0);
            }
            let prox = hover_proximity(e.rect.x, e.rect.y, e.rect.z, e.rect.w);
            let border = hover_border(px, e.rect.x, e.rect.y, e.rect.z, e.rect.w, e.hue, u.time, prox);
            out = out + border;
        }
        return out;
    }

    // Full loop only when cinder enabled but no hover (static shadows)
    for (var i = 0u; i < count; i++) {
        let e  = elems[i];
        // Skip structural elements in 3D views
        if is_3d_view && e.kind < 0.5 { continue; }
        let ex = e.rect.x;
        let ey = e.rect.y;
        let ew = e.rect.z;
        let eh = e.rect.w;

        let is_hovered = i32(i) == hover_idx;

        if is_hovered {
            let prox = hover_proximity(ex, ey, ew, eh);
            let border = hover_border(px, ex, ey, ew, eh, e.hue, u.time, prox);
            out = out + border;
        } else {
            let shadow = static_shadow(px, ex, ey, ew, eh);
            out = out + vec4f(0.0, 0.0, 0.0, shadow);
        }
    }

    return out;
}

// ---- 3D skybox helpers (triplanar mapping) ----------------------------------

// ---- Optimised smoke noise (Book of Shaders / domain warping) ---------------
// 2-octave fbm for cheap warping / palette blending
fn fbm_lo(p: vec2f) -> f32 {
    return smooth_noise(p) * 0.667 + smooth_noise(p * 2.0) * 0.333;
}

// 3-octave fbm with inter-octave rotation for richer patterns at lower cost.
// Rotation breaks axis-aligned grid artefacts, making 3 octaves look as good
// as 4 non-rotated ones.  (Inspired by Book of Shaders ch.13.)
fn smoke_noise(p_in: vec2f) -> f32 {
    var val = 0.0;
    var amp = 0.5;
    var p = p_in;
    for (var i = 0; i < 3; i++) {
        val += amp * smooth_noise(p);
        amp *= 0.5;
        // Rotate ≈37° + lacunarity 2.0 between octaves
        p = vec2f(p.x * 1.6 - p.y * 1.2, p.x * 1.2 + p.y * 1.6);
    }
    return val;
}

// Triplanar blend weights from ray direction - avoids pole singularities
// Returns weights for XY, XZ, YZ planes (sums to 1.0)
fn triplanar_weights(dir: vec3f) -> vec3f {
    let n = abs(dir);
    // Sharpen the blend with power to reduce visible transitions
    let weights = pow(n, vec3f(4.0));
    return weights / (weights.x + weights.y + weights.z);
}

// Sample 2D noise with triplanar blending - no pole artifacts
fn triplanar_noise(dir: vec3f, scale: f32, offset: vec2f) -> f32 {
    let w = triplanar_weights(dir);
    let d = dir * scale;
    let n_xy = smooth_noise(d.xy + offset);
    let n_xz = smooth_noise(d.xz + offset);
    let n_yz = smooth_noise(d.yz + offset);
    return n_xy * w.z + n_xz * w.y + n_yz * w.x;
}

// Cheap triplanar warp noise (2 octaves)
fn triplanar_fbm_lo(dir: vec3f, scale: f32, offset: vec2f) -> f32 {
    let w = triplanar_weights(dir);
    let d = dir * scale;
    return fbm_lo(d.xy + offset) * w.z +
           fbm_lo(d.xz + offset) * w.y +
           fbm_lo(d.yz + offset) * w.x;
}

// Triplanar domain-warped smoke (3 octaves, rotated)
fn triplanar_smoke(dir: vec3f, scale: f32, offset: vec2f) -> f32 {
    let w = triplanar_weights(dir);
    let d = dir * scale;
    return smoke_noise(d.xy + offset) * w.z +
           smoke_noise(d.xz + offset) * w.y +
           smoke_noise(d.yz + offset) * w.x;
}

// Compute world-space ray direction from screen pixel using inverse VP matrix
fn screen_to_ray_dir(px: vec2f, width: f32, height: f32) -> vec3f {
    // Convert pixel to NDC (clip space)
    let ndc = vec2f(
        (px.x / width) * 2.0 - 1.0,
        1.0 - (px.y / height) * 2.0  // flip Y for screen coords
    );
    
    // Unproject near and far points using inverse VP
    let near_clip = vec4f(ndc.x, ndc.y, -1.0, 1.0);
    let far_clip = vec4f(ndc.x, ndc.y, 1.0, 1.0);
    
    let near_world = u.particle_inv_vp * near_clip;
    let far_world = u.particle_inv_vp * far_clip;
    
    let near_pos = near_world.xyz / near_world.w;
    let far_pos = far_world.xyz / far_world.w;
    
    return normalize(far_pos - near_pos);
}

// ---- fragment entry point ---------------------------------------------------

@fragment
fn fs_main(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    let raw_px = pos.xy;
    let raw_uv = raw_px / vec2f(u.width, u.height);

    // --- Configurable smoke parameters from uniforms --------------------------
    let s_intensity  = u.smoke_intensity;       // 0.0–1.0
    let s_speed      = u.smoke_speed;           // 0.0–5.0
    let s_warm_scale = u.smoke_warm_scale;      // 0.0–2.0 (warm layers 1+4)
    let s_cool_scale = u.smoke_cool_scale;      // 0.0–2.0 (cool layer 2)
    let s_moss_scale = u.smoke_moss_scale;      // 0.0–2.0 (moss-tone layer)
    let s_grain_i    = u.grain_intensity;       // 0.0–1.0 brightness
    let s_grain_c    = mix(0.5, 2.0, u.grain_coarseness);  // freq scale 0.5–2.0
    let s_grain_sz   = mix(1.0, 8.0, u.grain_size);         // pixel block 1–8
    let s_vignette   = u.vignette_str;          // 0.0–1.0
    let s_underglow  = u.underglow_str;         // 0.0–1.0

    // Speed is baked into the base time so ALL time-dependent effects scale uniformly
    let t = u.time * 0.35 * s_speed;

    // --- Check if we're in a 3D view (scene3d=4, hypergraph=5) ----------------
    let is_3d_view = u.current_view >= 4.0 && u.current_view <= 5.0;

    var bg: vec3f;
    var vignette: f32;
    var ds_px: vec2f;
    var ds_uv: vec2f;
    var drift: vec2f;
    var grain_px: vec2f;

    // --- 3D SKYBOX PATH: triplanar projection (no pole artifacts) --------------
    if is_3d_view {
        // Compute ray direction from camera through this pixel
        let ray_dir = screen_to_ray_dir(raw_px, u.width, u.height);
        
        // Triplanar mapping: sample noise from XY, XZ, YZ planes and blend
        // This eliminates singularities at poles that spherical mapping has
        
        drift = vec2f(t * 0.12, t * 0.06);
        grain_px = floor(raw_px / 4.0) * 4.0;  // grain uses screen coords
        ds_px = raw_px;
        ds_uv = raw_uv;
        
        // Scale factor to match 2D appearance (ray_dir is normalized to length 1)
        let skybox_scale = max(u.width, u.height) * 0.5;
        
        // --- Vignette for 3D: gentle edge darkening --------------------------
        let vig_d = length((raw_uv - 0.5) * vec2f(1.0, 0.8));
        vignette = 1.0 - smoothstep(0.5, 1.2, vig_d) * 0.35 * s_vignette;
        
        // --- Base palette using triplanar noise (same logic as 2D) -----------
        let palette_t = triplanar_noise(ray_dir, skybox_scale * 0.003 * s_warm_scale, drift * 5.0);
        let cool_var  = triplanar_noise(ray_dir, skybox_scale * 0.005 * s_cool_scale, drift * 3.0 + vec2f(3.7, 2.1));
        let moss_var  = triplanar_noise(ray_dir, skybox_scale * 0.008 * s_moss_scale, drift * 4.0 + vec2f(5.1, 9.3));
        let cool_tone = palette.smoke_cool.rgb;
        let warm_tone = palette.smoke_warm.rgb;
        let moss_tone = palette.smoke_moss.rgb;
        var base_col = mix(cool_tone, warm_tone, smoothstep(0.3, 0.7, palette_t));
        base_col = mix(base_col, cool_tone, smoothstep(0.4, 0.7, cool_var) * 0.3);
        base_col = mix(base_col, moss_tone, smoothstep(0.3, 0.7, moss_var) * 0.5 * s_moss_scale);
        
        // Add grain (using screen coords for temporal stability)
        var grain_sum = 0.0;
        if (s_grain_i > 0.0) {
            let n_fine   = smooth_noise((grain_px + drift * 40.0) * 0.025 * s_grain_c) * 0.028 * s_grain_i;
            let n_coarse = smooth_noise((grain_px + drift * 20.0) * 0.006 * s_grain_c + vec2f(7.3, 2.1)) * 0.018 * s_grain_i;
            let n_grain  = hash2(grain_px * 0.37 * s_grain_c + vec2f(floor(u.time * 8.0 * s_speed))) * 0.015 * s_grain_i;
            grain_sum = n_fine + n_coarse + n_grain;
        }
        bg = base_col + vec3f(grain_sum);
        
        // --- Domain-warped smoke (Book of Shaders ch.13) ---------------------
        // Instead of 4 independent fbm layers, use 1 cheap warp + 2 warped
        // smoke layers.  Domain warping produces organic, swirling patterns
        // with fewer texture lookups.
        if (s_intensity > 0.0) {
            // Cheap warp field (2 octaves) — shifts sample coords for organic curl
            let warp = triplanar_fbm_lo(ray_dir, 2.0, vec2f(t * 0.04, t * 0.02));
            let warp_offset = vec3f(warp * 0.15, warp * 0.1, warp * 0.12);
            let warped_dir = normalize(ray_dir + warp_offset);

            // Layer 1: warm rolling smoke (3 oct, rotated)
            let smoke_warm = triplanar_smoke(warped_dir, 2.0 * s_warm_scale,
                             vec2f(t * 0.05, t * 0.02)) * 0.14 * s_intensity;

            // Layer 2: cool tendrils drifting opposite direction (3 oct, rotated)
            let smoke_cool = triplanar_smoke(warped_dir, 3.5 * s_cool_scale,
                             vec2f(-t * 0.07, t * 0.04)) * 0.09 * s_intensity;

            // Composite with colour tinting
            bg = bg + vec3f(smoke_warm) * vec3f(0.85, 0.80, 0.75);  // warm smoke
            bg = bg + vec3f(smoke_cool) * vec3f(0.6, 0.7, 0.85);    // cool wisps
            // Moss tinting from warp noise (free — reuses existing value)
            bg = bg + vec3f(warp * 0.04 * s_intensity * s_moss_scale) * palette.smoke_moss.rgb;
        }
        
        // Grain shimmer
        if (s_grain_i > 0.0) {
            let grain_hi = smooth_noise((grain_px + drift * 60.0) * 0.12 * s_grain_c) * 0.012 * s_grain_i;
            bg = bg + vec3f(grain_hi * 0.6, grain_hi * 0.55, grain_hi * 0.5);
        }
        
        // Underglow (based on screen position, not spherical)
        if (s_underglow > 0.001) {
            let underglow = smoothstep(1.0, 0.4, raw_uv.y) * 0.015 * s_underglow;
            bg = bg + vec3f(0.5, 0.18, 0.05) * underglow;
        }
        
        // Apply vignette
        bg = bg * vignette;
    } else {
        // --- 2D PATH: Original screen-space background -------------------------
        // Existing logic preserved for non-3D views
        
        // FAST PATH: minimal background when smoke+grain disabled
        let effects_minimal = s_intensity <= 0.0 && s_grain_i <= 0.0;

        if effects_minimal {
            // Simple UV-based gradient — no noise calls at all
            let cool_tone = palette.smoke_cool.rgb;
            let warm_tone = palette.smoke_warm.rgb;
            // Vertical gradient from cool (top) to warm (bottom)
            let grad_t = raw_uv.y * 0.5 + raw_uv.x * 0.2;
            bg = mix(cool_tone, warm_tone, grad_t);
            // Cheap vignette
            let vig_d = length((raw_uv - 0.5) * vec2f(1.2, 1.0));
            vignette = 1.0 - smoothstep(0.3, 1.1, vig_d) * 0.55 * s_vignette;
            ds_px = raw_px;
            ds_uv = raw_uv;
            drift = vec2f(0.0);
            grain_px = raw_px;
        } else {
            // --- Full quality path with noise effects ---------------------------------
            // Smoke uses full-resolution coordinates for smooth blending.
            // Only grain gets intentional pixel-block downsampling.
            ds_px = raw_px;
            ds_uv = raw_uv;

            // Vignette — darken edges, bright centre (scaled by vignette_str)
            let vig_d = length((raw_uv - 0.5) * vec2f(1.2, 1.0));
            vignette = 1.0 - smoothstep(0.3, 1.1, vig_d) * 0.55 * s_vignette;

            // Grain pixel-block downsampling (s_grain_sz controls block size)
            let grain_base = floor(raw_px / 4.0) * 4.0;
            grain_px = floor(grain_base / s_grain_sz) * s_grain_sz;

            // Animated coarse noise — drifting (speed already baked into t)
            drift = vec2f(t * 0.12, t * 0.06);
            var grain_sum = 0.0;
            if (s_grain_i > 0.0) {
                let n_fine   = smooth_noise((grain_px + drift * 40.0) * 0.025 * s_grain_c) * 0.028 * s_grain_i;
                let n_coarse = smooth_noise((grain_px + drift * 20.0) * 0.006 * s_grain_c + vec2f(7.3, 2.1)) * 0.018 * s_grain_i;
                let n_grain  = hash2(grain_px * 0.37 * s_grain_c + vec2f(floor(u.time * 8.0 * s_speed))) * 0.015 * s_grain_i;
                grain_sum = n_fine + n_coarse + n_grain;
            }

            // Varied base palette — colour variation across the screen.
            // s_warm_scale controls base warm/cool blending frequency;
            // s_cool_scale biases toward cool tones; s_moss_scale drives moss-tone blending.
            let palette_t = smooth_noise(ds_px * (0.003 * s_warm_scale) + drift * 5.0);
            let cool_var  = smooth_noise(ds_px * (0.005 * s_cool_scale) + drift * 3.0 + vec2f(3.7, 2.1));
            let moss_var  = smooth_noise(ds_px * (0.008 * s_moss_scale) + drift * 4.0 + vec2f(5.1, 9.3));
            let cool_tone = palette.smoke_cool.rgb;
            let warm_tone = palette.smoke_warm.rgb;
            let moss_tone = palette.smoke_moss.rgb;
            var base_col = mix(cool_tone, warm_tone, smoothstep(0.3, 0.7, palette_t));
            // Cool-scale noise pulls toward cool tones
            base_col = mix(base_col, cool_tone, smoothstep(0.4, 0.7, cool_var) * 0.3);
            // Moss-scale noise blends in the moss mid-tone (higher scale = more varied moss patches)
            base_col = mix(base_col, moss_tone, smoothstep(0.3, 0.7, moss_var) * 0.5 * s_moss_scale);
            bg = base_col + vec3f(grain_sum);
        }

        // --- Domain-warped smoke (Book of Shaders ch.13) ---------------------
        // 1 cheap warp + 2 warped smoke layers replaces 4 plain fbm layers.
        // Domain warping produces organic swirling with fewer lookups.
        if (s_intensity > 0.0 && !effects_minimal) {
            // Cheap warp field (2 octaves) — swirls the sample coordinates
            let warp = fbm_lo(ds_uv * 2.0 + vec2f(t * 0.04, t * 0.02));
            let warped_uv = ds_uv + vec2f(warp * 0.15, warp * 0.1);

            // Layer 1: warm rolling smoke (3 oct, rotated)
            let smoke_warm = smoke_noise(warped_uv * (2.0 * s_warm_scale)
                             + vec2f(t * 0.05, t * 0.02)) * 0.14 * s_intensity;

            // Layer 2: cool tendrils drifting opposite direction (3 oct, rotated)
            let smoke_cool = smoke_noise(warped_uv * (3.5 * s_cool_scale)
                             + vec2f(-t * 0.07, t * 0.04)) * 0.09 * s_intensity;

            // Composite with colour tinting
            bg = bg + vec3f(smoke_warm) * vec3f(0.85, 0.80, 0.75);  // warm smoke
            bg = bg + vec3f(smoke_cool) * vec3f(0.6, 0.7, 0.85);    // cool wisps
            // Moss tinting from warp noise (free — reuses existing value)
            bg = bg + vec3f(warp * 0.04 * s_intensity * s_moss_scale) * palette.smoke_moss.rgb;
        }

        // Faint animated grain shimmer (skip noise when grain disabled)
        if (s_grain_i > 0.0 && !effects_minimal) {
            let grain_hi = smooth_noise((grain_px + drift * 60.0) * 0.12 * s_grain_c) * 0.012 * s_grain_i;
            bg = bg + vec3f(grain_hi * 0.6, grain_hi * 0.55, grain_hi * 0.5);
        }

        // Dim warm underglow from bottom edge (skip when off)
        if (s_underglow > 0.001) {
            let underglow = smoothstep(1.0, 0.4, raw_uv.y) * 0.015 * s_underglow;
            bg = bg + vec3f(0.5, 0.18, 0.05) * underglow;
        }

        // Apply vignette
        bg = bg * vignette;
    }

    // --- Scene elements ------------------------------------------------------
    let scene = sample_scene(raw_px);
    var color = bg * (1.0 - scene.a) + scene.rgb;

    // --- Atmospheric CRT effects (independently controlled) ------------------
    let sh_i = u.crt_scanlines_h;  // horizontal scanlines
    let sv_i = u.crt_scanlines_v;  // vertical scanlines
    let es_i = u.crt_edge_shadow;  // edge/border shadow
    let fl_i = u.crt_flicker;      // torch flicker

    let any_crt = max(max(sh_i, sv_i), max(es_i, fl_i));
    if (any_crt > 0.001) {
        let lw = u.crt_line_width;  // 0 = thin, 1 = wide

        // Horizontal scanlines + horizontal component of pixel grid
        let scanline = mix(1.0, crt_scanlines(raw_px.y, lw), sh_i);
        // Vertical scanlines + vertical component of pixel grid
        let vline    = mix(1.0, crt_vertical_lines(raw_px.x, lw), sv_i);
        // Pixel grid: blend of both axes — only visible where both have intensity
        let grid_i = min(sh_i, sv_i);
        let pgrid  = mix(1.0, crt_pixel_grid(raw_px), grid_i);

        let edge     = mix(1.0, crt_edge_shadow(raw_uv), es_i);
        let torch_flicker = 1.0 - fl_i * (0.03 - 0.03 * sin(t * 3.0 + raw_uv.x * 2.0));

        // Apply multiplicative CRT darkening
        let crt_mult = scanline * vline * pgrid * edge * torch_flicker;
        color = color * crt_mult;

        // CRT color tint — blend toward scanline color in dark CRT bands
        let crt_tint = vec3f(u.crt_color_r, u.crt_color_g, u.crt_color_b);
        let tint_len = length(crt_tint);
        if (tint_len > 0.01) {
            // Tint strength based on how dark the CRT made this pixel
            let darkness = 1.0 - crt_mult;
            color = color + crt_tint * darkness * 0.3;
        }
    }

    return vec4f(clamp(color, vec3f(0.0), vec3f(1.0)), 1.0);
}
