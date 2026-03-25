/**
 * Public API surface for the WgpuOverlay system.
 *
 * Provides:
 *   - **Signals** — `gpuOverlayEnabled`, `overlayGpu`
 *   - **Overlay callbacks** — register/unregister renderers that draw into
 *     the single shared render pass (edges, grids, cubes)
 *   - **Particle projection** — viewProj/inverse, viewport region, depth,
 *     world-scale; set by 3D views each frame in their overlay callbacks
 *   - **Capture** — one-shot JPEG thumbnail (theme previews)
 *   - **Invalidation** — scan dirty / particle reset delegates
 */
import { signal } from '@preact/signals';

// ---------------------------------------------------------------------------
// GPU enabled toggle
// ---------------------------------------------------------------------------

export const gpuOverlayEnabled = signal(true);

// ---------------------------------------------------------------------------
// FX enabled toggle — controls visual effects (particles, background smoke,
// CRT, element decorations, cursors) independently of the GPU pipeline.
// When off, overlay callbacks (3D edges, grids) still render.
// ---------------------------------------------------------------------------

export const fxEnabled = signal(true);

// ---------------------------------------------------------------------------
// Shared GPU device + format for external renderers
// ---------------------------------------------------------------------------

/**
 * Exposes the shared GPU device + canvas format so external components
 * can create pipelines compatible with the overlay's render pass.
 * `null` when WebGPU is not available or the overlay is disabled.
 */
export const overlayGpu = signal<{ device: GPUDevice; format: GPUTextureFormat } | null>(null);

// ---------------------------------------------------------------------------
// Overlay render callback system — allows external components (e.g.
// HypergraphView) to draw into the shared WgpuOverlay canvas.
// ---------------------------------------------------------------------------

/**
 * Callback invoked during the overlay's render pass each frame.
 * Receivers can set their own pipeline, bind groups, viewport/scissor,
 * and issue draw calls.  Buffer writes via `device.queue.writeBuffer()`
 * are safe here — they're staged before `queue.submit()`.
 *
 * The `depthView` parameter provides the shared depth texture (format:
 * depth24plus) for 3D callbacks that need depth testing.  2D overlay
 * pipelines can ignore it.
 */
export type OverlayRenderCallback = (
    pass: GPURenderPassEncoder,
    device: GPUDevice,
    time: number,
    dt: number,
    canvasWidth: number,
    canvasHeight: number,
    depthView: GPUTextureView,
) => void;

const _overlayCallbacks = new Set<OverlayRenderCallback>();

export function registerOverlayRenderer(cb: OverlayRenderCallback): void {
    _overlayCallbacks.add(cb);
}

export function unregisterOverlayRenderer(cb: OverlayRenderCallback): void {
    _overlayCallbacks.delete(cb);
}

/** Read-only access to the set of registered callbacks (used by render loop). */
export function getOverlayCallbacks(): ReadonlySet<OverlayRenderCallback> {
    return _overlayCallbacks;
}

// ---------------------------------------------------------------------------
// One-shot frame capture (for theme thumbnails)
// ---------------------------------------------------------------------------

let _captureResolve: ((url: string) => void) | null = null;

/**
 * Request a low-res JPEG thumbnail of the next rendered frame.
 * The promise resolves after the frame is submitted to the GPU.
 */
export function captureOverlayThumbnail(): Promise<string> {
    return new Promise(resolve => {
        _captureResolve = resolve;
    });
}

/** Non-destructive check for pending capture request (used by frame skip logic). */
export function hasCaptureRequest(): boolean {
    return _captureResolve != null;
}

/** Consume and return a pending capture resolver (used by render loop). */
export function consumeCaptureRequest(): ((url: string) => void) | null {
    const resolve = _captureResolve;
    _captureResolve = null;
    return resolve;
}

// ---------------------------------------------------------------------------
// Scan dirty trigger — delegates to the live ElementScanner instance
// ---------------------------------------------------------------------------

/** Callback set by WgpuOverlay component when scanner is created/destroyed. */
let _scanInvalidator: (() => void) | null = null;

/** Register/unregister the scanner's invalidateAll method. */
export function setScanInvalidator(fn: (() => void) | null): void {
    _scanInvalidator = fn;
}

/**
 * Mark the element scan as dirty so positions are re-queried on the next
 * animation frame.  Call this from any component that moves, adds, or
 * removes overlay-tracked DOM elements (e.g. HypergraphView after
 * repositioning nodes).
 */
export function markOverlayScanDirty(): void {
    _scanInvalidator?.();
}

// ---------------------------------------------------------------------------
// Particle reset — delegates to the live BufferManager instance
// ---------------------------------------------------------------------------

