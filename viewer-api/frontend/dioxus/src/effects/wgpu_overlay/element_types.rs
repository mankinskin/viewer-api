//! Selector registry, element-kind constants, and packed-buffer sizes.
//!
//! Pure data — no DOM access, no GPU dependency. Mirrors
//! `tools/viewer/log-viewer/frontend/src/components/WgpuOverlay/element-types.ts`.

// ── Storage buffer layout ────────────────────────────────────────────────────

/// `f32` values per element rect: `[x, y, w, h, hue, kind, depth, _pad]`.
pub(super) const ELEM_FLOATS: usize = 8;
/// Bytes per element (16-byte aligned).
pub(super) const ELEM_BYTES: usize = ELEM_FLOATS * 4;

/// Initial element-buffer capacity (doubled dynamically on overflow).
pub(super) const INITIAL_ELEM_CAP: usize = 128;

// ── Particles ────────────────────────────────────────────────────────────────

/// Total number of particles simulated by the compute shader.
pub(super) const NUM_PARTICLES: usize = 640;
/// `f32` values per particle (vec3f-aligned, 48 bytes total).
pub(super) const PARTICLE_FLOATS: usize = 12;
pub(super) const PARTICLE_BUF_SIZE: usize = NUM_PARTICLES * PARTICLE_FLOATS * 4;
/// Compute-shader workgroup size (must match `compute.wgsl`).
pub(super) const COMPUTE_WORKGROUP: usize = 64;

// ── Uniforms / palette ───────────────────────────────────────────────────────

/// Palette uniform: 24 × `vec4f` = 384 bytes.
pub(super) const PALETTE_VEC4_COUNT: usize = 24;
pub(super) const PALETTE_BYTE_SIZE: usize = PALETTE_VEC4_COUNT * 16;

/// Uniforms buffer: 88 × `f32` = 352 bytes (matches `types.wgsl` `Uniforms` struct).
pub(super) const UNIFORMS_F32_COUNT: usize = 88;
pub(super) const UNIFORMS_BYTE_SIZE: usize = UNIFORMS_F32_COUNT * 4;

// ── WebGPU buffer-usage flags (mirror GPUBufferUsage) ────────────────────────

pub(super) const USAGE_UNIFORM: u32 = 0x0040;
pub(super) const USAGE_STORAGE: u32 = 0x0080;
pub(super) const USAGE_COPY_DST: u32 = 0x0008;

// ── DOM selectors ────────────────────────────────────────────────────────────

/// `(selector, kind)` pairs scanned each frame.  Hue is `idx / len`.
pub(super) const UI_SELECTORS: &[(&str, u32)] = &[
    // Structural regions → kind 0
    (".header",              0),
    (".sidebar",             0),
    (".tab-bar",             0),
    (".filter-panel",        0),
    (".view-container",      0),
    (".log-list",            0),
    (".code-viewer",         0),
    // Main content panel (shared viewer-api class — ticket-viewer, spec-viewer) → kind 0
    (".content",             0),
    // Spec-viewer list items → kind 0
    (".spec-card",           0),
    // Per-severity log entries → kinds 1-4
    (".log-entry.level-error", 1),
    (".log-entry.level-warn",  2),
    (".log-entry.level-info",  3),
    (".log-entry.level-debug", 4),
    (".log-entry.level-trace", 4),
    // Interactive states → kinds 5-7
    (".log-entry.span-highlighted", 5),
    (".log-entry.selected",         6),
    (".log-entry.panic-entry",      7),
    // Spec-viewer selected item → kind 6
    (".spec-card--selected",  6),
];
