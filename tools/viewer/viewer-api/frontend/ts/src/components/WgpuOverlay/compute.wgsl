// compute.wgsl — multi-effect particle physics simulation
//
// Concatenated after: types.wgsl + noise.wgsl
// Four particle types, partitioned by index:
//   [0, SPARK_END)               — metal sparks (at mouse position while hovering)
//   [SPARK_END, EMBER_END)       — flying embers / ash (continuous rise from hovered)
//   [EMBER_END, RAY_END)         — angelic beams (pixel-thin vertical from hovered)
//   [RAY_END, GLITTER_END)       — angelic glitter (around selected element)

// ---- bindings (compute pass) ------------------------------------------------

@group(0) @binding(0) var<uniform>             u         : Uniforms;
@group(0) @binding(1) var<storage, read>       elems     : array<ElemRect>;
@group(0) @binding(2) var<storage, read_write> particles : array<Particle>;

// ---- constants --------------------------------------------------------------

const BURST_WINDOW : f32 = 0.25;   // seconds — initial intense burst

// ---- helpers ----------------------------------------------------------------

// True when the active view is a real 3D scene (scene3d=4, hypergraph=5).
// Other views (logs=0..debug=3, settings=6) use screen-space 2D.
fn is_3d_view() -> bool {
    return u.current_view >= 4.0 && u.current_view <= 5.0;
}

// True when an element is positioned in screen-space (not 3D world space).
// Elements with depth ≈ 0 are static UI / containers that should not move
// when the camera moves in a 3D viewport.
fn is_elem_screen_space(ei: u32) -> bool {
    return elems[ei].depth < 0.0001;
}

// World-space "up" direction for a particle's coordinate space.
// 2D screen-space: (0, -1, 0) because screen Y increases downward.
// 3D world-space: (0, +1, 0) because Y is up.
fn particle_up(is_screen: bool) -> vec3f {
    return select(vec3f(0.0, 1.0, 0.0), vec3f(0.0, -1.0, 0.0), is_screen);
}

// World-space "right" direction (always +X).
fn particle_right() -> vec3f {
    return vec3f(1.0, 0.0, 0.0);
}

// World-space "up" direction based on current view (legacy helper).
// 2D views: (0, -1, 0) because screen Y increases downward.
// 3D views: (0, +1, 0) because Y is up in world space.
fn world_up() -> vec3f {
    return select(vec3f(0.0, -1.0, 0.0), vec3f(0.0, 1.0, 0.0), is_3d_view());
}

// World-space "right" direction (always +X).
fn world_right() -> vec3f {
    return vec3f(1.0, 0.0, 0.0);
}

// Convert a screen-pixel position to world space using the inverse viewProj.
// For 2D views (ortho), world ≈ screen pixels with z=0.
// For 3D views, unprojecting at the given NDC depth gives a world position
// on the corresponding depth plane.
fn screen_to_world(screen_pos: vec2f) -> vec3f {
    return screen_to_world_at(screen_pos, u.ref_depth);
}

// Like screen_to_world but with an explicit NDC depth parameter.
// Converts screen-pixel coordinates to full-canvas NDC, then unprojections
// via particle_inv_vp (which maps full-canvas NDC → world space).
fn screen_to_world_at(screen_pos: vec2f, ndc_z: f32) -> vec3f {
    let ndc_x = screen_pos.x / u.width * 2.0 - 1.0;
    let ndc_y = -(screen_pos.y / u.height * 2.0 - 1.0);
    let clip = vec4f(ndc_x, ndc_y, ndc_z, 1.0);
    let world4 = u.particle_inv_vp * clip;
    return world4.xyz / world4.w;
}

// Resolve per-element depth: use the element's own NDC depth if set (>0),
// otherwise fall back to the global reference depth (camera target plane).
fn elem_depth(ei: u32) -> f32 {
    let d = elems[ei].depth;
    return select(u.ref_depth, d, d > 0.0001);
}

