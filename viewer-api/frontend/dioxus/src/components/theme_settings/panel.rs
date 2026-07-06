use dioxus::prelude::*;

use crate::{
    effects::wgpu_overlay::EffectSettings,
    store::ThemeStore,
};

use super::{
    custom_themes::CustomThemesSection,
    effects::EffectControls,
    model::{
        load_custom_themes,
        CustomTheme,
        ThemeSnapshot,
    },
    presets::render_preset_section,
    preview::{
        inject_preview_css,
        remove_preview_css,
    },
    tokens::TokenSections,
};

#[component]
pub fn ThemeSettings(
    #[props(default)] on_close: EventHandler<()>,
    #[props(default)] class: String,
) -> Element {
    let store = use_context::<ThemeStore>();
    let draft = use_signal(|| ThemeSnapshot::from_colors(store.colors()));
    let committed =
        use_signal(|| ThemeSnapshot::from_colors(store.colors()));
    let effects_draft = use_signal(|| store.effects_committed());
    let effects_committed_local = use_signal(|| store.effects_committed());
    let custom_themes: Signal<Vec<CustomTheme>> =
        use_signal(load_custom_themes);
    let save_name = use_signal(String::new);
    let rename_idx: Signal<Option<usize>> = use_signal(|| None);
    let rename_name = use_signal(String::new);
    let import_json = use_signal(String::new);
    let message: Signal<Option<(bool, String)>> = use_signal(|| None);

    use_effect(move || {
        let effects = effects_draft.read().clone();
        store.preview_effects(effects);
    });

    use_drop(move || {
        remove_preview_css();
        store.revert_effects();
    });

    use_effect(move || {
        let snapshot = draft.read().clone();
        inject_preview_css(&snapshot);
    });

    let panel_class = if class.is_empty() {
        "theme-settings glass-panel".to_string()
    } else {
        format!("theme-settings glass-panel {class}")
    };

    rsx! {
        div {
            class: "{panel_class}",
            role: "dialog",
            aria_label: "Theme settings",
            div {
                class: "glass-panel__header theme-settings__header",
                span { class: "glass-panel__title", "Theme Settings" }
                button {
                    class: "tab-close",
                    aria_label: "Close theme settings",
                    onclick: move |_| {
                        remove_preview_css();
                        store.revert_effects();
                        on_close.call(());
                    },
                    "✕"
                }
            }

            div {
                class: "theme-settings__body",
                {render_preset_section(store, draft, committed, custom_themes, message)}
                {render_effects_section(store, effects_draft, message)}
                TokenSections { draft }
                CustomThemesSection {
                    draft,
                    custom_themes,
                    save_name,
                    rename_idx,
                    rename_name,
                    message,
                }
                ImportExportSection { draft, import_json, message }
                {render_footer_actions(store, draft, committed, effects_draft, effects_committed_local, message)}

                if let Some((is_error, text)) = message.read().as_ref() {
                    div {
                        class: if *is_error { "theme-settings__message theme-settings__message--error" }
                               else { "theme-settings__message" },
                        "{text}"
                    }
                }
            }
        }
    }
}

fn render_effects_section(
    mut store: ThemeStore,
    effects_draft: Signal<EffectSettings>,
    mut message: Signal<Option<(bool, String)>>,
) -> Element {
    rsx! {
        section {
            class: "theme-settings__section",
            h3 { class: "theme-settings__section-title", "Effects" }
            p {
                class: "theme-settings__section-hint",
                "Animated background — smoke, sparks/embers/beams (\"angelic beam\" particle simulation), and CRT scanlines. ",
                "Disabled by default to keep the viewer lightweight; enable to show the full visual treatment."
            }
            div {
                class: "theme-settings__effect-row",
                div {
                    class: "theme-settings__effect-info",
                    span { class: "theme-settings__effect-label", "Enable GPU overlay" }
                    span { class: "theme-settings__effect-desc", "Master switch — toggles smoke, particles, and CRT effects." }
                }
                label {
                    class: "theme-settings__toggle-switch",
                    aria_label: "Toggle GPU overlay effects",
                    input {
                        r#type: "checkbox",
                        checked: store.gpu_enabled(),
                        onchange: move |event: Event<FormData>| {
                            let enabled = event.value() == "true";
                            store.set_gpu_enabled(enabled);
                            let text = if enabled {
                                "GPU overlay enabled.".to_string()
                            } else {
                                "GPU overlay disabled.".to_string()
                            };
                            message.set(Some((false, text)));
                        },
                    }
                    span { class: "theme-settings__toggle-slider" }
                }
            }
            EffectControls { draft: effects_draft }
        }
    }
}

