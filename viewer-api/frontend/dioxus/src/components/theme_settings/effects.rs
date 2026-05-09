use dioxus::prelude::*;

use crate::effects::wgpu_overlay::{
    hex_to_rgba, rgba_to_hex, EffectSettings, PaletteColor, PALETTE_LABELS, PALETTE_LEN,
};

#[component]
fn EffectSlider(
    label: String,
    min: f32,
    max: f32,
    step: f32,
    value: f32,
    on_change: EventHandler<f32>,
) -> Element {
    rsx! {
        label {
            class: "theme-settings__slider-row",
            span { class: "theme-settings__slider-label", "{label}" }
            input {
                r#type: "range",
                class: "theme-settings__slider",
                min: "{min}", max: "{max}", step: "{step}",
                value: "{value}",
                aria_label: "{label}",
                oninput: move |event| {
                    if let Ok(value) = event.value().parse::<f32>() {
                        on_change.call(value);
                    }
                },
            }
            span { class: "theme-settings__slider-value", "{value:.2}" }
        }
    }
}

#[component]
fn PaletteSwatch(
    label: String,
    hint: String,
    value: PaletteColor,
    on_change: EventHandler<PaletteColor>,
) -> Element {
    let hex = rgba_to_hex(value);
    rsx! {
        label {
            class: "theme-settings__token-row",
            span { class: "theme-settings__token-label", title: "{hint}", "{label}" }
            span {
                class: "theme-settings__token-swatch",
                style: "background: {hex};",
            }
            input {
                r#type: "color",
                class: "theme-settings__color-input",
                value: "{hex}",
                aria_label: "{label}",
                oninput: move |event| {
                    if let Some(rgba) = hex_to_rgba(&event.value()) {
                        on_change.call(rgba);
                    }
                },
            }
        }
    }
}

