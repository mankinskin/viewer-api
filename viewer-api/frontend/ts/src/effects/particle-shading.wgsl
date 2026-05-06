// particle-shading.wgsl — Shared particle fragment shading functions
//
// Provides reusable colour computation for all four particle types:
//   - Metal sparks (cursor hover)
//   - Embers / ash (rising from borders)
//   - Angelic beams (double-pointed crystalline shards)
//   - Glitter (twinkling sparkles)
//
// Prerequisites:
//   - ThemePalette struct must be defined (from palette.wgsl)
//   - `palette` must be declared as a var<uniform> in the including shader
//
// These functions take plain parameters (UV, life, hue, time) so they
// work across both 2D overlay and 3D scene shader architectures.

// ── Cinder palette cycling (palette-driven) ─────────────────────────────────

fn cinder_rgb(t: f32) -> vec3f {
    let ember = palette.cinder_ember.rgb;
    let gold  = palette.cinder_gold.rgb;
    let ash   = palette.cinder_ash.rgb;
    let vine  = palette.cinder_vine.rgb;
    let s = fract(t);
    if s < 0.25 { return mix(ember, gold, s * 4.0); }
    if s < 0.50 { return mix(gold, ash, (s - 0.25) * 4.0); }
    if s < 0.75 { return mix(ash, vine, (s - 0.50) * 4.0); }
    return mix(vine, ember, (s - 0.75) * 4.0);
}

// ── Kind-aware glow colors (palette-driven) ─────────────────────────────────

fn kind_ember(kind: u32) -> vec3f {
    if kind == 1u { return palette.kind_error.rgb; }
    if kind == 2u { return palette.kind_warn.rgb; }
    if kind == 3u { return palette.kind_info.rgb; }
    if kind == 4u { return palette.kind_debug.rgb; }
    if kind == 5u { return palette.kind_span.rgb; }
    if kind == 6u { return palette.kind_selected.rgb; }
    if kind == 7u { return palette.kind_panic.rgb; }
    return palette.kind_structural.rgb;
}

// ── Metal spark fragment ────────────────────────────────────────────────────

fn shade_spark_fx(uv: vec2f, t_life: f32, hue: f32) -> vec4f {
    // Streak shape: tapered along length (Y), thin across width (X)
    let ax = abs(uv.x);              // cross-axis (thin)
    let ay = uv.y;                   // along-axis (long), -1 = tail, +1 = head

    // Width falloff — very thin hot wire
    let width_mask = exp(-ax * ax * 12.0);

    // Length taper — bright hot head, fading tail
    let head_t = (ay + 1.0) * 0.5;   // 0 at tail, 1 at head
    let len_mask = smoothstep(0.0, 0.15, head_t) * smoothstep(1.0, 0.7, head_t);

    let bright = t_life * width_mask * len_mask * 2.2;
    if bright < 0.005 { discard; }

    // Colour: white-hot core at head, orange-red ember at tail
    let hot_core = palette.spark_core.rgb;
    let steel    = palette.spark_steel.rgb;
    let ember    = cinder_rgb(hue);
    let core_t   = head_t * width_mask;                        // hot at head centre
    var spark_col = mix(ember * 1.5, hot_core * 1.2, core_t);
    spark_col = mix(spark_col, steel, (1.0 - head_t) * 0.4);  // steel tint at tail

    let col = spark_col * bright;
    let a   = min(bright, 1.0);
    return vec4f(col * a, a);
}

// ── Ember / ash fragment ────────────────────────────────────────────────────

fn shade_ember_fx(uv: vec2f, t_life: f32, hue: f32) -> vec4f {
    let d = length(uv);
    let glow = exp(-d * d * 2.5);
    let bright = t_life * glow;

    if bright < 0.005 { discard; }

    let ember_col = cinder_rgb(hue);
    let hot = palette.ember_hot.rgb;
    let base = mix(ember_col * 1.5, hot, smoothstep(0.5, 0.0, d));

    let col = base * bright;
    let a   = min(bright * 0.7, 1.0);
    return vec4f(col * a, a);
}

// ── Angelic beam (double-pointed crystalline shard) ─────────────────────────

fn shade_beam_fx(uv: vec2f, t_life: f32, aa: f32) -> vec4f {
    let dx = uv.x;
    let dy = uv.y;

    let t   = (dy + 1.0) * 0.5;
    let mid = abs(t - 0.5) * 2.0;
    let shard_width = (1.0 - mid * mid) * 0.18;

    let hx = abs(dx) / max(shard_width, 0.005);
    let edge = smoothstep(1.0 + aa, 1.0 - aa, hx);

    let core = exp(-dx * dx / max(shard_width * shard_width * 0.1, 0.0005));
    let h_falloff = edge * (0.25 + 0.75 * core);

    let v_fade = (1.0 - mid * mid);
    let bright = h_falloff * v_fade * t_life * 1.6;

    if bright < 0.003 { discard; }

    let center_col = palette.beam_center.rgb;
    let edge_col   = palette.beam_edge.rgb;
    var ray_col = mix(edge_col, center_col, core * 0.95);

    let col = ray_col * bright * 0.6;
    let a   = min(bright * 0.4, 1.0);
    return vec4f(col * a, a);
}

// ── Glitter (pixel-size twinkle) ────────────────────────────────────────────

fn shade_glitter_fx(uv: vec2f, t_life: f32, hue: f32, time: f32, id: f32) -> vec4f {
    let d = length(uv);
    let dot_mask = smoothstep(1.0, 0.15, d);

    let twinkle = 0.6 + 0.4 * sin(time * 12.0 + id * 7.3);
    let bright = t_life * dot_mask * twinkle * 1.4;

    if bright < 0.008 { discard; }

    let warm = palette.glitter_warm.rgb;
    let cool = palette.glitter_cool.rgb;
    let phase = fract(hue * 3.7 + time * 0.5);
    let glitter_col = mix(warm, cool, smoothstep(0.3, 0.7, phase));

    let col = glitter_col * bright;
    let a   = min(bright * 0.9, 1.0);
    return vec4f(col * a, a);
}
