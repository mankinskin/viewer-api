#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

//! Comprehensive shader-effect settings for the [`WgpuOverlay`].
//!
//! Owns every tunable scalar that the smoke / particle / CRT shaders read
//! from the uniform buffer, plus the 24-entry colour palette.  Persisted to
//! `localStorage` as a flat key=value JSON-ish blob (no serde dependency).
//!
//! This module is **not** WASM-gated so the Dioxus components can construct,
//! diff, and edit `EffectSettings` on any target.  WASM-only side effects
//! (localStorage I/O) are gated internally with `cfg(target_arch = "wasm32")`.

mod palette;

const STORAGE_KEY: &str = "viewer-api-effects";

/// Number of palette colours uploaded to the GPU as a `[vec4f; 24]` uniform.
///
/// Mirrors `PALETTE_VEC4_COUNT` in [`super::element_types`].  Kept duplicated
/// here so this module stays usable on non-WASM targets where the shader
/// constants are not compiled in.
pub const PALETTE_LEN: usize = 24;

/// RGBA colour stored as floats in 0..1.  Mirrors WGSL `vec4f`.
pub type PaletteColor = [f32; 4];

pub use self::palette::{
    hex_to_rgba,
    rgba_to_hex,
    PALETTE_LABELS,
};

use self::palette::default_palette;

// ─────────────────────────────────────────────────────────────────────────────
// EffectSettings
// ─────────────────────────────────────────────────────────────────────────────

/// Every tunable shader uniform plus the colour palette.
///
/// Field grouping mirrors the **Theme Settings** UI sections so add/remove
/// flows stay obvious.  Defaults match the hand-tuned values previously
/// hard-coded into `pack_uniforms` so existing visuals are preserved when no
/// saved settings exist.
#[derive(Clone, PartialEq, Debug)]
pub struct EffectSettings {
    // ── Master flags ────────────────────────────────────────────────────────
    pub smoke_enabled: bool,
    pub particles_enabled: bool,
    pub crt_enabled: bool,
    pub grain_enabled: bool,
    pub vignette_enabled: bool,

    // ── Smoke ────────────────────────────────────────────────────────────────
    pub smoke_intensity: f32,
    pub smoke_speed: f32,
    pub smoke_warm_scale: f32,
    pub smoke_cool_scale: f32,
    pub smoke_moss_scale: f32,

    // ── CRT ──────────────────────────────────────────────────────────────────
    pub crt_scanlines_h: f32,
    pub crt_scanlines_v: f32,
    pub crt_edge_shadow: f32,
    pub crt_flicker: f32,
    pub crt_line_width: f32,
    pub crt_color: PaletteColor,

    // ── Grain / vignette / underglow ─────────────────────────────────────────
    pub grain_intensity: f32,
    pub grain_coarseness: f32,
    pub grain_size: f32,
    pub vignette_strength: f32,
    pub underglow_strength: f32,

    // ── Particles (per-type tuning) ──────────────────────────────────────────
    pub spark_speed: f32,
    pub spark_size: f32,
    pub spark_count: f32, // 0..1 multiplier on global spark count
    pub ember_speed: f32,
    pub ember_size: f32,
    pub ember_count: f32,
    pub beam_speed: f32,
    pub beam_size: f32, // size proxy via cinder_size for cinder/beam
    pub beam_count: f32,
    pub beam_height: f32,
    pub beam_drift: f32,
    pub glitter_speed: f32,
    pub glitter_size: f32,
    pub glitter_count: f32,
    pub cinder_size: f32,

    // ── Palette (24 RGBA colours) ────────────────────────────────────────────
    pub palette: [PaletteColor; PALETTE_LEN],
}

impl Default for EffectSettings {
    fn default() -> Self {
        Self {
            smoke_enabled: true,
            particles_enabled: true,
            crt_enabled: true,
            grain_enabled: true,
            vignette_enabled: true,

            smoke_intensity: 0.6,
            smoke_speed: 1.0,
            smoke_warm_scale: 1.0,
            smoke_cool_scale: 1.0,
            smoke_moss_scale: 1.0,

            crt_scanlines_h: 0.15,
            crt_scanlines_v: 0.0,
            crt_edge_shadow: 0.4,
            crt_flicker: 0.08,
            crt_line_width: 0.3,
            crt_color: [0.9, 0.7, 0.4, 1.0],

            grain_intensity: 0.15,
            grain_coarseness: 0.5,
            grain_size: 0.3,
            vignette_strength: 0.5,
            underglow_strength: 0.2,

            spark_speed: 1.0,
            spark_size: 1.0,
            spark_count: 1.0,
            ember_speed: 1.0,
            ember_size: 1.0,
            ember_count: 1.0,
            beam_speed: 1.0,
            beam_size: 1.0,
            beam_count: 0.0,
            beam_height: 35.0,
            beam_drift: 1.0,
            glitter_speed: 1.0,
            glitter_size: 1.0,
            glitter_count: 1.0,
            cinder_size: 1.0,

            palette: default_palette(),
        }
    }
}