#[component]
pub(super) fn EffectControls(mut draft: Signal<EffectSettings>) -> Element {
    rsx! {
        details { open: true,
            summary { class: "theme-settings__effect-summary", "Background smoke" }
            label {
                class: "theme-settings__effect-row",
                input {
                    r#type: "checkbox",
                    checked: draft.read().smoke_enabled,
                    onchange: move |event: Event<FormData>| {
                        draft.write().smoke_enabled = event.value() == "true";
                    },
                }
                span { "Enabled" }
            }
            EffectSlider { label: "Intensity".to_string(), min: 0.0, max: 1.5, step: 0.01, value: draft.read().smoke_intensity, on_change: move |value| draft.write().smoke_intensity = value }
            EffectSlider { label: "Speed".to_string(), min: 0.0, max: 4.0, step: 0.05, value: draft.read().smoke_speed, on_change: move |value| draft.write().smoke_speed = value }
            EffectSlider { label: "Warm scale".to_string(), min: 0.0, max: 3.0, step: 0.05, value: draft.read().smoke_warm_scale, on_change: move |value| draft.write().smoke_warm_scale = value }
            EffectSlider { label: "Cool scale".to_string(), min: 0.0, max: 3.0, step: 0.05, value: draft.read().smoke_cool_scale, on_change: move |value| draft.write().smoke_cool_scale = value }
            EffectSlider { label: "Moss scale".to_string(), min: 0.0, max: 3.0, step: 0.05, value: draft.read().smoke_moss_scale, on_change: move |value| draft.write().smoke_moss_scale = value }
        }

        details {
            summary { class: "theme-settings__effect-summary", "CRT scanlines" }
            label {
                class: "theme-settings__effect-row",
                input {
                    r#type: "checkbox",
                    checked: draft.read().crt_enabled,
                    onchange: move |event: Event<FormData>| {
                        draft.write().crt_enabled = event.value() == "true";
                    },
                }
                span { "Enabled" }
            }
            EffectSlider { label: "Horizontal lines".to_string(), min: 0.0, max: 1.0, step: 0.01, value: draft.read().crt_scanlines_h, on_change: move |value| draft.write().crt_scanlines_h = value }
            EffectSlider { label: "Vertical lines".to_string(), min: 0.0, max: 1.0, step: 0.01, value: draft.read().crt_scanlines_v, on_change: move |value| draft.write().crt_scanlines_v = value }
            EffectSlider { label: "Edge shadow".to_string(), min: 0.0, max: 1.0, step: 0.01, value: draft.read().crt_edge_shadow, on_change: move |value| draft.write().crt_edge_shadow = value }
            EffectSlider { label: "Flicker".to_string(), min: 0.0, max: 0.5, step: 0.01, value: draft.read().crt_flicker, on_change: move |value| draft.write().crt_flicker = value }
            EffectSlider { label: "Line width".to_string(), min: 0.0, max: 1.0, step: 0.01, value: draft.read().crt_line_width, on_change: move |value| draft.write().crt_line_width = value }
            PaletteSwatch {
                label: "Tint colour".to_string(),
                hint: "Warm tint applied to the CRT overlay".to_string(),
                value: draft.read().crt_color,
                on_change: move |color| draft.write().crt_color = color,
            }
        }

        details {
            summary { class: "theme-settings__effect-summary", "Grain & vignette" }
            label {
                class: "theme-settings__effect-row",
                input {
                    r#type: "checkbox",
                    checked: draft.read().grain_enabled,
                    onchange: move |event: Event<FormData>| {
                        draft.write().grain_enabled = event.value() == "true";
                    },
                }
                span { "Grain enabled" }
            }
            EffectSlider { label: "Grain intensity".to_string(), min: 0.0, max: 0.5, step: 0.01, value: draft.read().grain_intensity, on_change: move |value| draft.write().grain_intensity = value }
            EffectSlider { label: "Grain coarseness".to_string(), min: 0.0, max: 2.0, step: 0.05, value: draft.read().grain_coarseness, on_change: move |value| draft.write().grain_coarseness = value }
            EffectSlider { label: "Grain size".to_string(), min: 0.0, max: 2.0, step: 0.05, value: draft.read().grain_size, on_change: move |value| draft.write().grain_size = value }
            label {
                class: "theme-settings__effect-row",
                input {
                    r#type: "checkbox",
                    checked: draft.read().vignette_enabled,
                    onchange: move |event: Event<FormData>| {
                        draft.write().vignette_enabled = event.value() == "true";
                    },
                }
                span { "Vignette enabled" }
            }
            EffectSlider { label: "Vignette strength".to_string(), min: 0.0, max: 1.5, step: 0.01, value: draft.read().vignette_strength, on_change: move |value| draft.write().vignette_strength = value }
            EffectSlider { label: "Underglow".to_string(), min: 0.0, max: 1.0, step: 0.01, value: draft.read().underglow_strength, on_change: move |value| draft.write().underglow_strength = value }
        }

        details {
            summary { class: "theme-settings__effect-summary", "Particles" }
            label {
                class: "theme-settings__effect-row",
                input {
                    r#type: "checkbox",
                    checked: draft.read().particles_enabled,
                    onchange: move |event: Event<FormData>| {
                        draft.write().particles_enabled = event.value() == "true";
                    },
                }
                span { "All particles enabled" }
            }

            h4 { class: "theme-settings__effect-subhead", "Sparks" }
            EffectSlider { label: "Speed".to_string(), min: 0.0, max: 3.0, step: 0.05, value: draft.read().spark_speed, on_change: move |value| draft.write().spark_speed = value }
            EffectSlider { label: "Size".to_string(), min: 0.0, max: 3.0, step: 0.05, value: draft.read().spark_size, on_change: move |value| draft.write().spark_size = value }
            EffectSlider { label: "Count".to_string(), min: 0.0, max: 1.0, step: 0.05, value: draft.read().spark_count, on_change: move |value| draft.write().spark_count = value }

            h4 { class: "theme-settings__effect-subhead", "Embers" }
            EffectSlider { label: "Speed".to_string(), min: 0.0, max: 3.0, step: 0.05, value: draft.read().ember_speed, on_change: move |value| draft.write().ember_speed = value }
            EffectSlider { label: "Size".to_string(), min: 0.0, max: 3.0, step: 0.05, value: draft.read().ember_size, on_change: move |value| draft.write().ember_size = value }
            EffectSlider { label: "Count".to_string(), min: 0.0, max: 1.0, step: 0.05, value: draft.read().ember_count, on_change: move |value| draft.write().ember_count = value }

            h4 { class: "theme-settings__effect-subhead", "Beams" }
            EffectSlider { label: "Speed".to_string(), min: 0.0, max: 3.0, step: 0.05, value: draft.read().beam_speed, on_change: move |value| draft.write().beam_speed = value }
            EffectSlider { label: "Count".to_string(), min: 0.0, max: 1.0, step: 0.05, value: draft.read().beam_count, on_change: move |value| draft.write().beam_count = value }
            EffectSlider { label: "Height".to_string(), min: 0.0, max: 200.0, step: 1.0, value: draft.read().beam_height, on_change: move |value| draft.write().beam_height = value }
            EffectSlider { label: "Drift".to_string(), min: 0.0, max: 5.0, step: 0.05, value: draft.read().beam_drift, on_change: move |value| draft.write().beam_drift = value }

            h4 { class: "theme-settings__effect-subhead", "Glitter" }
            EffectSlider { label: "Speed".to_string(), min: 0.0, max: 3.0, step: 0.05, value: draft.read().glitter_speed, on_change: move |value| draft.write().glitter_speed = value }
            EffectSlider { label: "Size".to_string(), min: 0.0, max: 3.0, step: 0.05, value: draft.read().glitter_size, on_change: move |value| draft.write().glitter_size = value }
            EffectSlider { label: "Count".to_string(), min: 0.0, max: 1.0, step: 0.05, value: draft.read().glitter_count, on_change: move |value| draft.write().glitter_count = value }

            h4 { class: "theme-settings__effect-subhead", "Cinder" }
            EffectSlider { label: "Cinder size".to_string(), min: 0.0, max: 3.0, step: 0.05, value: draft.read().cinder_size, on_change: move |value| draft.write().cinder_size = value }
        }

        details {
            summary { class: "theme-settings__effect-summary", "Palette ({PALETTE_LEN} colours)" }
            div {
                class: "theme-settings__token-grid",
                for idx in 0..PALETTE_LEN {
                    {
                        let (label, hint) = PALETTE_LABELS[idx];
                        let colour = draft.read().palette[idx];
                        rsx! {
                            PaletteSwatch {
                                key: "pal-{idx}",
                                label: label.to_string(),
                                hint: hint.to_string(),
                                value: colour,
                                on_change: move |color: PaletteColor| draft.write().palette[idx] = color,
                            }
                        }
                    }
                }
            }
        }
    }
}