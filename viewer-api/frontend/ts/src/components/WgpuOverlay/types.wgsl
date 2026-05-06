// types.wgsl — shared struct definitions for all shader modules
//
// Concatenated after palette.wgsl and before noise.wgsl / pipeline files.
// Declares the palette uniform binding (shared across ALL pipelines).

// ---- palette uniform (binding 3, shared by compute + render) ----------------
@group(0) @binding(3) var<uniform> palette : ThemePalette;

// ---- particle kind constants ------------------------------------------------
const PK_METAL_SPARK : f32 = 0.0;
const PK_EMBER       : f32 = 1.0;
const PK_GOD_RAY     : f32 = 2.0;
const PK_GLITTER     : f32 = 3.0;

// Screen-space flag for kind_view packing
// kind_view = kind + view_id * 8 + is_screen_space * 64
const PK_SCREEN_SPACE : f32 = 64.0;

// ---- element kind constants for effect preview containers -------------------
const KIND_FX_SPARK   : f32 = 8.0;
const KIND_FX_EMBER   : f32 = 9.0;
const KIND_FX_BEAM    : f32 = 10.0;
const KIND_FX_GLITTER : f32 = 11.0;

// ---- index ranges per particle type (must match TypeScript) -----------------
const SPARK_END   : u32 = 96u;
const EMBER_END   : u32 = 288u;
const RAY_END     : u32 = 544u;
const GLITTER_END : u32 = 640u;

// ---- uniforms (352 bytes = 52 scalars + 4 camera floats + 2 × mat4x4<f32>) ---------------------
struct Uniforms {
    time             : f32,
    width            : f32,
    height           : f32,
    element_count    : f32,
    mouse_x          : f32,
    mouse_y          : f32,
    delta_time       : f32,
    hover_elem       : f32,
    hover_start_time : f32,
    selected_elem    : f32,    // index of selected element (-1 if none)
    crt_scanlines_h  : f32,    // horizontal scanlines (+grid) intensity 0.0–1.0
    crt_scanlines_v  : f32,    // vertical scanlines (+grid) intensity 0.0–1.0
    crt_edge_shadow  : f32,    // edge/border shadow intensity 0.0–1.0
    crt_flicker      : f32,    // torch flicker intensity 0.0–1.0
    crt_line_width   : f32,    // CRT scanline width/thickness 0.0–1.0 (0 = thin, 1 = wide)
    smoke_intensity  : f32,    // background smoke brightness 0.0–1.0
    smoke_speed      : f32,    // smoke animation speed multiplier 0.0–5.0
    smoke_warm_scale : f32,    // UV scale for warm smoke layers 0.0–2.0
    smoke_cool_scale : f32,    // UV scale for cool wisp layer 0.0–2.0
    smoke_moss_scale : f32,    // UV scale for moss-tone blending 0.0–2.0
    grain_intensity  : f32,    // grain brightness/amplitude 0.0–1.0
    grain_coarseness : f32,    // grain frequency scale 0.0–1.0
    grain_size       : f32,    // grain pixel block size (1–8 px, normalized 0.0–1.0)
    vignette_str     : f32,    // edge vignette darkening 0.0–1.0
    underglow_str    : f32,    // warm bottom underglow 0.0–1.0
    spark_speed      : f32,    // metal spark speed multiplier 0.0–3.0
    ember_speed      : f32,    // ember/ash speed multiplier 0.0–3.0
    beam_speed       : f32,    // angelic beam speed multiplier 0.0–3.0
    glitter_speed    : f32,    // glitter speed multiplier 0.0–3.0
    beam_height      : f32,    // beam quad height multiplier (default 35.0)
    beam_count       : f32,    // max active beams (0 = all slots)
    beam_drift       : f32,    // beam upward drift distance multiplier 0.0–3.0
    scroll_dx        : f32,    // scroll delta X this frame (pixels, screen space)
    scroll_dy        : f32,    // scroll delta Y this frame (pixels, screen space)
    spark_count      : f32,    // max active sparks (fraction of slots, 0–2)
    spark_size       : f32,    // spark size multiplier 0.0–3.0
    ember_count      : f32,    // max active embers (fraction of slots, 0–2)
    ember_size       : f32,    // ember size multiplier 0.0–3.0
    glitter_count    : f32,    // max active glitter (fraction of slots, 0–2)
    glitter_size     : f32,    // glitter size multiplier 0.0–3.0
    cinder_size      : f32,    // cinder border glow size multiplier 0.0–3.0
    ref_depth        : f32,    // reference NDC depth for unprojection (0.0 for 2D views)
    world_scale      : f32,    // world units per screen pixel (1.0 for 2D views)
    vp_x             : f32,    // particle viewport left (canvas pixels)
    vp_y             : f32,    // particle viewport top (canvas pixels)
    vp_w             : f32,    // particle viewport width (pixels)
    vp_h             : f32,    // particle viewport height (pixels)
    current_view     : f32,    // active view/tab ID (0=logs,1=stats,2=code,3=debug,4=scene3d,5=hypergraph,6=settings)
    // ---- CRT scanline color (4 floats for alignment) ----
    crt_color_r      : f32,    // CRT scanline tint red 0.0–1.0
    crt_color_g      : f32,    // CRT scanline tint green 0.0–1.0
    crt_color_b      : f32,    // CRT scanline tint blue 0.0–1.0
    _crt_pad         : f32,    // padding for mat4 alignment
    // ---- camera position for 3D skybox (4 floats for mat4 alignment) ----
    camera_pos_x     : f32,    // camera world-space X position
    camera_pos_y     : f32,    // camera world-space Y position
    camera_pos_z     : f32,    // camera world-space Z position
    _cam_pad         : f32,    // padding for mat4 alignment
    // ---- projection matrices (column-major, 16 f32 each) ----
    particle_vp      : mat4x4<f32>,   // world → clip (ortho for 2D, camera VP for 3D)
    particle_inv_vp  : mat4x4<f32>,   // clip → world (for unprojecting spawn positions)
}

// ---- DOM element rectangle --------------------------------------------------
struct ElemRect {
    rect  : vec4f,   // x, y, w, h
    hue   : f32,
    kind  : f32,
    depth : f32,     // NDC depth (0 = flat/2D; >0 = 3D-positioned element)
    _p2   : f32,
}

// ---- particle state (48 bytes) -----------------------------------------------
// vec3f has alignment 16 in storage buffers, so life/max_life fill the
// padding slots after each vec3f → total 48 bytes with no waste.
struct Particle {
    pos       : vec3f,      // world-space position (screen pixels for 2D, world units for 3D)
    life      : f32,
    vel       : vec3f,      // world-space velocity
    max_life  : f32,
    hue       : f32,
    size      : f32,        // visual size in screen pixels
    kind_view : f32,        // packed: kind + view_id * 8 (kind 0-3, view_id 0-7)
    spawn_t   : f32,        // absolute time when particle was spawned
}
