#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

use crate::effects::wgpu_overlay::PaletteColor;

const STORAGE_KEY: &str = "viewer-api-graph-theme";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum GraphEdgeBlendMode {
    Normal,
    PlusLighter,
    #[default]
    Screen,
}

impl GraphEdgeBlendMode {
    pub(crate) const ALL: [GraphEdgeBlendMode; 3] = [
        GraphEdgeBlendMode::Screen,
        GraphEdgeBlendMode::PlusLighter,
        GraphEdgeBlendMode::Normal,
    ];

    pub(crate) fn css_value(self) -> &'static str {
        match self {
            GraphEdgeBlendMode::Normal => "normal",
            GraphEdgeBlendMode::PlusLighter => "plus-lighter",
            GraphEdgeBlendMode::Screen => "screen",
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            GraphEdgeBlendMode::Normal => "Normal",
            GraphEdgeBlendMode::PlusLighter => "Plus lighter",
            GraphEdgeBlendMode::Screen => "Screen",
        }
    }

    fn from_storage_key(value: &str) -> Option<Self> {
        match value {
            "normal" => Some(GraphEdgeBlendMode::Normal),
            "plus-lighter" => Some(GraphEdgeBlendMode::PlusLighter),
            "screen" => Some(GraphEdgeBlendMode::Screen),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GraphThemeSettings {
    pub edge_dependency: PaletteColor,
    pub edge_blocking: PaletteColor,
    pub edge_structural: PaletteColor,
    pub edge_default: PaletteColor,
    pub edge_overlay_opacity: f32,
    pub edge_blend_mode: GraphEdgeBlendMode,
    pub node_surface: PaletteColor,
    pub node_border: PaletteColor,
    pub node_text: PaletteColor,
    pub node_shadow_alpha: f32,
}

impl Default for GraphThemeSettings {
    fn default() -> Self {
        Self {
            edge_dependency: [0.28, 0.86, 1.00, 1.00],
            edge_blocking: [1.00, 0.56, 0.28, 1.00],
            edge_structural: [0.76, 0.62, 1.00, 0.96],
            edge_default: [0.78, 0.82, 1.00, 0.90],
            edge_overlay_opacity: 0.80,
            edge_blend_mode: GraphEdgeBlendMode::Screen,
            node_surface: [0.11, 0.13, 0.18, 0.93],
            node_border: [0.80, 0.85, 0.95, 0.18],
            node_text: [0.96, 0.97, 1.00, 1.00],
            node_shadow_alpha: 0.32,
        }
    }
}

impl GraphThemeSettings {
    pub(crate) fn load() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let raw = web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
                .and_then(|s| s.get_item(STORAGE_KEY).ok().flatten());
            if let Some(raw) = raw {
                return Self::from_storage_string(&raw);
            }
        }
        Self::default()
    }

    pub(crate) fn save(&self) {
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

    pub(crate) fn edge_color(
        &self,
        kind: &str,
    ) -> PaletteColor {
        match kind {
            "depends_on" | "dep" | "code_ref" => self.edge_dependency,
            "blocks" => self.edge_blocking,
            "parent" | "child" | "section" => self.edge_structural,
            _ => self.edge_default,
        }
    }

    fn to_storage_string(&self) -> String {
        let mut out = String::with_capacity(512);
        out.push_str(&format!(
            "edge_dependency={},{},{},{}\n",
            self.edge_dependency[0],
            self.edge_dependency[1],
            self.edge_dependency[2],
            self.edge_dependency[3]
        ));
        out.push_str(&format!(
            "edge_blocking={},{},{},{}\n",
            self.edge_blocking[0],
            self.edge_blocking[1],
            self.edge_blocking[2],
            self.edge_blocking[3]
        ));
        out.push_str(&format!(
            "edge_structural={},{},{},{}\n",
            self.edge_structural[0],
            self.edge_structural[1],
            self.edge_structural[2],
            self.edge_structural[3]
        ));
        out.push_str(&format!(
            "edge_default={},{},{},{}\n",
            self.edge_default[0],
            self.edge_default[1],
            self.edge_default[2],
            self.edge_default[3]
        ));
        out.push_str(&format!(
            "edge_overlay_opacity={}\n",
            self.edge_overlay_opacity
        ));
        out.push_str(&format!(
            "edge_blend_mode={}\n",
            self.edge_blend_mode.css_value()
        ));
        out.push_str(&format!(
            "node_surface={},{},{},{}\n",
            self.node_surface[0],
            self.node_surface[1],
            self.node_surface[2],
            self.node_surface[3]
        ));
        out.push_str(&format!(
            "node_border={},{},{},{}\n",
            self.node_border[0],
            self.node_border[1],
            self.node_border[2],
            self.node_border[3]
        ));
        out.push_str(&format!(
            "node_text={},{},{},{}\n",
            self.node_text[0],
            self.node_text[1],
            self.node_text[2],
            self.node_text[3]
        ));
        out.push_str(&format!(
            "node_shadow_alpha={}\n",
            self.node_shadow_alpha
        ));
        out
    }

    fn from_storage_string(value: &str) -> Self {
        let mut out = Self::default();
        for line in value.lines() {
            apply_storage_line(&mut out, line);
        }
        out
    }
}

fn apply_storage_line(
    settings: &mut GraphThemeSettings,
    line: &str,
) {
    let Some((key, value)) = line.split_once('=') else {
        return;
    };

    match key {
        "edge_dependency" => parse_color(value, &mut settings.edge_dependency),
        "edge_blocking" => parse_color(value, &mut settings.edge_blocking),
        "edge_structural" => parse_color(value, &mut settings.edge_structural),
        "edge_default" => parse_color(value, &mut settings.edge_default),
        "edge_overlay_opacity" => {
            if let Ok(value) = value.trim().parse::<f32>() {
                settings.edge_overlay_opacity = value.clamp(0.0, 1.0);
            }
        },
        "edge_blend_mode" => {
            if let Some(value) =
                GraphEdgeBlendMode::from_storage_key(value.trim())
            {
                settings.edge_blend_mode = value;
            }
        },
        "node_surface" => parse_color(value, &mut settings.node_surface),
        "node_border" => parse_color(value, &mut settings.node_border),
        "node_text" => parse_color(value, &mut settings.node_text),
        "node_shadow_alpha" =>
            if let Ok(value) = value.trim().parse::<f32>() {
                settings.node_shadow_alpha = value.clamp(0.0, 1.0);
            },
        _ => {},
    }
}

fn parse_color(
    value: &str,
    dst: &mut PaletteColor,
) {
    for (index, part) in value.split(',').take(4).enumerate() {
        if let Ok(value) = part.trim().parse::<f32>() {
            dst[index] = value;
        }
    }
}
