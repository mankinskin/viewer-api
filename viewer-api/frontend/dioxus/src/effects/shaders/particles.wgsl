// particles.wgsl — multi-type instanced particle rendering
//
// Concatenated after: palette.wgsl + types.wgsl + noise.wgsl + particle-shading.wgsl
// Four particle types rendered in one instanced draw call:
//   PK_METAL_SPARK (0) — velocity-aligned thin streak (grinding spark)
//   PK_EMBER       (1) — tiny pixel-size warm ember/ash glow (continuous)
//   PK_GOD_RAY     (2) — pixel-thin tall vertical angelic beam (continuous)
//   PK_GLITTER     (3) — tiny angelic twinkle around selected element border
//
// Fragment colouring uses shared functions from particle-shading.wgsl
// (shade_spark_fx, shade_ember_fx, shade_beam_fx, shade_glitter_fx).

// ---- bindings (render pass — read-only) -------------------------------------

@group(0) @binding(0) var<uniform>       u         : Uniforms;
@group(0) @binding(1) var<storage, read> elems     : array<ElemRect>;
@group(0) @binding(2) var<storage, read> particles : array<Particle>;

// ---- interpolated data between VS and FS ------------------------------------

struct ParticleVarying {
    @builtin(position)                    clip_pos : vec4f,
    @location(0)                          local_uv : vec2f,   // [-1..1] in oriented quad space
    @location(1) @interpolate(flat)       pidx     : u32,
    @location(2) @interpolate(flat)       pkind    : u32,
    @location(3) @interpolate(flat)       aspect   : f32,     // elongation ratio
}

// ---- vertex shader ----------------------------------------------------------

