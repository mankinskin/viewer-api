//! Per-theme effect settings persisted to `localStorage`.

#![cfg(target_arch = "wasm32")]

const STORAGE_KEY_PREFIX: &str = "viewer-api-effects-";

#[derive(Clone)]
pub(super) struct EffectSettings {
    pub smoke_intensity:   f32,
    pub crt_scanlines_h:   f32,
    pub crt_edge_shadow:   f32,
    pub grain_intensity:   f32,
    pub vignette_str:      f32,
    pub particles_enabled: bool,
}

impl Default for EffectSettings {
    fn default() -> Self {
        Self {
            smoke_intensity:   0.6,
            crt_scanlines_h:   0.15,
            crt_edge_shadow:   0.4,
            grain_intensity:   0.15,
            vignette_str:      0.5,
            particles_enabled: true,
        }
    }
}

impl EffectSettings {
    /// Load from `localStorage` for the active theme; falls back to defaults.
    pub fn load(theme_key: &str) -> Self {
        let key = format!("{}{}", STORAGE_KEY_PREFIX, theme_key);
        let json = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item(&key).ok().flatten());
        let Some(j) = json else { return Self::default(); };

        // Hand-rolled parse — avoids serde dependency.
        let mut s = Self::default();
        for pair in j.trim_matches(['{', '}'].as_slice()).split(',') {
            let mut parts = pair.splitn(2, ':');
            let k = parts.next().unwrap_or("").trim().trim_matches('"');
            let v = parts.next().unwrap_or("").trim();
            match k {
                "smoke_intensity"   => { if let Ok(f) = v.parse() { s.smoke_intensity   = f; } }
                "crt_scanlines_h"   => { if let Ok(f) = v.parse() { s.crt_scanlines_h   = f; } }
                "crt_edge_shadow"   => { if let Ok(f) = v.parse() { s.crt_edge_shadow   = f; } }
                "grain_intensity"   => { if let Ok(f) = v.parse() { s.grain_intensity   = f; } }
                "vignette_str"      => { if let Ok(f) = v.parse() { s.vignette_str      = f; } }
                "particles_enabled" => { s.particles_enabled = v == "true"; }
                _ => {}
            }
        }
        s
    }

    #[allow(dead_code)]
    pub fn save(&self, theme_key: &str) {
        let key  = format!("{}{}", STORAGE_KEY_PREFIX, theme_key);
        let json = format!(
            r#"{{"smoke_intensity":{},"crt_scanlines_h":{},"crt_edge_shadow":{},"grain_intensity":{},"vignette_str":{},"particles_enabled":{}}}"#,
            self.smoke_intensity,
            self.crt_scanlines_h,
            self.crt_edge_shadow,
            self.grain_intensity,
            self.vignette_str,
            self.particles_enabled,
        );
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
        {
            let _ = storage.set_item(&key, &json);
        }
    }
}