#[component]
fn ImportExportSection(
    mut draft: Signal<ThemeSnapshot>,
    mut import_json: Signal<String>,
    mut message: Signal<Option<(bool, String)>>,
) -> Element {
    rsx! {
        section {
            class: "theme-settings__section",
            h3 { class: "theme-settings__section-title", "Export / Import JSON" }
            div {
                class: "theme-settings__export-row",
                button {
                    class: "theme-settings__action-btn",
                    aria_label: "Copy theme JSON to clipboard",
                    onclick: move |_| {
                        let json = draft.read().to_json();
                        import_json.set(json.clone());
                        #[cfg(target_arch = "wasm32")]
                        {
                            if let Some(window) = web_sys::window() {
                                let _ = window.navigator().clipboard().write_text(&json);
                            }
                        }
                        message.set(Some((false, "JSON copied to clipboard.".to_string())));
                    },
                    "Export (copy)"
                }
            }
            textarea {
                class: "theme-settings__import-textarea",
                rows: "4",
                placeholder: "Paste JSON here to import…",
                aria_label: "Import theme JSON",
                value: "{import_json}",
                oninput: move |event| import_json.set(event.value()),
            }
            button {
                class: "theme-settings__action-btn",
                onclick: move |_| {
                    let raw = import_json.read().clone();
                    if let Some(snapshot) = ThemeSnapshot::from_json(&raw) {
                        draft.set(snapshot.clone());
                        inject_preview_css(&snapshot);
                        message.set(Some((false, "Theme imported — adjust and save.".to_string())));
                    } else {
                        message.set(Some((true, "Invalid JSON — import failed.".to_string())));
                    }
                },
                "Import"
            }
        }
    }
}

fn render_footer_actions(
    mut store: ThemeStore,
    mut draft: Signal<ThemeSnapshot>,
    mut committed: Signal<ThemeSnapshot>,
    mut effects_draft: Signal<EffectSettings>,
    mut effects_committed_local: Signal<EffectSettings>,
    mut message: Signal<Option<(bool, String)>>,
) -> Element {
    rsx! {
        div {
            class: "theme-settings__footer",
            button {
                class: "theme-settings__action-btn",
                title: "Revert all unsaved edits",
                onclick: move |_| {
                    let saved = committed.read().clone();
                    draft.set(saved.clone());
                    inject_preview_css(&saved);
                    let saved_effects = effects_committed_local.read().clone();
                    effects_draft.set(saved_effects.clone());
                    store.preview_effects(saved_effects);
                    message.set(Some((false, "Changes reverted.".to_string())));
                },
                "Undo changes"
            }
            button {
                class: "theme-settings__action-btn theme-settings__action-btn--primary",
                title: "Apply current draft as the active theme",
                onclick: move |_| {
                    let snapshot = draft.read().clone();
                    committed.set(snapshot.clone());
                    inject_preview_css(&snapshot);
                    let effects = effects_draft.read().clone();
                    store.commit_effects(effects.clone());
                    effects_committed_local.set(effects);
                    message.set(Some((false, "Theme applied.".to_string())));
                },
                "Apply"
            }
        }
    }
}
