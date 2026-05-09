use super::{PaletteColor, PALETTE_LEN};

/// Human-readable label and hint for each palette slot.  Index matches the
/// WGSL `PaletteUniform` array order — keep in sync with `types.wgsl`.
pub const PALETTE_LABELS: [(&str, &str); PALETTE_LEN] = [
    ("Spark core", "Hot white-yellow spark center"),
    ("Spark ember", "Outer ember glow"),
    ("Spark steel", "Metallic highlight"),
    ("Ember hot", "Bright hot center"),
    ("Beam center", "Golden-white beam core"),
    ("Beam edge", "Warm gold beam edge"),
    ("Glitter warm", "Golden-white glitter"),
    ("Glitter cool", "Blue-white glitter variation"),
    ("Cinder ember", "Deep orange-red cinder"),
    ("Cinder gold", "Tarnished gold cinder"),
    ("Cinder ash", "Cool grey ash"),
    ("Cinder vine", "Deep green vine"),
    ("Smoke cool", "Blue-grey smoke band"),
    ("Smoke warm", "Brown-amber smoke band"),
    ("Smoke moss", "Mossy mid-tone smoke"),
    ("Kind: structural", "Structural element underlay"),
    ("Kind: error", "Error element glow"),
    ("Kind: warn", "Warn element glow"),
    ("Kind: info", "Info element glow"),
    ("Kind: debug", "Debug element glow"),
    ("Kind: span", "Span element glow"),
    ("Kind: selected", "Selected element glow"),
    ("Kind: panic", "Panic element glow"),
    ("Reserved", "Reserved padding slot"),
];

pub(super) fn default_palette() -> [PaletteColor; PALETTE_LEN] {
    [
        [1.0, 0.97, 0.85, 1.0],
        [1.0, 0.4, 0.05, 1.0],
        [0.7, 0.75, 0.85, 1.0],
        [1.0, 0.6, 0.1, 1.0],
        [1.0, 0.98, 0.88, 1.0],
        [1.0, 0.78, 0.2, 1.0],
        [1.0, 0.95, 0.7, 1.0],
        [0.7, 0.85, 1.0, 1.0],
        [0.7, 0.15, 0.02, 1.0],
        [0.6, 0.45, 0.05, 1.0],
        [0.35, 0.33, 0.32, 1.0],
        [0.05, 0.22, 0.05, 1.0],
        [0.28, 0.34, 0.50, 1.0],
        [0.45, 0.30, 0.12, 1.0],
        [0.18, 0.32, 0.16, 1.0],
        [0.18, 0.16, 0.14, 1.0],
        [0.97, 0.47, 0.55, 1.0],
        [0.88, 0.68, 0.41, 1.0],
        [0.48, 0.81, 0.64, 1.0],
        [0.48, 0.60, 0.97, 1.0],
        [0.61, 0.80, 0.41, 1.0],
        [1.0, 0.62, 0.39, 1.0],
        [0.97, 0.47, 0.55, 1.0],
        [0.0, 0.0, 0.0, 0.0],
    ]
}

/// Parse `#rrggbb` or `rrggbb` into RGBA floats (alpha defaulted to 1.0).
pub fn hex_to_rgba(hex: &str) -> Option<PaletteColor> {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
    Some([r, g, b, 1.0])
}

/// Format RGBA floats as `#rrggbb` (alpha discarded for `<input type="color">`).
pub fn rgba_to_hex(color: PaletteColor) -> String {
    let r = (color[0].clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (color[1].clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (color[2].clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}