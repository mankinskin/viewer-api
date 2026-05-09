use dioxus::prelude::*;

use crate::store::{ThemeColors, ThemePreset, ThemeStore, ARCADIA, DARK, PAPER, SCRATCHBOARD};

use super::model::{CustomTheme, ThemeSnapshot};
use super::preview::inject_preview_css;

pub(super) fn render_preset_section(
    mut store: ThemeStore,
    mut draft: Signal<ThemeSnapshot>,
    mut committed: Signal<ThemeSnapshot>,
    custom_themes: Signal<Vec<CustomTheme>>,
    mut message: Signal<Option<(bool, String)>>,
) -> Element {
    rsx! {
        section {
            class: "theme-settings__section",
            h3 { class: "theme-settings__section-title", "Preset" }
            div {
                class: "theme-settings__preset-row",
                for (preset_key, preset_label) in [
                    ("arcadia", "Arcadia"),
                    ("dark", "Dark"),
                    ("paper", "Paper"),
                    ("scratchboard", "Scratchboard"),
                ] {
                    {
                        let active = store.preset().key() == preset_key;
                        let colors: &ThemeColors = match preset_key {
                            "arcadia" => &ARCADIA,
                            "dark" => &DARK,
                            "paper" => &PAPER,
                            _ => &SCRATCHBOARD,
                        };
                        let snapshot = ThemeSnapshot::from_colors(colors);
                        rsx! {
                            button {
                                key: "{preset_key}",
                                class: if active { "theme-settings__preset-btn theme-settings__preset-btn--active" }
                                       else { "theme-settings__preset-btn" },
                                onclick: move |_| {
                                    if let Some(preset) = ThemePreset::from_key(preset_key) {
                                        store.apply_preset(preset);
                                    }
                                    draft.set(snapshot.clone());
                                    committed.set(snapshot.clone());
                                    message.set(None);
                                },
                                "{preset_label}"
                            }
                        }
                    }
                }

                for (idx, custom_theme) in custom_themes.read().iter().enumerate() {
                    {
                        let theme = custom_theme.clone();
                        let theme_name = custom_theme.name.clone();
                        rsx! {
                            button {
                                key: "custom-{idx}",
                                class: "theme-settings__preset-btn",
                                onclick: move |_| {
                                    draft.set(theme.colors.clone());
                                    committed.set(theme.colors.clone());
                                    inject_preview_css(&theme.colors);
                                    message.set(Some((false, format!("Loaded \"{theme_name}\""))));
                                },
                                "{custom_theme.name}"
                            }
                        }
                    }
                }
            }
        }
    }
}