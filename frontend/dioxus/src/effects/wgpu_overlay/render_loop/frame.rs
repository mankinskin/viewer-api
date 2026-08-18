use js_sys::{
    Array,
    Float32Array,
};
use tracing::debug;
use wasm_bindgen::{
    JsCast,
    JsValue,
};
use web_sys::{
    HtmlCanvasElement,
    Window,
};

use super::{
    super::{
        element_scanner::scan_ui_rects,
        element_types::*,
        invoke_frame_callbacks,
        live_effects,
        mouse_pos,
        settings::EffectSettings,
        take_palette_dirty,
        webgpu::*,
        FrameContext,
    },
    uniforms::{
        find_hovered_elem,
        find_selected_elem,
        pack_uniforms,
    },
    GpuCtx,
};

pub(super) fn render_frame(
    gpu: &mut GpuCtx,
    ts_ms: f64,
    win: &Window,
) {
    let (time_s, dt_s) = update_frame_timing(gpu, ts_ms);
    let settings = live_effects();

    maybe_upload_palette(gpu, &settings);
    maybe_log_frame(gpu, time_s, dt_s, &settings);

    let Some((document, cw, ch)) = resize_canvas_and_depth(gpu, win) else {
        return;
    };

    let (elem_data, elem_count) = upload_scanned_elements(gpu, &document);
    write_uniforms(
        gpu, &settings, &document, &elem_data, elem_count, time_s, dt_s, cw, ch,
    );

    let Some((frame_view, encoder)) = begin_frame(gpu) else {
        return;
    };

    run_compute_pass(&encoder, gpu, &settings);
    run_render_pass(&encoder, gpu, &settings, &frame_view);
    submit_commands(gpu, &encoder);
    invoke_overlay_frame_callbacks(gpu, &frame_view, cw, ch, time_s);

    let _ = &gpu.depth_tex;
}

fn update_frame_timing(
    gpu: &mut GpuCtx,
    ts_ms: f64,
) -> (f32, f32) {
    let dt_s = ((ts_ms - gpu.prev_time_ms) / 1000.0).min(0.1) as f32;
    gpu.prev_time_ms = ts_ms;
    let time_s = ((ts_ms - gpu.start_time_ms) / 1000.0) as f32;
    (time_s, dt_s)
}

fn maybe_upload_palette(
    gpu: &GpuCtx,
    settings: &EffectSettings,
) {
    if !take_palette_dirty() {
        return;
    }

    let flat = settings.palette_flat();
    let view = unsafe { Float32Array::view(&flat) };
    queue_write_f32(&gpu.queue, &gpu.buffers.palette_buf, 0, &view);
}

fn maybe_log_frame(
    gpu: &GpuCtx,
    time_s: f32,
    dt_s: f32,
    settings: &EffectSettings,
) {
    thread_local! { static FRAME_NO: std::cell::Cell<u32> = const { std::cell::Cell::new(0) }; }
    FRAME_NO.with(|counter| {
        let frame_no = counter.get().wrapping_add(1);
        counter.set(frame_no);
        if frame_no == 1 || frame_no.is_multiple_of(120) {
            let device_label =
                js_sys::Reflect::get(&gpu.device, &"label".into())
                    .ok()
                    .and_then(|value| value.as_string())
                    .unwrap_or_default();
            debug!(
                target: "wgpu_overlay::frame",
                frame_n = frame_no,
                frame_time_s = time_s,
                frame_dt_s = dt_s,
                smoke = settings.smoke_intensity,
                device_label = %device_label,
                "frame"
            );
        }
    });
}

fn resize_canvas_and_depth(
    gpu: &mut GpuCtx,
    win: &Window,
) -> Option<(web_sys::Document, u32, u32)> {
    let document = win.document()?;
    let canvas = document
        .get_element_by_id("webgpu-canvas")
        .and_then(|element| element.dyn_into::<HtmlCanvasElement>().ok())?;

    let dpr = win.device_pixel_ratio();
    let cw = ((canvas.client_width() as f64 * dpr) as u32).max(1);
    let ch = ((canvas.client_height() as f64 * dpr) as u32).max(1);
    if cw != canvas.width() {
        canvas.set_width(cw);
    }
    if ch != canvas.height() {
        canvas.set_height(ch);
    }

    if cw != gpu.depth_w || ch != gpu.depth_h {
        if let Some((depth_tex, depth_view)) =
            create_depth_texture(&gpu.device, cw, ch)
        {
            gpu.depth_tex = depth_tex;
            gpu.depth_view = depth_view;
            gpu.depth_w = cw;
            gpu.depth_h = ch;
        }
    }

    Some((document, cw, ch))
}