// Spawn a random point on the perimeter of an element's screen-space rect.
// Returns SCREEN-PIXEL coordinates (call screen_to_world to convert).
fn spawn_on_perimeter(elem_idx: u32, seed: u32) -> vec2f {
    let e  = elems[elem_idx];
    let ex = e.rect.x;
    let ey = e.rect.y;
    let ew = e.rect.z;
    let eh = e.rect.w;
    let perim = 2.0 * (ew + eh);
    let t = rand_f(seed) * perim;

    if t < ew        { return vec2f(ex + t, ey); }
    if t < ew + eh   { return vec2f(ex + ew, ey + (t - ew)); }
    if t < 2.0*ew+eh { return vec2f(ex + ew - (t - ew - eh), ey + eh); }
    return vec2f(ex, ey + eh - (t - 2.0 * ew - eh));
}

fn outward_normal(pos: vec2f, elem_idx: u32) -> vec2f {
    let e  = elems[elem_idx];
    let cx = e.rect.x + e.rect.z * 0.5;
    let cy = e.rect.y + e.rect.w * 0.5;
    return normalize(pos - vec2f(cx, cy) + vec2f(0.001, 0.001));
}

// Convert a 2D screen-space outward normal to a world-space direction.
// For 2D: maps directly (x, y, 0). For 3D: uses the projection to
// find the world-space direction corresponding to a screen offset.
fn normal_to_world(n: vec2f, center_world: vec3f, center_screen: vec2f) -> vec3f {
    let offset_world = screen_to_world(center_screen + n * 10.0);
    let d = offset_world - center_world;
    let len = length(d);
    if len < 0.0001 { return vec3f(n.x, n.y, 0.0); }
    return d / len;
}

fn park_dead(idx: u32) {
    var p = particles[idx];
    p.pos  = vec3f(-9999.0);
    p.vel  = vec3f(0.0);
    p.life = 0.0;
    p.size = 0.0;
    particles[idx] = p;
}

// Returns the kind of the currently hovered element, or -1.0 if none.
fn hovered_elem_kind() -> f32 {
    let hi = i32(u.hover_elem);
    if hi < 0 || hi >= i32(u.element_count) { return -1.0; }
    return elems[u32(hi)].kind;
}

// Returns true if this hover-based effect should spawn on the hovered element.
// Regular elements (kind < 8) allow all effects.
// Preview containers (kind 8-11) only allow their matching effect.
// In 3D views (scene3d=4, hypergraph=5), skip structural elements (kind=0)
// like the view-container — effects should only apply to scene objects.
fn hover_allows(fx_kind: f32) -> bool {
    let hk = hovered_elem_kind();
    if hk < 0.0 { return false; }       // nothing hovered
    // Skip structural elements in 3D views
    let is_3d_view = u.current_view >= 4.0 && u.current_view <= 5.0;
    if is_3d_view && hk < 0.5 { return false; }
    if hk < 7.5 { return true; }        // regular element — all effects
    return abs(hk - fx_kind) < 0.5;     // preview — must match
}

// Decode particle kind from packed kind_view field.
fn p_kind(p: Particle) -> u32 { return u32(p.kind_view) % 8u; }
// Decode view_id from packed kind_view field.
fn p_view(p: Particle) -> u32 { return (u32(p.kind_view) / 8u) % 8u; }
// Check if particle is screen-space (not 3D world-space).
fn p_is_screen_space(p: Particle) -> bool { return u32(p.kind_view) >= 64u; }

// ---- metal spark physics (at mouse cursor, continuous while hovering) --------

