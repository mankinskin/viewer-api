//! Comprehensive shader-effect settings for the [`WgpuOverlay`].
//!
//! Owns every tunable scalar that the smoke / particle / CRT shaders read
//! from the uniform buffer, plus the 24-entry colour palette.  Persisted to
//! `localStorage` as a flat key=value JSON-ish blob (no serde dependency).
//!
//! This module is **not** WASM-gated so the Dioxus components can construct,
//! diff, and edit `EffectSettings` on any target.  WASM-only side effects
//! (localStorage I/O) are gated internally with `cfg(target_arch = "wasm32")`.

const STORAGE_KEY: &str = "viewer-api-effects";

/// Number of palette colours uploaded to the GPU as a `[vec4f; 24]` uniform.
///
/// Mirrors `PALETTE_VEC4_COUNT` in [`super::element_types`].  Kept duplicated
/// here so this module stays usable on non-WASM targets where the shader
/// constants are not compiled in.
pub const PALETTE_LEN: usize = 24;

/// RGBA colour stored as floats in 0..1.  Mirrors WGSL `vec4f`.
pub type PaletteColor = [f32; 4];

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
    pub smoke_enabled:    bool,
    pub particles_enabled: bool,
    pub crt_enabled:      bool,
    pub grain_enabled:    bool,
    pub vignette_enabled: bool,

    // ── Smoke ────────────────────────────────────────────────────────────────
    pub smoke_intensity:  f32,
    pub smoke_speed:      f32,
    pub smoke_warm_scale: f32,
    pub smoke_cool_scale: f32,
    pub smoke_moss_scale: f32,

    // ── CRT ──────────────────────────────────────────────────────────────────
    pub crt_scanlines_h: f32,
    pub crt_scanlines_v: f32,
    pub crt_edge_shadow: f32,
    pub crt_flicker:     f32,
    pub crt_line_width:  f32,
    pub crt_color:       PaletteColor,

    // ── Grain / vignette / underglow ─────────────────────────────────────────
    pub grain_intensity:   f32,
    pub grain_coarseness:  f32,
    pub grain_size:        f32,
    pub vignette_strength: f32,
    pub underglow_strength: f32,

    // ── Particles (per-type tuning) ──────────────────────────────────────────
    pub spark_speed: f32,
    pub spark_size:  f32,
    pub spark_count: f32, // 0..1 multiplier on global spark count
    pub ember_speed: f32,
    pub ember_size:  f32,
    pub ember_count: f32,
    pub beam_speed:  f32,
    pub beam_size:   f32, // size proxy via cinder_size for cinder/beam
    pub beam_count:  f32,
    pub beam_height: f32,
    pub beam_drift:  f32,
    pub glitter_speed: f32,
    pub glitter_size:  f32,
    pub glitter_count: f32,
    pub cinder_size:   f32,

    // ── Palette (24 RGBA colours) ────────────────────────────────────────────
    pub palette: [PaletteColor; PALETTE_LEN],
}

impl Default for EffectSettings {
    fn default() -> Self {
        Self {
            smoke_enabled:     true,
            particles_enabled: true,
            crt_enabled:       true,
            grain_enabled:     true,
            vignette_enabled:  true,

            smoke_intensity:  0.6,
            smoke_speed:      1.0,
            smoke_warm_scale: 1.0,
            smoke_cool_scale: 1.0,
            smoke_moss_scale: 1.0,

            crt_scanlines_h: 0.15,
            crt_scanlines_v: 0.0,
            crt_edge_shadow: 0.4,
            crt_flicker:     0.08,
            crt_line_width:  0.3,
            crt_color:       [0.9, 0.7, 0.4, 1.0],

            grain_intensity:    0.15,
            grain_coarseness:   0.5,
            grain_size:         0.3,
            vignette_strength:  0.5,
            underglow_strength: 0.2,

            spark_speed: 1.0, spark_size: 1.0, spark_count: 1.0,
            ember_speed: 1.0, ember_size: 1.0, ember_count: 1.0,
            beam_speed:  1.0, beam_size:  1.0, beam_count:  0.0,
            beam_height: 35.0, beam_drift: 1.0,
            glitter_speed: 1.0, glitter_size: 1.0, glitter_count: 1.0,
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
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
            {
                let _ = storage.set_item(STORAGE_KEY, &self.to_storage_string());
            }
        }
    }