@vertex
fn vs_particle(
    @builtin(vertex_index)   vid : u32,
    @builtin(instance_index) iid : u32,
) -> ParticleVarying {
    var out: ParticleVarying;
    out.pidx  = iid;
    out.pkind = 0u;
    out.aspect = 1.0;

    let p = particles[iid];
    let kind = u32(p.kind_view) % 8u;
    let view_id = (u32(p.kind_view) / 8u) % 8u;
    let is_screen_space = u32(p.kind_view) >= 64u;
    out.pkind = kind;

    // Dead or wrong view → degenerate quad (off-screen)
    if p.life <= 0.0 || f32(view_id) != u.current_view {
        out.clip_pos = vec4f(-2.0, -2.0, 0.0, 1.0);
        out.local_uv = vec2f(0.0);
        return out;
    }

    // 6-vertex quad (two triangles)
    var corner: vec2f;
    switch vid {
        case 0u: { corner = vec2f(-1.0, -1.0); }
        case 1u: { corner = vec2f( 1.0, -1.0); }
        case 2u: { corner = vec2f(-1.0,  1.0); }
        case 3u: { corner = vec2f( 1.0, -1.0); }
        case 4u: { corner = vec2f( 1.0,  1.0); }
        default: { corner = vec2f(-1.0,  1.0); }
    }
    out.local_uv = corner;

    // Project particle center to clip space.
    // Screen-space particles: orthographic (screen pixels → clip)
    // World-space particles: use particle_vp (3D or 2D viewProj)
    var clip_center: vec4f;
    if is_screen_space {
        // Orthographic: screen pixels → NDC → clip
        let ndc_x = p.pos.x / u.width * 2.0 - 1.0;
        let ndc_y = -(p.pos.y / u.height * 2.0 - 1.0);  // Y flip
        clip_center = vec4f(ndc_x, ndc_y, 0.0, 1.0);
    } else {
        clip_center = u.particle_vp * vec4f(p.pos, 1.0);
    }
    let cw = clip_center.w;

    // All offsets are computed in PIXEL space (Y down), then converted to clip
    // at the end via:  clip_x += pixel_x * 2/vp_w * w
    //                  clip_y -= pixel_y * 2/vp_h * w   (Y flip: pixel Y↓, NDC Y↑)
    var pixel_offset: vec2f;

    if kind == 0u {
        // ---- METAL SPARK: velocity-aligned thin streak ----
        let vel_len = length(p.vel);
        var pixel_fwd: vec2f;
        var speed_px: f32;

        if is_screen_space {
            // Screen-space: velocity is already in pixels/sec
            let vel2d = p.vel.xy;
            let vel2d_len = length(vel2d);
            // Screen Y is down, so use velocity direction directly
            pixel_fwd = select(vec2f(0.0, 1.0), vel2d / vel2d_len, vel2d_len > 0.0001);
            speed_px = vel2d_len;
        } else {
            // World-space: project velocity direction to screen
            let fwd_world = select(vec3f(0.0, -1.0, 0.0), p.vel / vel_len, vel_len > 0.0001);
            let clip_ahead = u.particle_vp * vec4f(p.pos + fwd_world * u.world_scale, 1.0);
            let ndc_c = clip_center.xy / cw;
            let ndc_a = clip_ahead.xy / clip_ahead.w;
            // Full-canvas NDC → pixel direction (Y flipped)
            let pd = vec2f((ndc_a.x - ndc_c.x) * u.width * 0.5,
                           -(ndc_a.y - ndc_c.y) * u.height * 0.5);
            let pd_len = length(pd);
            pixel_fwd = select(vec2f(0.0, -1.0), pd / pd_len, pd_len > 0.0001);
            speed_px = vel_len / max(u.world_scale, 0.0001);
        }
        let pixel_right = vec2f(-pixel_fwd.y, pixel_fwd.x);

        // Length scales with on-screen speed; width stays thin
        let half_len = p.size * (2.0 + speed_px * 0.04);
        let half_wid = p.size * 0.35;
        out.aspect = half_len / max(half_wid, 0.1);
        pixel_offset = pixel_fwd * corner.y * half_len + pixel_right * corner.x * half_wid;

    } else if kind == 1u {
        // ---- EMBER / ASH: tiny pixel-size dot ----
        let radius = p.size * 1.2;
        pixel_offset = corner * radius;

    } else if kind == 2u {
        // ---- ANGELIC BEAM: tall line oriented along world up ----
        var pixel_up: vec2f;
        var px_per_world: f32;

        if is_screen_space {
            // Screen-space: up is negative Y (toward top of screen)
            pixel_up = vec2f(0.0, -1.0);
            px_per_world = 1.0;  // p.size is already in pixels
        } else {
            // World-space: project direction to screen
            // p.size is in world units (set at spawn: pixel_size × ws).
            // We project a 1-world-unit offset to find how many pixels one
            // world unit covers on screen, then scale p.size by that.
            let up_w = select(vec3f(0.0, -1.0, 0.0), vec3f(0.0, 1.0, 0.0), u.current_view >= 4.0 && u.current_view <= 5.0);

            // Project p.pos + 1 world unit in the up direction
            let clip_up_pt = u.particle_vp * vec4f(p.pos + up_w, 1.0);
            let ndc_c = clip_center.xy / cw;
            let ndc_u = clip_up_pt.xy / clip_up_pt.w;
            // Full-canvas NDC → pixel direction (Y flipped)
            let pd = vec2f((ndc_u.x - ndc_c.x) * u.width * 0.5,
                           -(ndc_u.y - ndc_c.y) * u.height * 0.5);
            let pd_len = length(pd);
            pixel_up = select(vec2f(0.0, -1.0), pd / pd_len, pd_len > 0.0001);
            // px_per_world: how many screen pixels 1 world unit covers
            px_per_world = pd_len;
        }
        let pixel_rt = vec2f(-pixel_up.y, pixel_up.x);

        // World-space half-extents → pixel sizes via px_per_world
        let half_w = p.size * 2.0 * px_per_world;
        let bh = select(35.0, u.beam_height, u.beam_height > 0.0);
        let half_h = p.size * bh * px_per_world;
        out.aspect = half_h / max(half_w, 0.1);
        // Beam extends upward from spawn point: base at pos, top at pos + 2×half_h
        pixel_offset = pixel_rt * corner.x * half_w + pixel_up * (corner.y * half_h + half_h);

    } else {
        // ---- GLITTER: slightly larger sparkle ----
        let radius = p.size * 1.8;
        pixel_offset = corner * radius;
    }

    // Convert pixel offset to clip-space offset (perspective-correct billboard).
    // particle_vp outputs full-canvas clip coords, so use full canvas dims.
    // Pixel Y increases downward, NDC Y increases upward → negate Y.
    out.clip_pos = clip_center + vec4f(
        pixel_offset.x *  2.0 / u.width * cw,
        pixel_offset.y * -2.0 / u.height * cw,
        0.0, 0.0
    );

    return out;
}

// ---- fragment shader (delegates to shared shade functions) -------------------

@fragment
fn fs_particle(in: ParticleVarying) -> @location(0) vec4f {
    let p = particles[in.pidx];
    let t_life = p.life / p.max_life;
    let kind = in.pkind;

    // Compute fwidth in uniform control flow (before branching on kind)
    let fw = fwidth(in.local_uv.x);

    if kind == 0u {
        return shade_spark_fx(in.local_uv, t_life, p.hue);
    } else if kind == 1u {
        return shade_ember_fx(in.local_uv, t_life, p.hue);
    } else if kind == 2u {
        return shade_beam_fx(in.local_uv, t_life, fw * 2.0);
    } else {
        return shade_glitter_fx(in.local_uv, t_life, p.hue, u.time, f32(in.pidx));
    }
}
