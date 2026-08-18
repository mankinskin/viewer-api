use super::{
    super::{
        settings::EffectSettings,
        webgpu::{
            get_fn,
            prop_f32,
        },
    },
    GpuCtx,
};

pub(super) fn pack_uniforms(
    gpu: &GpuCtx,
    settings: &EffectSettings,
    time_s: f32,
    dt_s: f32,
    cw: u32,
    ch: u32,
    elem_count: usize,
    mouse_x: f32,
    mouse_y: f32,
    hover_elem: f32,
    selected_elem: f32,
) {
    let uniforms = &gpu.uniforms_f32;
    let vp = ortho_vp(cw as f32, ch as f32);
    let iv = ortho_inv_vp(cw as f32, ch as f32);

    let smoke_gate = if settings.smoke_enabled { 1.0 } else { 0.0 };
    let crt_gate = if settings.crt_enabled { 1.0 } else { 0.0 };
    let grain_gate = if settings.grain_enabled { 1.0 } else { 0.0 };
    let vignette_gate = if settings.vignette_enabled { 1.0 } else { 0.0 };
    let particle_gate = if settings.particles_enabled { 1.0 } else { 0.0 };

    uniforms.set_index(0, time_s);
    uniforms.set_index(1, cw as f32);
    uniforms.set_index(2, ch as f32);
    uniforms.set_index(3, elem_count as f32);
    uniforms.set_index(4, mouse_x);
    uniforms.set_index(5, mouse_y);
    uniforms.set_index(6, dt_s);
    uniforms.set_index(7, hover_elem);
    uniforms.set_index(8, 0.0);
    uniforms.set_index(9, selected_elem);
    uniforms.set_index(10, settings.crt_scanlines_h * crt_gate);
    uniforms.set_index(11, settings.crt_scanlines_v * crt_gate);
    uniforms.set_index(12, settings.crt_edge_shadow * crt_gate);
    uniforms.set_index(13, settings.crt_flicker * crt_gate);
    uniforms.set_index(14, settings.crt_line_width);
    uniforms.set_index(15, settings.smoke_intensity * smoke_gate);
    uniforms.set_index(16, settings.smoke_speed);
    uniforms.set_index(17, settings.smoke_warm_scale);
    uniforms.set_index(18, settings.smoke_cool_scale);
    uniforms.set_index(19, settings.smoke_moss_scale);
    uniforms.set_index(20, settings.grain_intensity * grain_gate);
    uniforms.set_index(21, settings.grain_coarseness);
    uniforms.set_index(22, settings.grain_size);
    uniforms.set_index(23, settings.vignette_strength * vignette_gate);
    uniforms.set_index(24, settings.underglow_strength);
    uniforms.set_index(25, settings.spark_speed);
    uniforms.set_index(26, settings.ember_speed);
    uniforms.set_index(27, settings.beam_speed);
    uniforms.set_index(28, settings.glitter_speed);
    uniforms.set_index(29, settings.beam_height);
    uniforms.set_index(30, settings.beam_count * particle_gate);
    uniforms.set_index(31, settings.beam_drift);
    uniforms.set_index(32, 0.0);
    uniforms.set_index(33, 0.0);
    uniforms.set_index(34, settings.spark_count * particle_gate);
    uniforms.set_index(35, settings.spark_size);
    uniforms.set_index(36, settings.ember_count * particle_gate);
    uniforms.set_index(37, settings.ember_size);
    uniforms.set_index(38, settings.glitter_count * particle_gate);
    uniforms.set_index(39, settings.glitter_size);
    uniforms.set_index(40, settings.cinder_size);
    uniforms.set_index(41, 0.0);
    uniforms.set_index(42, 1.0);
    uniforms.set_index(43, 0.0);
    uniforms.set_index(44, 0.0);
    uniforms.set_index(45, cw as f32);
    uniforms.set_index(46, ch as f32);
    uniforms.set_index(47, 0.0);
    uniforms.set_index(48, settings.crt_color[0]);
    uniforms.set_index(49, settings.crt_color[1]);
    uniforms.set_index(50, settings.crt_color[2]);
    uniforms.set_index(51, 0.0);
    uniforms.set_index(52, 0.0);
    uniforms.set_index(53, 0.0);
    uniforms.set_index(54, 0.0);
    uniforms.set_index(55, 0.0);
    for (index, &value) in vp.iter().enumerate() {
        uniforms.set_index(56 + index as u32, value);
    }
    for (index, &value) in iv.iter().enumerate() {
        uniforms.set_index(72 + index as u32, value);
    }
}

pub(super) fn find_hovered_elem(
    elem_data: &[f32],
    elem_count: usize,
    mx: f32,
    my: f32,
) -> f32 {
    if mx < -999.0 {
        return -1.0;
    }

    for index in 0..elem_count {
        let base = index * 8;
        let x = elem_data[base];
        let y = elem_data[base + 1];
        let w = elem_data[base + 2];
        let h = elem_data[base + 3];
        if mx >= x && mx <= x + w && my >= y && my <= y + h {
            return index as f32;
        }
    }
    -1.0
}

pub(super) fn find_selected_elem(
    elem_data: &[f32],
    elem_count: usize,
    document: &web_sys::Document,
) -> f32 {
    let selected_element = document
        .query_selector(
            ".log-entry.selected, .spec-card--selected, [aria-selected=true]",
        )
        .ok()
        .flatten();
    let selected_element = match selected_element {
        Some(element) => element,
        None => return -1.0,
    };

    let rect = match get_fn(&selected_element, "getBoundingClientRect")
        .and_then(|function| function.call0(&selected_element).ok())
    {
        Some(rect) => rect,
        None => return -1.0,
    };
    let sx = prop_f32(&rect, "x");
    let sy = prop_f32(&rect, "y");

    for index in 0..elem_count {
        let base = index * 8;
        let x = elem_data[base];
        let y = elem_data[base + 1];
        if (x - sx).abs() < 2.0 && (y - sy).abs() < 2.0 {
            return index as f32;
        }
    }
    -1.0
}

fn ortho_vp(
    w: f32,
    h: f32,
) -> [f32; 16] {
    [
        2.0 / w,
        0.0,
        0.0,
        0.0,
        0.0,
        -2.0 / h,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        -1.0,
        1.0,
        0.0,
        1.0,
    ]
}

fn ortho_inv_vp(
    w: f32,
    h: f32,
) -> [f32; 16] {
    [
        w / 2.0,
        0.0,
        0.0,
        0.0,
        0.0,
        -h / 2.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        w / 2.0,
        h / 2.0,
        0.0,
        1.0,
    ]
}
