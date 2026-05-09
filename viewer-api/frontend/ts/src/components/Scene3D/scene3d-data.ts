import type { Vec3 } from '../../utils/math3d';

export interface SceneObject {
    position: Vec3;
    scale: Vec3;
    color: [number, number, number, number];
    rotationY: number;
}

export interface SceneState {
    objects: SceneObject[];
    camYaw: number;
    camPitch: number;
    camDist: number;
    camTarget: Vec3;
    dragIdx: number;
    dragPlaneY: number;
    dragOffset: Vec3;
    orbiting: boolean;
    lastMX: number;
    lastMY: number;
    hoverIdx: number;
    mouseX: number;
    mouseY: number;
    startTime: number;
}

/** Unit cube (-0.5 -> 0.5), 36 verts, interleaved pos+normal (6 floats each). */
export function createCubeGeometry(): Float32Array {
    const faces: { n: Vec3; v: Vec3[] }[] = [
        { n: [0, 0, 1], v: [[-0.5, -0.5, 0.5], [0.5, -0.5, 0.5], [0.5, 0.5, 0.5], [-0.5, 0.5, 0.5]] },
        { n: [0, 0, -1], v: [[0.5, -0.5, -0.5], [-0.5, -0.5, -0.5], [-0.5, 0.5, -0.5], [0.5, 0.5, -0.5]] },
        { n: [1, 0, 0], v: [[0.5, -0.5, 0.5], [0.5, -0.5, -0.5], [0.5, 0.5, -0.5], [0.5, 0.5, 0.5]] },
        { n: [-1, 0, 0], v: [[-0.5, -0.5, -0.5], [-0.5, -0.5, 0.5], [-0.5, 0.5, 0.5], [-0.5, 0.5, -0.5]] },
        { n: [0, 1, 0], v: [[-0.5, 0.5, 0.5], [0.5, 0.5, 0.5], [0.5, 0.5, -0.5], [-0.5, 0.5, -0.5]] },
        { n: [0, -1, 0], v: [[-0.5, -0.5, -0.5], [0.5, -0.5, -0.5], [0.5, -0.5, 0.5], [-0.5, -0.5, 0.5]] },
    ];
    const data: number[] = [];
    for (const { n, v } of faces) {
        const [a, b, c, e] = v as [Vec3, Vec3, Vec3, Vec3];
        data.push(a[0], a[1], a[2], n[0], n[1], n[2]);
        data.push(b[0], b[1], b[2], n[0], n[1], n[2]);
        data.push(c[0], c[1], c[2], n[0], n[1], n[2]);
        data.push(a[0], a[1], a[2], n[0], n[1], n[2]);
        data.push(c[0], c[1], c[2], n[0], n[1], n[2]);
        data.push(e[0], e[1], e[2], n[0], n[1], n[2]);
    }
    return new Float32Array(data);
}

export const UNIFORM_STRIDE = 256;
export const MAX_DRAWS = 16;
export const CUBE_VERTS = 36;
export const GRID_LINE_FLOATS = 12;

const GRID_EXTENT = 10;
const GRID_STEP = 1;

export function createGridLineData(): { data: Float32Array; count: number } {
    const lines: number[] = [];
    for (let i = -GRID_EXTENT; i <= GRID_EXTENT; i += GRID_STEP) {
        lines.push(i, 0, -GRID_EXTENT, i, 0, GRID_EXTENT, 0.25, 0.24, 0.22, 0.08, 0, 0);
        lines.push(-GRID_EXTENT, 0, i, GRID_EXTENT, 0, i, 0.25, 0.24, 0.22, 0.08, 0, 0);
    }
    lines.push(-GRID_EXTENT, 0, 0, GRID_EXTENT, 0, 0, 0.55, 0.22, 0.18, 0.18, 1, 0);
    lines.push(0, 0, -GRID_EXTENT, 0, 0, GRID_EXTENT, 0.18, 0.22, 0.55, 0.18, 1, 0);
    return {
        data: new Float32Array(lines),
        count: lines.length / GRID_LINE_FLOATS,
    };
}

export const INITIAL_OBJECTS: SceneObject[] = [
    { position: [0, 0.5, 0], scale: [1, 1, 1], color: [0.95, 0.25, 0.21, 1], rotationY: 0 },
    { position: [2.5, 1.0, 0.8], scale: [0.5, 2.0, 0.5], color: [0.18, 0.60, 0.95, 1], rotationY: 0.2 },
    { position: [-2.2, 0.2, -1.2], scale: [1.8, 0.4, 1.5], color: [0.22, 0.88, 0.38, 1], rotationY: 0.5 },
    { position: [1.2, 0.25, -2.3], scale: [0.5, 0.5, 0.5], color: [0.98, 0.85, 0.15, 1], rotationY: 0.8 },
    { position: [-1.5, 0.5, 2.2], scale: [1, 1, 1], color: [0.72, 0.32, 0.95, 1], rotationY: 1.1 },
    { position: [3.2, 0.45, -2.0], scale: [0.9, 0.9, 1.4], color: [0.15, 0.85, 0.82, 1], rotationY: 0.4 },
];

export function createInitialSceneState(): SceneState {
    return {
        objects: INITIAL_OBJECTS.map((object) => ({
            ...object,
            position: [...object.position] as Vec3,
            scale: [...object.scale] as Vec3,
            color: [...object.color] as [number, number, number, number],
        })),
        camYaw: 0.6,
        camPitch: 0.35,
        camDist: 10,
        camTarget: [0, 0.5, 0] as Vec3,
        dragIdx: -1,
        dragPlaneY: 0,
        dragOffset: [0, 0, 0] as Vec3,
        orbiting: false,
        lastMX: 0,
        lastMY: 0,
        hoverIdx: -1,
        mouseX: 0,
        mouseY: 0,
        startTime: performance.now() / 1000,
    };
}