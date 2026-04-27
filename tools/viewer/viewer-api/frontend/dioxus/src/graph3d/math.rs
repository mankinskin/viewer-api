//! Linear algebra helpers for the 3-D graph view.
//!
//! Column-major matrices, right-handed view space, WebGPU clip-space
//! z ∈ [0, 1].

/// Perspective projection matrix (WebGPU clip-space z ∈ [0, 1]).
pub fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> [f32; 16] {
    let f = 1.0 / (fov * 0.5).tan();
    let nf = 1.0 / (near - far);
    let mut m = [0.0f32; 16];
    m[0]  = f / aspect;
    m[5]  = f;
    m[10] = far * nf;
    m[11] = -1.0;
    m[14] = near * far * nf;
    m
}

/// Look-at view matrix (column-major, right-handed).
pub fn look_at(eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> [f32; 16] {
    let fwd = normalise([
        target[0] - eye[0],
        target[1] - eye[1],
        target[2] - eye[2],
    ]);
    let side = normalise(cross(fwd, up));
    let u    = cross(side, fwd);
    let mut m = [0.0f32; 16];
    m[0]  = side[0]; m[4] = side[1]; m[8]  = side[2];
    m[1]  = u[0];    m[5] = u[1];    m[9]  = u[2];
    m[2]  = -fwd[0]; m[6] = -fwd[1]; m[10] = -fwd[2];
    m[12] = -dot(side, eye);
    m[13] = -dot(u, eye);
    m[14] =  dot(fwd, eye);
    m[15] = 1.0;
    m
}

/// Column-major 4×4 multiply: out = a · b.
pub fn mul(a: [f32; 16], b: [f32; 16]) -> [f32; 16] {
    let mut out = [0.0f32; 16];
    for col in 0..4 {
        for row in 0..4 {
            let mut s = 0.0;
            for k in 0..4 {
                s += a[k * 4 + row] * b[col * 4 + k];
            }
            out[col * 4 + row] = s;
        }
    }
    out
}

fn normalise(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 { return [0.0, 0.0, 1.0]; }
    [v[0] / len, v[1] / len, v[2] / len]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