fn update_metal_spark(idx: u32) {
    // Speed == 0 means sparks are disabled — buffer already zeroed by CPU
    if u.spark_speed <= 0.0 { return; }

    var p  = particles[idx];
    let dt = u.delta_time;
    let hover_idx = i32(u.hover_elem);
    let spd = u.spark_speed;
    let ws  = u.world_scale;  // world units per screen pixel

    // Respect spark count limit — park excess sparks
    let spark_frac = select(1.0, u.spark_count, u.spark_count > 0.0);
    let max_sparks = u32(f32(SPARK_END) * clamp(spark_frac, 0.0, 2.0));
    if idx >= max_sparks {
        park_dead(idx);
        return;
    }

    p.life -= dt * spd;

    if p.life <= 0.0 {
        if hover_idx < 0 || hover_idx >= i32(u.element_count)
           || !hover_allows(KIND_FX_SPARK) {
            park_dead(idx);
            return;
        }

        let ei = u32(hover_idx);
        let is_screen = is_elem_screen_space(ei);
        let seed = idx * 7919u + u32(u.time * 5000.0);

        // Spawn at mouse cursor
        let base_angle = rand_f(seed) * 6.2832;
        let scatter_r  = 5.0 + rand_f(seed + 1u) * 35.0;
        let scatter = vec2f(cos(base_angle), sin(base_angle)) * scatter_r;
        let cursor_screen = vec2f(u.mouse_x, u.mouse_y);
        let spawn_screen  = cursor_screen + scatter;

        let spread = (rand_f(seed + 2u) - 0.5) * 0.87;  // ±25°
        let spread_angle = base_angle + spread;
        let screen_dir = vec2f(cos(spread_angle), sin(spread_angle));

        if is_screen {
            // Screen-space: position IS screen pixels
            p.pos = vec3f(spawn_screen, 0.0);
            // Velocity in screen pixels per second
            let since_hover = u.time - u.hover_start_time;
            let burst_mult = select(0.5, 1.2, since_hover < BURST_WINDOW);
            let speed = (40.0 + rand_f(seed + 3u) * 100.0) * burst_mult * spd;
            p.vel = vec3f(screen_dir * speed, 0.0);
        } else {
            // World-space: unproject to world
            p.pos = screen_to_world(spawn_screen);
            let cursor_world = screen_to_world(cursor_screen);
            let vel_dir = normal_to_world(screen_dir, cursor_world, cursor_screen);
            let since_hover = u.time - u.hover_start_time;
            let burst_mult = select(0.5, 1.2, since_hover < BURST_WINDOW);
            let speed = (40.0 + rand_f(seed + 3u) * 100.0) * burst_mult * spd * ws;
            p.vel = vel_dir * speed;
        }

        p.max_life = 0.5 + rand_f(seed + 5u) * 0.8;
        p.life     = p.max_life;
        p.hue      = rand_f(seed + 6u) * 0.12;
        p.size     = (1.0 + rand_f(seed + 7u) * 2.0) * max(u.spark_size, 0.01);
        p.kind_view = PK_METAL_SPARK + u.current_view * 8.0 + select(0.0, PK_SCREEN_SPACE, is_screen);
        p.spawn_t  = u.time;
    } else {
        let is_screen = p_is_screen_space(p);
        // Moderate drag — particles trail behind with gravity
        p.vel = p.vel * (1.0 - 2.0 * dt * spd);
        // Gravity pulls "down" — screen Y positive, world Y negative
        let grav_scale = select(ws, 1.0, is_screen);
        p.vel = p.vel + particle_up(is_screen) * -80.0 * dt * spd * grav_scale;
        p.pos = p.pos + p.vel * dt * spd;
    }

    particles[idx] = p;
}

// ---- ember / ash physics (continuous rising embers) -------------------------