/** Callback set by WgpuOverlay component when buffers are created/destroyed. */
let _particleResetter: (() => void) | null = null;

/** Register/unregister the buffer's resetParticles method. */
export function setParticleResetter(fn: (() => void) | null): void {
    _particleResetter = fn;
}

/**
 * Clear all overlay particles immediately.  Call this when switching tabs
 * or any context change where particles from the previous view should not
 * persist into the new view.
 */
export function resetOverlayParticles(): void {
    _particleResetter?.();
}

// ---------------------------------------------------------------------------
// Particle projection for 3D views (viewProj, viewport, depth)
// ---------------------------------------------------------------------------

/**
 * ViewProj and inverse ViewProj matrices for projecting world-space particles
 * to clip space. For 2D views these are set to orthographic matrices by the
 * render loop. For 3D views, the view component (HypergraphView, Scene3D)
 * sets these each frame with its camera's projection.
 *
 * The matrices are "full-canvas" viewProj, i.e. they map world positions to
 * NDC coordinates relative to the full overlay canvas (not the sub-viewport).
 */
let _particleVP: Float32Array = new Float32Array(16);
let _particleInvVP: Float32Array = new Float32Array(16);
let _vpDirty = false;

/**
 * Set the viewProj and inverse viewProj for particle projection this frame.
 * Call from 3D view overlay callbacks. The matrices should be pre-composed
 * with the viewport transform so they produce full-canvas clip coordinates.
 */
export function setOverlayParticleVP(vp: Float32Array, invVp: Float32Array): void {
    _particleVP = vp;
    _particleInvVP = invVp;
    _vpDirty = true;
}

/** Read the current particle viewProj (used by render loop). */
export function getOverlayParticleVP(): Float32Array { return _particleVP; }

/** Read the current inverse viewProj (used by render loop). */
export function getOverlayParticleInvVP(): Float32Array { return _particleInvVP; }

/** Check and consume the dirty flag (render loop uses this to detect 3D updates). */
export function consumeParticleVPDirty(): boolean {
    const d = _vpDirty;
    _vpDirty = false;
    return d;
}

// ---------------------------------------------------------------------------
// Particle viewport (sub-region of canvas used by 3D view)
// ---------------------------------------------------------------------------

const _particleViewport: [number, number, number, number] = [0, 0, 0, 0];

/**
 * Set the viewport region for the active 3D view (canvas-pixel coordinates).
 * The render loop uses this as the default for 3D views; 2D views use
 * (0, 0, canvasWidth, canvasHeight) automatically.
 */
export function setOverlayParticleViewport(x: number, y: number, w: number, h: number): void {
    _particleViewport[0] = x;
    _particleViewport[1] = y;
    _particleViewport[2] = w;
    _particleViewport[3] = h;
}

/** Read the particle viewport (used by render loop). */
export function getOverlayParticleViewport(): [number, number, number, number] {
    return _particleViewport;
}

// ---------------------------------------------------------------------------
// Reference depth for unprojection
// ---------------------------------------------------------------------------

let _refDepth: number = 0;

/**
 * Set the NDC depth used when unprojecting screen positions to world space.
 * For 2D views: 0 (particles at z=0). For 3D views: the NDC z of the
 * camera target (approximate depth of scene elements).
 */
export function setOverlayRefDepth(z: number): void { _refDepth = z; }

/** Read the reference depth (used by render loop). */
export function getOverlayRefDepth(): number { return _refDepth; }

// ---------------------------------------------------------------------------
// World scale (world units per screen pixel)
// ---------------------------------------------------------------------------

let _worldScale: number = 1;

/**
 * Set the conversion factor from screen pixels to world units.
 * For 2D views: 1.0 (world ≡ screen pixels). For 3D views: computed
 * from camera distance & viewport height.
 */
export function setOverlayWorldScale(s: number): void { _worldScale = s; }

/** Read the world scale (used by render loop). */
export function getOverlayWorldScale(): number { return _worldScale; }
// ---------------------------------------------------------------------------
// Camera position for 3D skybox rendering
// ---------------------------------------------------------------------------

const _cameraPos: [number, number, number] = [0, 0, 0];

/**
 * Set the camera world-space position for skybox rendering.
 * Call from 3D views each frame along with setOverlayParticleVP.
 * The background shader uses this to compute view rays for spherical sampling.
 */
export function setOverlayCameraPos(x: number, y: number, z: number): void {
    _cameraPos[0] = x;
    _cameraPos[1] = y;
    _cameraPos[2] = z;
}

/** Read the camera position (used by render loop for skybox uniforms). */
export function getOverlayCameraPos(): [number, number, number] { return _cameraPos; }