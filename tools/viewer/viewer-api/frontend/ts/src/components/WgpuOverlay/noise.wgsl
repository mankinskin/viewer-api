// noise.wgsl — procedural noise, RNG helpers
//
// Shared utility functions concatenated between types.wgsl and the
// pipeline-specific shader file.  No bindings declared here.
//
// Colour helpers (cinder_rgb, kind_ember) have moved to
// effects/particle-shading.wgsl so they can be shared with HypergraphView.

// ---- hash / noise -----------------------------------------------------------

// Integer-lattice hash — converts floor()ed coordinates to integers first,
// then mixes with a standard uint hash.  Avoids the bitcast-on-float
// correlation artefacts that caused visible checker patterns.
fn hash2(p: vec2f) -> f32 {
    let ix = u32(i32(p.x) + 32768);
    let iy = u32(i32(p.y) + 32768);
    var n = ix + iy * 0x45d9f3bu;
    n = (n ^ (n >> 16u)) * 0x45d9f3bu;
    n = (n ^ (n >> 16u)) * 0x45d9f3bu;
    n = n ^ (n >> 16u);
    return f32(n) / 4294967295.0;
}

fn smooth_noise(p: vec2f) -> f32 {
    let i  = floor(p);
    let f  = fract(p);
    let uv = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(hash2(i),                   hash2(i + vec2f(1.0, 0.0)), uv.x),
        mix(hash2(i + vec2f(0.0, 1.0)), hash2(i + vec2f(1.0, 1.0)), uv.x),
        uv.y
    );
}

fn fbm(p_in: vec2f) -> f32 {
    var val  = 0.0;
    var amp  = 0.5;
    var freq = 1.0;
    var p    = p_in;
    for (var i = 0; i < 4; i++) {
        val  += amp * smooth_noise(p * freq);
        amp  *= 0.5;
        freq *= 2.0;
    }
    return val;
}

// ---- pseudorandom number generator (PCG) ------------------------------------

fn pcg_hash(input: u32) -> u32 {
    var state = input * 747796405u + 2891336453u;
    let word  = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn rand_f(seed: u32) -> f32 {
    return f32(pcg_hash(seed)) / 4294967295.0;
}

fn rand2(seed: u32) -> vec2f {
    return vec2f(rand_f(seed), rand_f(seed + 1u));
}