fn upload_scanned_elements(
    gpu: &mut GpuCtx,
    document: &web_sys::Document,
) -> (Vec<f32>, usize) {
    let (elem_data, elem_count) = scan_ui_rects(document);
    if gpu.buffers.ensure_elem_capacity(&gpu.device, elem_count) {
        if let Some(bind_group) = super::mk_compute_bind_group(
            &gpu.device,
            &gpu.pipelines.compute_bgl,
            &gpu.buffers,
        ) {
            gpu.compute_bg = bind_group;
        }
        if let Some(bind_group) = super::mk_render_bind_group(
            &gpu.device,
            &gpu.pipelines.render_bgl,
            &gpu.buffers,
        ) {
            gpu.render_bg = bind_group;
        }
    }

    if !elem_data.is_empty() {
        let view = unsafe { Float32Array::view(&elem_data) };
        queue_write_f32(&gpu.queue, &gpu.buffers.elem_buf, 0, &view);
    }

    (elem_data, elem_count)
}

fn write_uniforms(
    gpu: &GpuCtx,
    settings: &EffectSettings,
    document: &web_sys::Document,
    elem_data: &[f32],
    elem_count: usize,
    time_s: f32,
    dt_s: f32,
    cw: u32,
    ch: u32,
) {
    let (mx, my) = mouse_pos();
    let hover_elem = find_hovered_elem(elem_data, elem_count, mx, my);
    let selected_elem = find_selected_elem(elem_data, elem_count, document);
    pack_uniforms(
        gpu,
        settings,
        time_s,
        dt_s,
        cw,
        ch,
        elem_count,
        mx,
        my,
        hover_elem,
        selected_elem,
    );
    queue_write_f32(&gpu.queue, &gpu.buffers.uniform_buf, 0, &gpu.uniforms_f32);
}

fn begin_frame(gpu: &GpuCtx) -> Option<(JsValue, JsValue)> {
    let frame_tex = get_fn(&gpu.context, "getCurrentTexture")
        .and_then(|function| function.call0(&gpu.context).ok())?;
    let frame_view = create_tex_view(&frame_tex)?;
    let encoder = get_fn(&gpu.device, "createCommandEncoder")
        .and_then(|function| function.call0(&gpu.device).ok())?;
    Some((frame_view, encoder))
}

fn run_compute_pass(
    encoder: &JsValue,
    gpu: &GpuCtx,
    settings: &EffectSettings,
) {
    if !settings.particles_enabled {
        return;
    }

    let Some(pass) = get_fn(encoder, "beginComputePass")
        .and_then(|function| function.call0(encoder).ok())
    else {
        return;
    };

    call_set_pipeline(&pass, &gpu.pipelines.compute_pipeline);
    call_set_bind_group(&pass, 0, &gpu.compute_bg);
    let workgroups =
        ((NUM_PARTICLES + COMPUTE_WORKGROUP - 1) / COMPUTE_WORKGROUP) as u32;
    call_dispatch(&pass, workgroups);
    call_end(&pass);
}

fn run_render_pass(
    encoder: &JsValue,
    gpu: &GpuCtx,
    settings: &EffectSettings,
    frame_view: &JsValue,
) {
    let render_pass_desc = build_render_pass_desc(frame_view, &gpu.depth_view);
    let Some(pass) = get_fn(encoder, "beginRenderPass")
        .and_then(|function| function.call1(encoder, &render_pass_desc).ok())
    else {
        return;
    };

    call_set_pipeline(&pass, &gpu.pipelines.bg_pipeline);
    call_set_bind_group(&pass, 0, &gpu.render_bg);
    call_draw(&pass, 6, 1);

    if settings.particles_enabled {
        call_set_pipeline(&pass, &gpu.pipelines.particle_pipeline);
        call_set_bind_group(&pass, 0, &gpu.render_bg);
        call_draw(&pass, 6, NUM_PARTICLES as u32);
    }

    call_end(&pass);
}

fn submit_commands(
    gpu: &GpuCtx,
    encoder: &JsValue,
) {
    let Some(finish) = get_fn(encoder, "finish")
        .and_then(|function| function.call0(encoder).ok())
    else {
        return;
    };
    let Some(submit) = get_fn(&gpu.queue, "submit") else {
        return;
    };

    let commands = Array::new();
    commands.push(&finish);
    let _ = submit.call1(&gpu.queue, &commands);
}

fn invoke_overlay_frame_callbacks(
    gpu: &GpuCtx,
    frame_view: &JsValue,
    cw: u32,
    ch: u32,
    time_s: f32,
) {
    let frame_ctx = FrameContext {
        device: &gpu.device,
        queue: &gpu.queue,
        frame_view,
        canvas_w: cw,
        canvas_h: ch,
        time_s,
    };
    invoke_frame_callbacks(&frame_ctx);
}
