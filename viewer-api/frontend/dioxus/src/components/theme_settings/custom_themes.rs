use dioxus::prelude::*;

use super::model::{save_custom_themes_storage, CustomTheme, ThemeSnapshot};

#[component]
pub(super) fn CustomThemesSection(
    draft: Signal<ThemeSnapshot>,
    mut custom_themes: Signal<Vec<CustomTheme>>,
    mut save_name: Signal<String>,
    mut rename_idx: Signal<Option<usize>>,
    mut rename_name: Signal<String>,
    mut message: Signal<Option<(bool, String)>>,
) -> Element {
    rsx! {
        section {
            class: "theme-settings__section",
            h3 { class: "theme-settings__section-title", "Save Custom Theme" }
            div {
                class: "theme-settings__save-row",
                input {
                    r#type: "text",
                    class: "theme-settings__name-input",
                    placeholder: "Theme name…",
                    value: "{save_name}",
                    aria_label: "Custom theme name",
                    oninput: move |event| save_name.set(event.value()),
                }
                button {
                    class: "theme-settings__action-btn",
                    disabled: save_name.read().trim().is_empty(),
                    onclick: move |_| {
                        let name = save_name.read().trim().to_string();
                        if name.is_empty() {
                            return;
                        }

                        let new_theme = CustomTheme {
                            name: name.clone(),
                            colors: draft.read().clone(),
                        };
                        let mut themes = custom_themes.write();
                        if let Some(position) = themes.iter().position(|theme| theme.name == name) {
                            themes[position] = new_theme;
                        } else {
                            themes.push(new_theme);
                        }
                        save_custom_themes_storage(&themes);
                        drop(themes);
                        save_name.set(String::new());
                        message.set(Some((false, format!("Saved \"{name}\""))));
                    },
                    "Save"
                }
            }

            if !custom_themes.read().is_empty() {
                div {
                    class: "theme-settings__custom-list",
                    for (idx, custom_theme) in custom_themes.read().iter().enumerate() {
                        {
                            let theme_name = custom_theme.name.clone();
                            let rename_source = theme_name.clone();
                            let delete_source = theme_name.clone();
                            let is_renaming = rename_idx.read().is_some_and(|value| value == idx);
                            rsx! {
                                div {
                                    key: "ct-{idx}",
                                    class: "theme-settings__custom-row",
                                    if is_renaming {
                                        input {
                                            r#type: "text",
                                            class: "theme-settings__name-input",
                                            value: "{rename_name}",
                                            aria_label: "New name for {theme_name}",
                                            oninput: move |event| rename_name.set(event.value()),
                                        }
                                        button {
                                            class: "theme-settings__action-btn",
                                            onclick: move |_| {
                                                let new_name = rename_name.read().trim().to_string();
                                                if !new_name.is_empty() {
                                                    let mut themes = custom_themes.write();
                                                    if let Some(index) = *rename_idx.read() {
                                                        if let Some(theme) = themes.get_mut(index) {
                                                            theme.name = new_name.clone();
                                                        }
                                                    }
                                                    save_custom_themes_storage(&themes);
                                                }
                                                rename_idx.set(None);
                                                rename_name.set(String::new());
                                                message.set(Some((false, "Renamed.".to_string())));
                                            },
                                            "OK"
                                        }
                                        button {
                                            class: "theme-settings__action-btn",
                                            onclick: move |_| {
                                                rename_idx.set(None);
                                                rename_name.set(String::new());
                                            },
                                            "Cancel"
                                        }
                                    } else {
                                        span { class: "theme-settings__custom-name", "{theme_name}" }
                                        button {
                                            class: "theme-settings__action-btn",
                                            aria_label: "Rename {theme_name}",
                                            onclick: move |_| {
                                                rename_idx.set(Some(idx));
                                                rename_name.set(rename_source.clone());
                                            },
                                            "Rename"
                                        }
                                        button {
                                            class: "theme-settings__action-btn theme-settings__action-btn--danger",
                                            aria_label: "Delete {theme_name}",
                                            onclick: move |_| {
                                                let mut themes = custom_themes.write();
                                                themes.retain(|theme| theme.name != delete_source);
                                                save_custom_themes_storage(&themes);
                                                drop(themes);
                                                message.set(Some((false, format!("Deleted \"{delete_source}\""))));
                                            },
                                            "Delete"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}