impl EffectSettings {
    /// Load committed settings from `localStorage`; falls back to defaults.
    ///
    /// On non-WASM builds always returns [`EffectSettings::default`].
    pub fn load() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let raw = web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
                .and_then(|s| s.get_item(STORAGE_KEY).ok().flatten());
            if let Some(json) = raw {
                return Self::from_storage_string(&json);
            }
        }
        Self::default()
    }

    /// Persist settings to `localStorage`.  No-op on non-WASM builds.
    pub fn save(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(storage) =
                web_sys::window().and_then(|w| w.local_storage().ok().flatten())
            {
                let _ =
                    storage.set_item(STORAGE_KEY, &self.to_storage_string());
            }
        }
    }

    /// Format: `key1=value1\nkey2=value2\n...` with palette stored as
    /// `palette_<idx>=r,g,b,a`.  Avoids JSON quoting headaches.
    pub fn to_storage_string(&self) -> String {
        let mut out = String::with_capacity(2048);
        macro_rules! kv {
            ($k:literal, $v:expr) => {{
                out.push_str($k);
                out.push('=');
                out.push_str(&$v.to_string());
                out.push('\n');
            }};
        }
        kv!("smoke_enabled", self.smoke_enabled);
        kv!("particles_enabled", self.particles_enabled);
        kv!("crt_enabled", self.crt_enabled);
        kv!("grain_enabled", self.grain_enabled);
        kv!("vignette_enabled", self.vignette_enabled);

        kv!("smoke_intensity", self.smoke_intensity);
        kv!("smoke_speed", self.smoke_speed);
        kv!("smoke_warm_scale", self.smoke_warm_scale);
        kv!("smoke_cool_scale", self.smoke_cool_scale);
        kv!("smoke_moss_scale", self.smoke_moss_scale);

        kv!("crt_scanlines_h", self.crt_scanlines_h);
        kv!("crt_scanlines_v", self.crt_scanlines_v);
        kv!("crt_edge_shadow", self.crt_edge_shadow);
        kv!("crt_flicker", self.crt_flicker);
        kv!("crt_line_width", self.crt_line_width);
        out.push_str(&format!(
            "crt_color={},{},{},{}\n",
            self.crt_color[0],
            self.crt_color[1],
            self.crt_color[2],
            self.crt_color[3]
        ));

        kv!("grain_intensity", self.grain_intensity);
        kv!("grain_coarseness", self.grain_coarseness);
        kv!("grain_size", self.grain_size);
        kv!("vignette_strength", self.vignette_strength);
        kv!("underglow_strength", self.underglow_strength);

        kv!("spark_speed", self.spark_speed);
        kv!("spark_size", self.spark_size);
        kv!("spark_count", self.spark_count);
        kv!("ember_speed", self.ember_speed);
        kv!("ember_size", self.ember_size);
        kv!("ember_count", self.ember_count);
        kv!("beam_speed", self.beam_speed);
        kv!("beam_size", self.beam_size);
        kv!("beam_count", self.beam_count);
        kv!("beam_height", self.beam_height);
        kv!("beam_drift", self.beam_drift);
        kv!("glitter_speed", self.glitter_speed);
        kv!("glitter_size", self.glitter_size);
        kv!("glitter_count", self.glitter_count);
        kv!("cinder_size", self.cinder_size);

        for (i, c) in self.palette.iter().enumerate() {
            out.push_str(&format!(
                "palette_{}={},{},{},{}\n",
                i, c[0], c[1], c[2], c[3]
            ));
        }
        out
    }

    pub fn from_storage_string(s: &str) -> Self {
        let mut out = Self::default();
        for line in s.lines() {
            apply_storage_line(&mut out, line);
        }
        out
    }

    /// Pack the palette as a flat `[f32; PALETTE_LEN * 4]` for GPU upload.
    pub fn palette_flat(&self) -> [f32; PALETTE_LEN * 4] {
        let mut out = [0.0f32; PALETTE_LEN * 4];
        for (i, c) in self.palette.iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(c);
        }
        out
    }
}

fn parse_into(
    v: &str,
    dst: &mut f32,
) {
    if let Ok(f) = v.parse() {
        *dst = f;
    }
}

fn apply_storage_line(
    settings: &mut EffectSettings,
    line: &str,
) {
    let Some((key, value)) = line.split_once('=') else {
        return;
    };
    let key = key.trim();
    let value = value.trim();

    if apply_flag_setting(settings, key, value)
        || apply_smoke_setting(settings, key, value)
        || apply_crt_setting(settings, key, value)
        || apply_post_processing_setting(settings, key, value)
        || apply_particle_setting_group_a(settings, key, value)
        || apply_particle_setting_group_b(settings, key, value)
    {
        return;
    }

    let _ = apply_palette_setting(settings, key, value);
}

