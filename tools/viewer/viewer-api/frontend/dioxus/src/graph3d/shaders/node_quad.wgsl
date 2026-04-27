// Node occluder quad shader — billboarded quads that write depth so GPU edges
// behind DOM ticket cards are properly occluded.

struct Camera {
    viewProj : mat4x4<f32>,
    eye      : vec4<f32>,
    time     : vec4<f32>,   // x = time, y = viewport_w, z = viewport_h
};

// Unused here but the shared bind-group layout requires both bindings.
struct Palette { _data : array<vec4f, 24> };

@group(0) @binding(0) var<uniform> cam : Camera;
@group(0) @binding(1) var<uniform> _palette : Palette;

@vertex
fn vs_node_quad(
    @location(0) quadPos : vec2<f32>,   // unit quad vertex (-1..1)
    @location(1) nodePos : vec3<f32>,   // world-space node centre
) -> @builtin(position) vec4<f32> {
    let clip = cam.viewProj * vec4(nodePos, 1.0);

    // Match the DOM card's depth-based pixel_scale
    let d   = cam.eye.xyz - nodePos;
    let dist = max(length(d), 0.1);
    let pixel_scale = clamp(15.0 / dist, 0.15, 2.5);

    // Slightly oversized to fully cover the DOM card (160×~50 px base).
    let card_w = 175.0 * pixel_scale;
    let card_h = 58.0  * pixel_scale;

    let vw = cam.time.y;
    let vh = cam.time.z;

    // Offset in clip-space (screen-aligned billboard).
    let ox = quadPos.x * card_w / vw * clip.w;
    let oy = quadPos.y * card_h / vh * clip.w;

    // Push depth slightly back so edges connecting TO this node aren't clipped.
    return vec4(clip.x + ox, clip.y + oy, clip.z + 0.0005 * clip.w, clip.w);
}

@fragment
fn fs_node_quad() -> @location(0) vec4<f32> {
    // Match the canvas clear colour so quads are invisible against the background.
    // Only the depth write matters — the DOM card renders text/borders on top.
    return vec4(0.05, 0.05, 0.07, 1.0);
}