fn update_ember(idx: u32) {
    // Speed == 0 means embers are disabled — buffer already zeroed by CPU
    if u.ember_speed <= 0.0 { return; }

    var p  = particles[idx];
    let dt = u.delta_time;
    let hover_idx = i32(u.hover_elem);
    let spd = u.ember_speed;
    let ws  = u.world_scale;

    // Respect ember count limit
    let ember_frac = select(1.0, u.ember_count, u.ember_count > 0.0);
    let max_embers = u32(f32(EMBER_END - SPARK_END) * clamp(ember_frac, 0.0, 2.0));
    if (idx - SPARK_END) >= max_embers {
        park_dead(idx);
        return;
    }

    p.life -= dt * spd;

    if p.life <= 0.0 {
        if hover_idx < 0 || hover_idx >= i32(u.element_count)
           || !hover_allows(KIND_FX_EMBER) {
            park_dead(idx);
            return;
        }

        let ei   = u32(hover_idx);
        let is_screen = is_elem_screen_space(ei);
        let seed = idx * 7919u + u32(u.time * 1000.0);

        let screen_pos = spawn_on_perimeter(ei, seed);
        let normal = outward_normal(screen_pos, ei);
        let speed  = 10.0 + rand_f(seed + 3u) * 25.0;
        let up_speed = 20.0 + rand_f(seed + 4u) * 15.0;

        if is_screen {
            // Screen-space: position IS pixels, velocity in pixels/sec
            p.pos = vec3f(screen_pos, 0.0);
            // Rise upward (screen Y negative) + outward drift
            let up = particle_up(true);
            let vel_screen = vec3f(normal * speed * 0.5, 0.0) + up * up_speed;
            p.vel = vel_screen * spd;
        } else {
            // World-space: unproject position and directions
            p.pos = screen_to_world_at(screen_pos, elem_depth(ei));
            let e_center = vec2f(elems[ei].rect.x + elems[ei].rect.z * 0.5,
                                 elems[ei].rect.y + elems[ei].rect.w * 0.5);
            let n_world = normal_to_world(normal, screen_to_world_at(e_center, elem_depth(ei)), e_center);
            p.vel = (n_world * speed * 0.5 + world_up() * up_speed) * spd * ws;
        }

        p.max_life = 1.0 + rand_f(seed + 5u) * 1.5;
        p.life     = p.max_life;

        let r = rand_f(seed + 6u);
        if r < 0.80 {
            p.hue = rand_f(seed + 8u) * 0.12;
        } else {
            p.hue = 0.25 + rand_f(seed + 8u) * 0.15;
        }

        p.size     = (0.4 + rand_f(seed + 7u) * 1.0) * max(u.ember_size, 0.01);
        p.kind_view = PK_EMBER + u.current_view * 8.0 + select(0.0, PK_SCREEN_SPACE, is_screen);
        p.spawn_t  = u.time;
    } else {
        let is_screen = p_is_screen_space(p);
        let scale = select(ws, 1.0, is_screen);
        let drift = sin(u.time * 2.0 + f32(idx) * 0.3) * 8.0 * scale;
        // Drift sideways + continue rising in appropriate space
        let up = particle_up(is_screen);
        let rt = particle_right();
        p.vel = p.vel * (1.0 - 1.5 * dt * spd) + rt * drift * dt * spd + up * 25.0 * dt * spd * scale;
        p.pos = p.pos + p.vel * dt * spd;
    }

    particles[idx] = p;
}

// ---- angelic beam physics (pixel-thin vertical rays from selected/opened) ---