    /// Format: `key1=value1\nkey2=value2\n...` with palette stored as
    /// `palette_<idx>=r,g,b,a`.  Avoids JSON quoting headaches.
    pub fn to_storage_string(&self) -> String {
        let mut out = String::with_capacity(2048);
        macro_rules! kv {
            ($k:literal, $v:expr) => {{ out.push_str($k); out.push('='); out.push_str(&$v.to_string()); out.push('\n'); }};
        }
        kv!("smoke_enabled",     self.smoke_enabled);
        kv!("particles_enabled", self.particles_enabled);
        kv!("crt_enabled",       self.crt_enabled);
        kv!("grain_enabled",     self.grain_enabled);
        kv!("vignette_enabled",  self.vignette_enabled);

        kv!("smoke_intensity",  self.smoke_intensity);
        kv!("smoke_speed",      self.smoke_speed);
        kv!("smoke_warm_scale", self.smoke_warm_scale);
        kv!("smoke_cool_scale", self.smoke_cool_scale);
        kv!("smoke_moss_scale", self.smoke_moss_scale);

        kv!("crt_scanlines_h", self.crt_scanlines_h);
        kv!("crt_scanlines_v", self.crt_scanlines_v);
        kv!("crt_edge_shadow", self.crt_edge_shadow);
        kv!("crt_flicker",     self.crt_flicker);
        kv!("crt_line_width",  self.crt_line_width);
        out.push_str(&format!(
            "crt_color={},{},{},{}\n",
            self.crt_color[0], self.crt_color[1], self.crt_color[2], self.crt_color[3]
        ));

        kv!("grain_intensity",    self.grain_intensity);
        kv!("grain_coarseness",   self.grain_coarseness);
        kv!("grain_size",         self.grain_size);
        kv!("vignette_strength",  self.vignette_strength);
        kv!("underglow_strength", self.underglow_strength);

        kv!("spark_speed", self.spark_speed);
        kv!("spark_size",  self.spark_size);
        kv!("spark_count", self.spark_count);
        kv!("ember_speed", self.ember_speed);
        kv!("ember_size",  self.ember_size);
        kv!("ember_count", self.ember_count);
        kv!("beam_speed",  self.beam_speed);
        kv!("beam_size",   self.beam_size);
        kv!("beam_count",  self.beam_count);
        kv!("beam_height", self.beam_height);
        kv!("beam_drift",  self.beam_drift);
        kv!("glitter_speed", self.glitter_speed);
        kv!("glitter_size",  self.glitter_size);
        kv!("glitter_count", self.glitter_count);
        kv!("cinder_size",   self.cinder_size);

        for (i, c) in self.palette.iter().enumerate() {
            out.push_str(&format!("palette_{}={},{},{},{}\n", i, c[0], c[1], c[2], c[3]));
        }
        out
    }

