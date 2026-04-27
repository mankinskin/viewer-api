//! Dynamic GPU buffer management.
//!
//! Owns the four GPU buffers (uniform, elements, particles, palette) and
//! handles dynamic element-buffer resizing.  Mirrors `gpu-buffers.ts`.

#![cfg(target_arch = "wasm32")]

use js_sys::{Array, Float32Array, Object};
use wasm_bindgen::JsValue;

use super::element_types::*;
use super::webgpu::*;

pub(super) struct GpuBuffers {
    pub uniform_buf:   JsValue,
    pub elem_buf:      JsValue,
    pub particle_buf:  JsValue,
    pub palette_buf:   JsValue,
    pub elem_capacity: usize,
}

impl GpuBuffers {
    /// Allocate all four buffers and seed the particle/palette buffers with
    /// initial data.
    pub fn new(device: &JsValue, queue: &JsValue) -> Option<Self> {
        let uniform_buf  = gpu_buffer(device, UNIFORMS_BYTE_SIZE as u32,
                                      USAGE_UNIFORM | USAGE_COPY_DST)?;
        let elem_buf     = gpu_buffer(device, (INITIAL_ELEM_CAP * ELEM_BYTES) as u32,
                                      USAGE_STORAGE | USAGE_COPY_DST)?;
        let particle_buf = gpu_buffer(device, PARTICLE_BUF_SIZE as u32,
                                      USAGE_STORAGE | USAGE_COPY_DST)?;
        let palette_buf  = gpu_buffer(device, PALETTE_BYTE_SIZE as u32,
                                      USAGE_UNIFORM | USAGE_COPY_DST)?;

        // Zero-init the particle buffer so all particles start dead.
        {
            let zeros = Float32Array::new_with_length((NUM_PARTICLES * PARTICLE_FLOATS) as u32);
            queue_write_f32(queue, &particle_buf, 0, &zeros);
        }
        // Default dark-theme palette.
        queue_write_f32(queue, &palette_buf, 0, &default_palette_f32());

        Some(Self {
            uniform_buf,
            elem_buf,
            particle_buf,
            palette_buf,
            elem_capacity: INITIAL_ELEM_CAP,
        })
    }

    /// Ensure `elem_buf` can hold `count` elements, doubling the allocation
    /// when needed.  Returns `true` when the buffer was reallocated (caller
    /// must rebuild the bind groups).
    pub fn ensure_elem_capacity(&mut self, device: &JsValue, count: usize) -> bool {
        if count <= self.elem_capacity { return false; }
        let new_cap = (count * 2).max(INITIAL_ELEM_CAP);
        if let Some(buf) = gpu_buffer(
            device,
            (new_cap * ELEM_BYTES) as u32,
            USAGE_STORAGE | USAGE_COPY_DST,
        ) {
            self.elem_buf      = buf;
            self.elem_capacity = new_cap;
            return true;
        }
        false
    }
}

// ── Bind-group factories ─────────────────────────────────────────────────────

pub(super) fn mk_compute_bind_group(
    device: &JsValue, layout: &JsValue, buffers: &GpuBuffers,
) -> Option<JsValue> {
    let entries = Array::new();
    entries.push(&bg_binding_entry(0, &buf_resource(&buffers.uniform_buf)));
    entries.push(&bg_binding_entry(1, &buf_resource(&buffers.elem_buf)));
    entries.push(&bg_binding_entry(2, &buf_resource(&buffers.particle_buf)));
    entries.push(&bg_binding_entry(3, &buf_resource(&buffers.palette_buf)));
    let desc = Object::new();
    set_prop(&desc, "layout",  layout);
    set_prop(&desc, "entries", &entries.into());
    get_fn(device, "createBindGroup")?.call1(device, &desc.into()).ok()
}

pub(super) fn mk_render_bind_group(
    device: &JsValue, layout: &JsValue, buffers: &GpuBuffers,
) -> Option<JsValue> {
    let entries = Array::new();
    entries.push(&bg_binding_entry(0, &buf_resource(&buffers.uniform_buf)));
    entries.push(&bg_binding_entry(1, &buf_resource(&buffers.elem_buf)));
    entries.push(&bg_binding_entry(2, &buf_resource(&buffers.particle_buf)));
    entries.push(&bg_binding_entry(3, &buf_resource(&buffers.palette_buf)));
    let desc = Object::new();
    set_prop(&desc, "layout",  layout);
    set_prop(&desc, "entries", &entries.into());
    get_fn(device, "createBindGroup")?.call1(device, &desc.into()).ok()
}

// ── Default palette (dark theme) ─────────────────────────────────────────────

/// Build a default dark-theme palette `Float32Array` (24 × vec4f = 384 bytes).
///
/// Mirrors the `DARK` preset from `theme.rs`. When the live theme system is
/// wired up this can be replaced with a runtime lookup.
fn default_palette_f32() -> Float32Array {
    let buf = Float32Array::new_with_length(96);
    let mut i = 0u32;
    let mut w = |r: f32, g: f32, b: f32, a: f32| {
        buf.set_index(i,     r);
        buf.set_index(i + 1, g);
        buf.set_index(i + 2, b);
        buf.set_index(i + 3, a);
        i += 4;
    };
    // [0]  spark_core   — hot white-yellow
    w(1.0,  0.97, 0.85, 1.0);
    // [1]  spark_ember  — outer ember glow
    w(1.0,  0.4,  0.05, 1.0);
    // [2]  spark_steel  — metallic highlight
    w(0.7,  0.75, 0.85, 1.0);
    // [3]  ember_hot    — bright hot center
    w(1.0,  0.6,  0.1,  1.0);
    // [4]  beam_center  — golden-white core
    w(1.0,  0.98, 0.88, 1.0);
    // [5]  beam_edge    — warm gold edge
    w(1.0,  0.78, 0.2,  1.0);
    // [6]  glitter_warm — golden-white base
    w(1.0,  0.95, 0.7,  1.0);
    // [7]  glitter_cool — blue-white variation
    w(0.7,  0.85, 1.0,  1.0);
    // [8]  cinder_ember — deep orange-red
    w(0.7,  0.15, 0.02, 1.0);
    // [9]  cinder_gold  — tarnished gold
    w(0.6,  0.45, 0.05, 1.0);
    // [10] cinder_ash   — cool grey
    w(0.35, 0.33, 0.32, 1.0);
    // [11] cinder_vine  — deep green
    w(0.05, 0.22, 0.05, 1.0);
    // [12] smoke_cool   — blue-grey (brightened so smoke is visible against
    //                                #0a0a0c shell background)
    w(0.28, 0.34, 0.50, 1.0);
    // [13] smoke_warm   — brown-amber
    w(0.45, 0.30, 0.12, 1.0);
    // [14] smoke_moss   — mossy mid-tone
    w(0.18, 0.32, 0.16, 1.0);
    // [15..22] kind glow colors (structural, error, warn, info, debug, span, selected, panic)
    w(0.18, 0.16, 0.14, 1.0); // kind_structural
    w(0.97, 0.47, 0.55, 1.0); // kind_error
    w(0.88, 0.68, 0.41, 1.0); // kind_warn
    w(0.48, 0.81, 0.64, 1.0); // kind_info
    w(0.48, 0.60, 0.97, 1.0); // kind_debug
    w(0.61, 0.80, 0.41, 1.0); // kind_span
    w(1.0,  0.62, 0.39, 1.0); // kind_selected
    w(0.97, 0.47, 0.55, 1.0); // kind_panic
    // [23] _pad
    w(0.0,  0.0,  0.0,  0.0);
    buf
}