fn update_god_ray(idx: u32) {
    // Speed == 0 means beams are disabled — buffer already zeroed by CPU
    if u.beam_speed <= 0.0 { return; }

    var p  = particles[idx];
    let dt = u.delta_time;
    let spd = u.beam_speed;
    let ws  = u.world_scale;

    // Beam source: selected element, hovered beam-preview, or any hovered element
    // (allows beams on hover in 3D views like hypergraph)
    var beam_src = i32(u.selected_elem);
    let hover_idx = i32(u.hover_elem);
    let is_3d_view = u.current_view >= 4.0 && u.current_view <= 5.0;
    if hover_idx >= 0 && hover_idx < i32(u.element_count) {
        let hk = elems[u32(hover_idx)].kind;
        if abs(hk - KIND_FX_BEAM) < 0.5 {
            // Beam-preview container always wins
            beam_src = hover_idx;
        } else if beam_src < 0 {
            // No selected element → fall back to hovered element
            // But skip structural elements (kind=0) in 3D views — these are
            // containers like view-container/hypergraph-container, not scene objects
            let is_structural = hk < 0.5;
            if !is_3d_view || !is_structural {
                beam_src = hover_idx;
            }
        }
    }

    // Respect beam count limit — park excess beams
    let max_beams = u32(u.beam_count);
    if max_beams > 0u && (idx - EMBER_END) >= max_beams {
        park_dead(idx);
        return;
    }

    p.life -= dt * spd;

    if p.life <= 0.0 {
        if beam_src < 0 || beam_src >= i32(u.element_count) {
            park_dead(idx);
            return;
        }

        let ei   = u32(beam_src);
        let is_screen = is_elem_screen_space(ei);
        let seed = idx * 7919u + u32(u.time * 800.0);

        let screen_pos = spawn_on_perimeter(ei, seed);
        let drift_scale = select(1.0, u.beam_drift, u.beam_drift > 0.0);
        let up_speed = (12.0 + rand_f(seed + 3u) * 10.0) * drift_scale;
        let side_drift = (rand_f(seed + 2u) - 0.5) * 2.0;

        if is_screen {
            // Screen-space: position IS pixels, velocity in pixels/sec
            p.pos = vec3f(screen_pos, 0.0);
            let up = particle_up(true);
            let rt = particle_right();
            p.vel = (rt * side_drift + up * up_speed) * spd;
            // Size in pixels
            p.size = 0.6 + rand_f(seed + 6u) * 1.0;
        } else {
            // World-space: unproject position
            p.pos = screen_to_world_at(screen_pos, elem_depth(ei));
            let up = world_up();
            let rt = world_right();
            p.vel = (rt * side_drift + up * up_speed) * spd * ws;
            // Size in WORLD units (pixel-equivalent × ws)
            p.size = (0.6 + rand_f(seed + 6u) * 1.0) * ws;
        }

        p.max_life = 2.0 + rand_f(seed + 4u) * 2.0;
        p.life     = p.max_life;
        p.hue      = 0.08 + rand_f(seed + 5u) * 0.06;
        p.kind_view = PK_GOD_RAY + u.current_view * 8.0 + select(0.0, PK_SCREEN_SPACE, is_screen);
        p.spawn_t  = u.time;
    } else {
        let is_screen = p_is_screen_space(p);
        let scale = select(ws, 1.0, is_screen);
        let sway = sin(u.time * 1.5 + f32(idx) * 0.7) * 1.5 * scale;
        // Sway sideways, drift upward
        let rt = particle_right();
        p.vel = p.vel * (1.0 - 0.5 * dt * spd) + rt * sway * dt * spd;
        // Dampen along up direction
        let up = particle_up(is_screen);
        let vel_up = dot(p.vel, up);
        p.vel = p.vel - up * vel_up * 0.2 * dt * spd;
        p.pos = p.pos + p.vel * dt * spd;
    }

    particles[idx] = p;
}

// ---- angelic glitter physics (around hovered element border) ----------------

