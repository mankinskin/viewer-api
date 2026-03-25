// Compact 3D math library for WebGPU (column-major matrices)

export type Vec3 = [number, number, number];
export type Mat4 = Float32Array; // 16 elements, column-major

export function vec3Sub(a: Vec3, b: Vec3): Vec3 {
    return [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
}

export function vec3Add(a: Vec3, b: Vec3): Vec3 {
    return [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
}

export function vec3Scale(v: Vec3, s: number): Vec3 {
    return [v[0] * s, v[1] * s, v[2] * s];
}

export function vec3Dot(a: Vec3, b: Vec3): number {
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}

export function vec3Cross(a: Vec3, b: Vec3): Vec3 {
    return [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
}

export function vec3Normalize(v: Vec3): Vec3 {
    const len = Math.sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
    if (len < 1e-8) return [0, 0, 0];
    return [v[0] / len, v[1] / len, v[2] / len];
}

// ── Matrices (column-major, WebGPU conventions) ──

export function mat4Identity(): Mat4 {
    const m = new Float32Array(16);
    m[0] = m[5] = m[10] = m[15] = 1;
    return m;
}

/** Perspective projection with Z ∈ [0,1] (WebGPU clip space) */
export function mat4Perspective(fovY: number, aspect: number, near: number, far: number): Mat4 {
    const m = new Float32Array(16);
    const f = 1.0 / Math.tan(fovY / 2);
    m[0] = f / aspect;
    m[5] = f;
    m[10] = far / (near - far);
    m[11] = -1;
    m[14] = (near * far) / (near - far);
    return m;
}

export function mat4LookAt(eye: Vec3, target: Vec3, up: Vec3): Mat4 {
    const f = vec3Normalize(vec3Sub(target, eye));
    const s = vec3Normalize(vec3Cross(f, up));
    const u = vec3Cross(s, f);

    const m = new Float32Array(16);
    m[0] = s[0];  m[1] = u[0];  m[2] = -f[0]; m[3] = 0;
    m[4] = s[1];  m[5] = u[1];  m[6] = -f[1]; m[7] = 0;
    m[8] = s[2];  m[9] = u[2];  m[10] = -f[2]; m[11] = 0;
    m[12] = -vec3Dot(s, eye);
    m[13] = -vec3Dot(u, eye);
    m[14] = vec3Dot(f, eye);
    m[15] = 1;
    return m;
}

export function mat4Multiply(a: Mat4, b: Mat4): Mat4 {
    const m = new Float32Array(16);
    for (let c = 0; c < 4; c++) {
        for (let r = 0; r < 4; r++) {
            let sum = 0;
            for (let k = 0; k < 4; k++) {
                sum += a[k * 4 + r] * b[c * 4 + k];
            }
            m[c * 4 + r] = sum;
        }
    }
    return m;
}

export function mat4Translate(v: Vec3): Mat4 {
    const m = mat4Identity();
    m[12] = v[0]; m[13] = v[1]; m[14] = v[2];
    return m;
}

export function mat4ScaleMat(v: Vec3): Mat4 {
    const m = new Float32Array(16);
    m[0] = v[0]; m[5] = v[1]; m[10] = v[2]; m[15] = 1;
    return m;
}

export function mat4RotateY(angle: number): Mat4 {
    const m = mat4Identity();
    const c = Math.cos(angle);
    const s = Math.sin(angle);
    m[0] = c;  m[2] = s;
    m[8] = -s; m[10] = c;
    return m;
}

/** Full 4×4 matrix inverse via cofactors (returns null if singular) */
export function mat4Inverse(m: Mat4): Mat4 | null {
    const inv = new Float32Array(16);
    const m0 = m[0]!;
    const m1 = m[1]!;
    const m2 = m[2]!;
    const m3 = m[3]!;
    const m4 = m[4]!;
    const m5 = m[5]!;
    const m6 = m[6]!;
    const m7 = m[7]!;
    const m8 = m[8]!;
    const m9 = m[9]!;
    const m10 = m[10]!;
    const m11 = m[11]!;
    const m12 = m[12]!;
    const m13 = m[13]!;
    const m14 = m[14]!;
    const m15 = m[15]!;

    inv[0]  =  m5*m10*m15 - m5*m11*m14 - m9*m6*m15
             + m9*m7*m14  + m13*m6*m11  - m13*m7*m10;
    inv[4]  = -m4*m10*m15 + m4*m11*m14  + m8*m6*m15
             - m8*m7*m14  - m12*m6*m11  + m12*m7*m10;
    inv[8]  =  m4*m9*m15  - m4*m11*m13  - m8*m5*m15
             + m8*m7*m13  + m12*m5*m11  - m12*m7*m9;
    inv[12] = -m4*m9*m14  + m4*m10*m13  + m8*m5*m14
             - m8*m6*m13  - m12*m5*m10  + m12*m6*m9;

    inv[1]  = -m1*m10*m15 + m1*m11*m14  + m9*m2*m15
             - m9*m3*m14  - m13*m2*m11  + m13*m3*m10;
    inv[5]  =  m0*m10*m15 - m0*m11*m14  - m8*m2*m15
             + m8*m3*m14  + m12*m2*m11  - m12*m3*m10;
    inv[9]  = -m0*m9*m15  + m0*m11*m13  + m8*m1*m15
             - m8*m3*m13  - m12*m1*m11  + m12*m3*m9;
    inv[13] =  m0*m9*m14  - m0*m10*m13  - m8*m1*m14
             + m8*m2*m13  + m12*m1*m10  - m12*m2*m9;

    inv[2]  =  m1*m6*m15  - m1*m7*m14   - m5*m2*m15
             + m5*m3*m14  + m13*m2*m7   - m13*m3*m6;
    inv[6]  = -m0*m6*m15  + m0*m7*m14   + m4*m2*m15
             - m4*m3*m14  - m12*m2*m7   + m12*m3*m6;
    inv[10] =  m0*m5*m15  - m0*m7*m13   - m4*m1*m15
             + m4*m3*m13  + m12*m1*m7   - m12*m3*m5;
    inv[14] = -m0*m5*m14  + m0*m6*m13   + m4*m1*m14
             - m4*m2*m13  - m12*m1*m6   + m12*m2*m5;

    inv[3]  = -m1*m6*m11  + m1*m7*m10   + m5*m2*m11
             - m5*m3*m10  - m9*m2*m7    + m9*m3*m6;
    inv[7]  =  m0*m6*m11  - m0*m7*m10   - m4*m2*m11
             + m4*m3*m10  + m8*m2*m7    - m8*m3*m6;
    inv[11] = -m0*m5*m11  + m0*m7*m9    + m4*m1*m11
             - m4*m3*m9   - m8*m1*m7    + m8*m3*m5;
    inv[15] =  m0*m5*m10  - m0*m6*m9    - m4*m1*m10
             + m4*m2*m9   + m8*m1*m6    - m8*m2*m5;

    let det = m0 * inv[0]! + m1 * inv[4]! + m2 * inv[8]! + m3 * inv[12]!;
    if (Math.abs(det) < 1e-10) return null;

    det = 1.0 / det;
    for (let i = 0; i < 16; i++) inv[i] *= det;
    return inv;
}

// ── Transforms ──

/** Transform a point (w=1) by a 4×4 matrix, with perspective divide */
export function mat4TransformPoint(m: Mat4, p: Vec3): Vec3 {
    const w = m[3] * p[0] + m[7] * p[1] + m[11] * p[2] + m[15];
    return [
        (m[0] * p[0] + m[4] * p[1] + m[8]  * p[2] + m[12]) / w,
        (m[1] * p[0] + m[5] * p[1] + m[9]  * p[2] + m[13]) / w,
        (m[2] * p[0] + m[6] * p[1] + m[10] * p[2] + m[14]) / w,
    ];
}

/** Transform a direction (w=0) by a 4×4 matrix */
export function mat4TransformDir(m: Mat4, d: Vec3): Vec3 {
    return [
        m[0] * d[0] + m[4] * d[1] + m[8]  * d[2],
        m[1] * d[0] + m[5] * d[1] + m[9]  * d[2],
        m[2] * d[0] + m[6] * d[1] + m[10] * d[2],
    ];
}

// ── Ray casting ──

export interface Ray { origin: Vec3; direction: Vec3 }

function mat4TransformVec4(
    m: Mat4, v: [number, number, number, number],
): [number, number, number, number] {
    return [
        m[0]*v[0] + m[4]*v[1] + m[8]*v[2]  + m[12]*v[3],
        m[1]*v[0] + m[5]*v[1] + m[9]*v[2]  + m[13]*v[3],
        m[2]*v[0] + m[6]*v[1] + m[10]*v[2] + m[14]*v[3],
        m[3]*v[0] + m[7]*v[1] + m[11]*v[2] + m[15]*v[3],
    ];
}

/** Cast a ray from camera through a screen pixel */
export function screenToRay(
    sx: number, sy: number,
    w: number, h: number,
    invViewProj: Mat4,
): Ray {
    const nx = (2 * sx / w) - 1;
    const ny = 1 - (2 * sy / h);

    const near4 = mat4TransformVec4(invViewProj, [nx, ny, 0, 1]);
    const far4  = mat4TransformVec4(invViewProj, [nx, ny, 1, 1]);
    const near: Vec3 = [near4[0] / near4[3], near4[1] / near4[3], near4[2] / near4[3]];
    const far:  Vec3 = [far4[0]  / far4[3],  far4[1]  / far4[3],  far4[2]  / far4[3]];

    return { origin: near, direction: vec3Normalize(vec3Sub(far, near)) };
}

/** Slab method for ray–AABB intersection. Returns t or null. */
export function rayAABBIntersect(ray: Ray, bmin: Vec3, bmax: Vec3): number | null {
    let tmin = -Infinity;
    let tmax = Infinity;

    for (let i = 0; i < 3; i++) {
        if (Math.abs(ray.direction[i]) < 1e-8) {
            if (ray.origin[i] < bmin[i] || ray.origin[i] > bmax[i]) return null;
        } else {
            let t1 = (bmin[i] - ray.origin[i]) / ray.direction[i];
            let t2 = (bmax[i] - ray.origin[i]) / ray.direction[i];
            if (t1 > t2) { const tmp = t1; t1 = t2; t2 = tmp; }
            tmin = Math.max(tmin, t1);
            tmax = Math.min(tmax, t2);
            if (tmin > tmax) return null;
        }
    }

    if (tmax < 0) return null;
    return tmin >= 0 ? tmin : tmax;
}

/** Ray–horizontal-plane intersection at given Y */
export function rayPlaneIntersect(ray: Ray, planeY: number): Vec3 | null {
    if (Math.abs(ray.direction[1]) < 1e-8) return null;
    const t = (planeY - ray.origin[1]) / ray.direction[1];
    if (t < 0) return null;
    return vec3Add(ray.origin, vec3Scale(ray.direction, t));
}

/** Ray–plane intersection for an arbitrary plane defined by a point and normal */
export function rayPlaneIntersectGeneral(ray: Ray, planePoint: Vec3, planeNormal: Vec3): Vec3 | null {
    const denom = vec3Dot(ray.direction, planeNormal);
    if (Math.abs(denom) < 1e-8) return null; // ray parallel to plane
    const t = vec3Dot(vec3Sub(planePoint, ray.origin), planeNormal) / denom;
    if (t < 0) return null;
    return vec3Add(ray.origin, vec3Scale(ray.direction, t));
}

// ── 3D projection helpers (shared with HypergraphView) ──

/**
 * Project a world position to screen coordinates.
 */
export function worldToScreen(
    worldPos: Vec3,
    viewProj: Float32Array,
    cw: number,
    ch: number,
): { x: number; y: number; z: number; visible: boolean } {
    const vp = viewProj;
    const cx = vp[0]! * worldPos[0] + vp[4]! * worldPos[1] + vp[8]! * worldPos[2] + vp[12]!;
    const cy = vp[1]! * worldPos[0] + vp[5]! * worldPos[1] + vp[9]! * worldPos[2] + vp[13]!;
    const cz = vp[2]! * worldPos[0] + vp[6]! * worldPos[1] + vp[10]! * worldPos[2] + vp[14]!;
    const cw2 = vp[3]! * worldPos[0] + vp[7]! * worldPos[1] + vp[11]! * worldPos[2] + vp[15]!;

    if (cw2 <= 0.001) return { x: -9999, y: -9999, z: 1, visible: false };

    const ndcX = cx / cw2;
    const ndcY = cy / cw2;
    const ndcZ = cz / cw2;

    const sx = (ndcX * 0.5 + 0.5) * cw;
    const sy = (1 - (ndcY * 0.5 + 0.5)) * ch;

    return { x: sx, y: sy, z: ndcZ, visible: ndcZ >= 0 && ndcZ <= 1 };
}

/**
 * Pixels-per-world-unit at a given world position.
 *
 * Uses the Euclidean distance from the camera to the point and the known
 * vertical FOV. This is completely independent of camera orientation —
 * a node at a given distance from the camera always has the same on-screen
 * scale regardless of which direction the camera faces.
 */
const HALF_FOV_TAN = Math.tan(Math.PI / 8); // tan(fov/2) where fov = PI/4

export function worldScaleAtDepth(
    camPos: Vec3,
    worldPos: Vec3,
    ch: number,
): number {
    const dx = worldPos[0] - camPos[0];
    const dy = worldPos[1] - camPos[1];
    const dz = worldPos[2] - camPos[2];
    const dist = Math.sqrt(dx * dx + dy * dy + dz * dz);
    if (dist < 0.001) return ch; // prevent division by zero
    return ch / (2 * dist * HALF_FOV_TAN);
}

/**
 * Ray-sphere intersection test.
 * Returns the distance along the ray to the first intersection, or null if no hit.
 */
export function raySphere(
    ro: Vec3,
    rd: Vec3,
    center: Vec3,
    radius: number,
): number | null {
    const oc: Vec3 = [ro[0] - center[0], ro[1] - center[1], ro[2] - center[2]];
    const a = rd[0] * rd[0] + rd[1] * rd[1] + rd[2] * rd[2];
    const b = 2 * (oc[0] * rd[0] + oc[1] * rd[1] + oc[2] * rd[2]);
    const c = oc[0] * oc[0] + oc[1] * oc[1] + oc[2] * oc[2] - radius * radius;
    const disc = b * b - 4 * a * c;
    if (disc < 0) return null;
    const t = (-b - Math.sqrt(disc)) / (2 * a);
    return t > 0 ? t : null;
}
