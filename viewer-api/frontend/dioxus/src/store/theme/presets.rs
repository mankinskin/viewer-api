/// Full set of design-token colours mirroring the TypeScript theme.ts interface.
#[derive(Clone, PartialEq, Debug)]
pub struct ThemeColors {
    pub bg_primary: &'static str,
    pub bg_secondary: &'static str,
    pub bg_tertiary: &'static str,
    pub bg_elevated: &'static str,
    pub text_primary: &'static str,
    pub text_secondary: &'static str,
    pub text_muted: &'static str,
    pub border_primary: &'static str,
    pub border_secondary: &'static str,
    pub accent_blue: &'static str,
    pub accent_purple: &'static str,
    pub accent_green: &'static str,
    pub accent_yellow: &'static str,
    pub accent_red: &'static str,
    pub accent_orange: &'static str,
    pub accent_cyan: &'static str,
    pub syntax_keyword: &'static str,
    pub syntax_string: &'static str,
    pub syntax_comment: &'static str,
    pub syntax_number: &'static str,
    pub syntax_function: &'static str,
    pub syntax_type: &'static str,
    pub syntax_variable: &'static str,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub enum ThemePreset {
    #[default]
    Arcadia,
    Dark,
    Paper,
    Scratchboard,
}

impl ThemePreset {
    pub fn key(&self) -> &'static str {
        match self {
            ThemePreset::Arcadia => "arcadia",
            ThemePreset::Dark => "dark",
            ThemePreset::Paper => "paper",
            ThemePreset::Scratchboard => "scratchboard",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "arcadia" => Some(ThemePreset::Arcadia),
            "dark" => Some(ThemePreset::Dark),
            "paper" => Some(ThemePreset::Paper),
            "scratchboard" => Some(ThemePreset::Scratchboard),
            _ => None,
        }
    }

    pub fn colors(&self) -> &'static ThemeColors {
        match self {
            ThemePreset::Arcadia => &ARCADIA,
            ThemePreset::Dark => &DARK,
            ThemePreset::Paper => &PAPER,
            ThemePreset::Scratchboard => &SCRATCHBOARD,
        }
    }
}

pub static ARCADIA: ThemeColors = ThemeColors {
    bg_primary: "#eae6df",
    bg_secondary: "#f2efe8",
    bg_tertiary: "#f8f6f1",
    bg_elevated: "#ffffff",
    text_primary: "#2c2a26",
    text_secondary: "#5a5650",
    text_muted: "#8a8680",
    border_primary: "#d4cfc7",
    border_secondary: "#e8e4dc",
    accent_blue: "#4a7fa5",
    accent_purple: "#7c5c9e",
    accent_green: "#4a8c5c",
    accent_yellow: "#b8860b",
    accent_red: "#c0392b",
    accent_orange: "#d35400",
    accent_cyan: "#1a8c8c",
    syntax_keyword: "#4a7fa5",
    syntax_string: "#4a8c5c",
    syntax_comment: "#8a8680",
    syntax_number: "#b8860b",
    syntax_function: "#7c5c9e",
    syntax_type: "#1a8c8c",
    syntax_variable: "#2c2a26",
};

pub static DARK: ThemeColors = ThemeColors {
    bg_primary: "#1a1b26",
    bg_secondary: "#1f2035",
    bg_tertiary: "#24283b",
    bg_elevated: "#2c2f4a",
    text_primary: "#c0caf5",
    text_secondary: "#9aa5ce",
    text_muted: "#565f89",
    border_primary: "#292e42",
    border_secondary: "#1f2335",
    accent_blue: "#7aa2f7",
    accent_purple: "#bb9af7",
    accent_green: "#9ece6a",
    accent_yellow: "#e0af68",
    accent_red: "#f7768e",
    accent_orange: "#ff9e64",
    accent_cyan: "#7dcfff",
    syntax_keyword: "#bb9af7",
    syntax_string: "#9ece6a",
    syntax_comment: "#565f89",
    syntax_number: "#ff9e64",
    syntax_function: "#7aa2f7",
    syntax_type: "#7dcfff",
    syntax_variable: "#c0caf5",
};

pub static PAPER: ThemeColors = ThemeColors {
    bg_primary: "#f5f0eb",
    bg_secondary: "#faf8f5",
    bg_tertiary: "#ffffff",
    bg_elevated: "#ffffff",
    text_primary: "#1a1a1a",
    text_secondary: "#4a4a4a",
    text_muted: "#888888",
    border_primary: "#ddd8d0",
    border_secondary: "#ece8e2",
    accent_blue: "#2563eb",
    accent_purple: "#7c3aed",
    accent_green: "#16a34a",
    accent_yellow: "#ca8a04",
    accent_red: "#dc2626",
    accent_orange: "#ea580c",
    accent_cyan: "#0891b2",
    syntax_keyword: "#2563eb",
    syntax_string: "#16a34a",
    syntax_comment: "#888888",
    syntax_number: "#ca8a04",
    syntax_function: "#7c3aed",
    syntax_type: "#0891b2",
    syntax_variable: "#1a1a1a",
};

pub static SCRATCHBOARD: ThemeColors = ThemeColors {
    bg_primary: "#0f0f0f",
    bg_secondary: "#1a1a1a",
    bg_tertiary: "#222222",
    bg_elevated: "#2a2a2a",
    text_primary: "#f0f0f0",
    text_secondary: "#b0b0b0",
    text_muted: "#606060",
    border_primary: "#333333",
    border_secondary: "#2a2a2a",
    accent_blue: "#58a6ff",
    accent_purple: "#c499f3",
    accent_green: "#56d364",
    accent_yellow: "#e3b341",
    accent_red: "#f85149",
    accent_orange: "#ffa657",
    accent_cyan: "#39d0d8",
    syntax_keyword: "#58a6ff",
    syntax_string: "#56d364",
    syntax_comment: "#606060",
    syntax_number: "#ffa657",
    syntax_function: "#c499f3",
    syntax_type: "#39d0d8",
    syntax_variable: "#f0f0f0",
};