fn apply_flag_setting(
    settings: &mut EffectSettings,
    key: &str,
    value: &str,
) -> bool {
    match key {
        "smoke_enabled" => settings.smoke_enabled = value == "true",
        "particles_enabled" => settings.particles_enabled = value == "true",
        "crt_enabled" => settings.crt_enabled = value == "true",
        "grain_enabled" => settings.grain_enabled = value == "true",
        "vignette_enabled" => settings.vignette_enabled = value == "true",
        _ => return false,
    }
    true
}

fn apply_smoke_setting(
    settings: &mut EffectSettings,
    key: &str,
    value: &str,
) -> bool {
    match key {
        "smoke_intensity" => parse_into(value, &mut settings.smoke_intensity),
        "smoke_speed" => parse_into(value, &mut settings.smoke_speed),
        "smoke_warm_scale" => parse_into(value, &mut settings.smoke_warm_scale),
        "smoke_cool_scale" => parse_into(value, &mut settings.smoke_cool_scale),
        "smoke_moss_scale" => parse_into(value, &mut settings.smoke_moss_scale),
        _ => return false,
    }
    true
}

fn apply_crt_setting(
    settings: &mut EffectSettings,
    key: &str,
    value: &str,
) -> bool {
    match key {
        "crt_scanlines_h" => parse_into(value, &mut settings.crt_scanlines_h),
        "crt_scanlines_v" => parse_into(value, &mut settings.crt_scanlines_v),
        "crt_edge_shadow" => parse_into(value, &mut settings.crt_edge_shadow),
        "crt_flicker" => parse_into(value, &mut settings.crt_flicker),
        "crt_line_width" => parse_into(value, &mut settings.crt_line_width),
        "crt_color" => parse_color(value, &mut settings.crt_color),
        _ => return false,
    }
    true
}

fn apply_post_processing_setting(
    settings: &mut EffectSettings,
    key: &str,
    value: &str,
) -> bool {
    match key {
        "grain_intensity" => parse_into(value, &mut settings.grain_intensity),
        "grain_coarseness" => parse_into(value, &mut settings.grain_coarseness),
        "grain_size" => parse_into(value, &mut settings.grain_size),
        "vignette_strength" =>
            parse_into(value, &mut settings.vignette_strength),
        "underglow_strength" =>
            parse_into(value, &mut settings.underglow_strength),
        _ => return false,
    }
    true
}

fn apply_particle_setting_group_a(
    settings: &mut EffectSettings,
    key: &str,
    value: &str,
) -> bool {
    match key {
        "spark_speed" => parse_into(value, &mut settings.spark_speed),
        "spark_size" => parse_into(value, &mut settings.spark_size),
        "spark_count" => parse_into(value, &mut settings.spark_count),
        "ember_speed" => parse_into(value, &mut settings.ember_speed),
        "ember_size" => parse_into(value, &mut settings.ember_size),
        "ember_count" => parse_into(value, &mut settings.ember_count),
        "beam_speed" => parse_into(value, &mut settings.beam_speed),
        _ => return false,
    }
    true
}

fn apply_particle_setting_group_b(
    settings: &mut EffectSettings,
    key: &str,
    value: &str,
) -> bool {
    match key {
        "beam_size" => parse_into(value, &mut settings.beam_size),
        "beam_count" => parse_into(value, &mut settings.beam_count),
        "beam_height" => parse_into(value, &mut settings.beam_height),
        "beam_drift" => parse_into(value, &mut settings.beam_drift),
        "glitter_speed" => parse_into(value, &mut settings.glitter_speed),
        "glitter_size" => parse_into(value, &mut settings.glitter_size),
        "glitter_count" => parse_into(value, &mut settings.glitter_count),
        "cinder_size" => parse_into(value, &mut settings.cinder_size),
        _ => return false,
    }
    true
}

fn apply_palette_setting(
    settings: &mut EffectSettings,
    key: &str,
    value: &str,
) -> bool {
    let Some(index) = key
        .strip_prefix("palette_")
        .and_then(|suffix| suffix.parse::<usize>().ok())
    else {
        return false;
    };

    if index < PALETTE_LEN {
        parse_color(value, &mut settings.palette[index]);
    }

    true
}

fn parse_color(
    v: &str,
    dst: &mut PaletteColor,
) {
    let parts: Vec<&str> = v.split(',').collect();
    for (i, p) in parts.iter().take(4).enumerate() {
        if let Ok(f) = p.trim().parse::<f32>() {
            dst[i] = f;
        }
    }
}