fn update_glitter(idx: u32) {
    // Speed == 0 means glitter is disabled — buffer already zeroed by CPU
    if u.glitter_speed <= 0.0 { return; }

    var p  = particles[idx];
    let dt = u.delta_time;
    let hover_idx = i32(u.hover_elem);
    let spd = u.glitter_speed;
    let ws  = u.world_scale;

    // Respect glitter count limit
    let glitter_frac = select(1.0, u.glitter_count, u.glitter_count > 0.0);
    let max_glitter = u32(f32(GLITTER_END - RAY_END) * clamp(glitter_frac, 0.0, 2.0));
    if (idx - RAY_END) >= max_glitter {
        park_dead(idx);
        return;
    }

    p.life -= dt * spd;

    if p.life <= 0.0 {
        if hover_idx < 0 || hover_idx >= i32(u.element_count)
           || !hover_allows(KIND_FX_GLITTER) {
            park_dead(idx);
            return;
        }

        let ei   = u32(hover_idx);
        let seed = idx * 7919u + u32(u.time * 1200.0);
        let is_screen = is_elem_screen_space(ei);

        // Spawn on the hovered element perimeter
        let screen_pos = spawn_on_perimeter(ei, seed);

        // Position: screen-space elements store screen pixels directly,
        // world-space elements go through inverse viewProj
        if is_screen {
            p.pos = vec3f(screen_pos, 0.0);
        } else {
            p.pos = screen_to_world_at(screen_pos, elem_depth(ei));
        }

        // Velocity: tangential drift along border
        let norm    = outward_normal(screen_pos, ei);
        let tangent = vec2f(-norm.y, norm.x);
        let tang_dir = select(-1.0, 1.0, rand_f(seed + 2u) > 0.5);

        if is_screen {
            // Screen-space: velocity in pixel units (no ws scaling)
            let vel_2d = tangent * tang_dir * (4.0 + rand_f(seed + 3u) * 10.0)
                       + norm * (0.5 + rand_f(seed + 4u) * 2.5);
            p.vel = vec3f(vel_2d * spd, 0.0);
        } else {
            // World-space: convert 2D screen directions to world directions
            let e_center = vec2f(elems[ei].rect.x + elems[ei].rect.z * 0.5,
                                 elems[ei].rect.y + elems[ei].rect.w * 0.5);
            let center_world = screen_to_world_at(e_center, elem_depth(ei));
            let tang_world = normal_to_world(tangent, center_world, e_center);
            let norm_world = normal_to_world(norm, center_world, e_center);
            p.vel = (tang_world * tang_dir * (4.0 + rand_f(seed + 3u) * 10.0)
                  + norm_world * (0.5 + rand_f(seed + 4u) * 2.5)) * spd * ws;
        }

        p.max_life = 0.8 + rand_f(seed + 5u) * 1.5;
        p.life     = p.max_life;
        p.hue      = rand_f(seed + 6u);
        p.size     = (0.6 + rand_f(seed + 7u) * 1.2) * max(u.glitter_size, 0.01);
        // Pack: kind + view_id * 8 + (is_screen ? 64 : 0)
        p.kind_view = PK_GLITTER + u.current_view * 8.0 + select(0.0, PK_SCREEN_SPACE, is_screen);
        p.spawn_t  = u.time;
    } else {
        // Update: check if this particle is screen-space
        let is_screen = p_is_screen_space(p);
        let up = particle_up(is_screen);
        let rt = particle_right();

        if is_screen {
            // Screen-space: velocity in pixels, no ws scaling
            let sway = sin(u.time * 4.0 + f32(idx) * 1.3) * 4.0;
            p.vel = p.vel * (1.0 - 3.0 * dt * spd) + rt * sway * dt * spd + up * 1.5 * dt * spd;
        } else {
            // World-space: velocity scaled by ws
            let sway = sin(u.time * 4.0 + f32(idx) * 1.3) * 4.0 * ws;
            p.vel = p.vel * (1.0 - 3.0 * dt * spd) + rt * sway * dt * spd + up * 1.5 * dt * spd * ws;
        }
        p.pos = p.pos + p.vel * dt * spd;
    }

    particles[idx] = p;
}

// ---- compute entry point ----------------------------------------------------

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) gid: vec3u) {
    let idx   = gid.x;
    let total = arrayLength(&particles);
    if idx >= total { return; }

    // Shift live particles by scroll delta (2D views only).
    // In 2D, "world space" = screen pixels, so scroll delta is a direct
    // world-space offset. For 3D views, scroll delta is always 0.
    if !is_3d_view() {
        let sd = vec2f(u.scroll_dx, u.scroll_dy);
        if sd.x != 0.0 || sd.y != 0.0 {
            var p = particles[idx];
            if p.life > 0.0 {
                p.pos = p.pos + vec3f(sd, 0.0);
                particles[idx] = p;
            }
        }
    }

    if idx < SPARK_END {
        update_metal_spark(idx);
    } else if idx < EMBER_END {
        update_ember(idx);
    } else if idx < RAY_END {
        update_god_ray(idx);
    } else {
        update_glitter(idx);
    }
}