    pub fn from_storage_string(s: &str) -> Self {
        let mut out = Self::default();
        for line in s.lines() {
            let Some((k, v)) = line.split_once('=') else { continue; };
            let k = k.trim();
            let v = v.trim();
            match k {
                "smoke_enabled"     => out.smoke_enabled     = v == "true",
                "particles_enabled" => out.particles_enabled = v == "true",
                "crt_enabled"       => out.crt_enabled       = v == "true",
                "grain_enabled"     => out.grain_enabled     = v == "true",
                "vignette_enabled"  => out.vignette_enabled  = v == "true",

                "smoke_intensity"  => parse_into(v, &mut out.smoke_intensity),
                "smoke_speed"      => parse_into(v, &mut out.smoke_speed),
                "smoke_warm_scale" => parse_into(v, &mut out.smoke_warm_scale),
                "smoke_cool_scale" => parse_into(v, &mut out.smoke_cool_scale),
                "smoke_moss_scale" => parse_into(v, &mut out.smoke_moss_scale),

                "crt_scanlines_h" => parse_into(v, &mut out.crt_scanlines_h),
                "crt_scanlines_v" => parse_into(v, &mut out.crt_scanlines_v),
                "crt_edge_shadow" => parse_into(v, &mut out.crt_edge_shadow),
                "crt_flicker"     => parse_into(v, &mut out.crt_flicker),
                "crt_line_width"  => parse_into(v, &mut out.crt_line_width),
                "crt_color"       => parse_color(v, &mut out.crt_color),

                "grain_intensity"    => parse_into(v, &mut out.grain_intensity),
                "grain_coarseness"   => parse_into(v, &mut out.grain_coarseness),
                "grain_size"         => parse_into(v, &mut out.grain_size),
                "vignette_strength"  => parse_into(v, &mut out.vignette_strength),
                "underglow_strength" => parse_into(v, &mut out.underglow_strength),

                "spark_speed"  => parse_into(v, &mut out.spark_speed),
                "spark_size"   => parse_into(v, &mut out.spark_size),
                "spark_count"  => parse_into(v, &mut out.spark_count),
                "ember_speed"  => parse_into(v, &mut out.ember_speed),
                "ember_size"   => parse_into(v, &mut out.ember_size),
                "ember_count"  => parse_into(v, &mut out.ember_count),
                "beam_speed"   => parse_into(v, &mut out.beam_speed),
                "beam_size"    => parse_into(v, &mut out.beam_size),
                "beam_count"   => parse_into(v, &mut out.beam_count),
                "beam_height"  => parse_into(v, &mut out.beam_height),
                "beam_drift"   => parse_into(v, &mut out.beam_drift),
                "glitter_speed" => parse_into(v, &mut out.glitter_speed),
                "glitter_size"  => parse_into(v, &mut out.glitter_size),
                "glitter_count" => parse_into(v, &mut out.glitter_count),
                "cinder_size"   => parse_into(v, &mut out.cinder_size),

                key if key.starts_with("palette_") => {
                    if let Ok(idx) = key["palette_".len()..].parse::<usize>() {
                        if idx < PALETTE_LEN {
                            parse_color(v, &mut out.palette[idx]);
                        }
                    }
                }
                _ => {}
            }
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

fn parse_into(v: &str, dst: &mut f32) {
    if let Ok(f) = v.parse() { *dst = f; }
}

fn parse_color(v: &str, dst: &mut PaletteColor) {
    let parts: Vec<&str> = v.split(',').collect();
    for (i, p) in parts.iter().take(4).enumerate() {
        if let Ok(f) = p.trim().parse::<f32>() {
            dst[i] = f;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Default palette (matches the dark-theme presets in `gpu_buffers.rs`)
// ─────────────────────────────────────────────────────────────────────────────

/// Human-readable label and hint for each palette slot.  Index matches the
/// WGSL `PaletteUniform` array order — keep in sync with `types.wgsl`.
pub const PALETTE_LABELS: [(&str, &str); PALETTE_LEN] = [
    ("Spark core",       "Hot white-yellow spark center"),
    ("Spark ember",      "Outer ember glow"),
    ("Spark steel",      "Metallic highlight"),
    ("Ember hot",        "Bright hot center"),
    ("Beam center",      "Golden-white beam core"),
    ("Beam edge",        "Warm gold beam edge"),
    ("Glitter warm",     "Golden-white glitter"),
    ("Glitter cool",     "Blue-white glitter variation"),
    ("Cinder ember",     "Deep orange-red cinder"),
    ("Cinder gold",      "Tarnished gold cinder"),
    ("Cinder ash",       "Cool grey ash"),
    ("Cinder vine",      "Deep green vine"),
    ("Smoke cool",       "Blue-grey smoke band"),
    ("Smoke warm",       "Brown-amber smoke band"),
    ("Smoke moss",       "Mossy mid-tone smoke"),
    ("Kind: structural", "Structural element underlay"),
    ("Kind: error",      "Error element glow"),
    ("Kind: warn",       "Warn element glow"),
    ("Kind: info",       "Info element glow"),
    ("Kind: debug",      "Debug element glow"),
    ("Kind: span",       "Span element glow"),
    ("Kind: selected",   "Selected element glow"),
    ("Kind: panic",      "Panic element glow"),
    ("Reserved",         "Reserved padding slot"),
];

fn default_palette() -> [PaletteColor; PALETTE_LEN] {
    [
        [1.0,  0.97, 0.85, 1.0], // 0  spark_core
        [1.0,  0.4,  0.05, 1.0], // 1  spark_ember
        [0.7,  0.75, 0.85, 1.0], // 2  spark_steel
        [1.0,  0.6,  0.1,  1.0], // 3  ember_hot
        [1.0,  0.98, 0.88, 1.0], // 4  beam_center
        [1.0,  0.78, 0.2,  1.0], // 5  beam_edge
        [1.0,  0.95, 0.7,  1.0], // 6  glitter_warm
        [0.7,  0.85, 1.0,  1.0], // 7  glitter_cool
        [0.7,  0.15, 0.02, 1.0], // 8  cinder_ember
        [0.6,  0.45, 0.05, 1.0], // 9  cinder_gold
        [0.35, 0.33, 0.32, 1.0], // 10 cinder_ash
        [0.05, 0.22, 0.05, 1.0], // 11 cinder_vine
        [0.28, 0.34, 0.50, 1.0], // 12 smoke_cool
        [0.45, 0.30, 0.12, 1.0], // 13 smoke_warm
        [0.18, 0.32, 0.16, 1.0], // 14 smoke_moss
        [0.18, 0.16, 0.14, 1.0], // 15 kind_structural
        [0.97, 0.47, 0.55, 1.0], // 16 kind_error
        [0.88, 0.68, 0.41, 1.0], // 17 kind_warn
        [0.48, 0.81, 0.64, 1.0], // 18 kind_info
        [0.48, 0.60, 0.97, 1.0], // 19 kind_debug
        [0.61, 0.80, 0.41, 1.0], // 20 kind_span
        [1.0,  0.62, 0.39, 1.0], // 21 kind_selected
        [0.97, 0.47, 0.55, 1.0], // 22 kind_panic
        [0.0,  0.0,  0.0,  0.0], // 23 _pad
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Hex <-> float helpers (used by colour-picker UI)
// ─────────────────────────────────────────────────────────────────────────────

/// Parse `#rrggbb` or `rrggbb` into RGBA floats (alpha defaulted to 1.0).
pub fn hex_to_rgba(hex: &str) -> Option<PaletteColor> {
    let h = hex.trim().trim_start_matches('#');
    if h.len() != 6 { return None; }
    let r = u8::from_str_radix(&h[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&h[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&h[4..6], 16).ok()? as f32 / 255.0;
    Some([r, g, b, 1.0])
}

/// Format RGBA floats as `#rrggbb` (alpha discarded for `<input type="color">`).
pub fn rgba_to_hex(c: PaletteColor) -> String {
    let r = (c[0].clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (c[1].clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (c[2].clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}
