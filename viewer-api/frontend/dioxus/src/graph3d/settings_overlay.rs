use dioxus::prelude::*;

use crate::{
    effects::wgpu_overlay::{
        hex_to_rgba,
        rgba_to_hex,
        PaletteColor,
    },
    store::ThemeStore,
};

use super::{
    theme::{
        GraphEdgeBlendMode,
        GraphThemeSettings,
    },
    LayoutMode,
    Projection,
};

#[derive(Props, Clone, PartialEq)]
pub(super) struct GraphSettingsOverlayProps {
    pub layout_mode: LayoutMode,
    pub projection: Projection,
    pub on_layout_mode_change: Option<EventHandler<LayoutMode>>,
    pub on_projection_change: Option<EventHandler<Projection>>,
}

fn opt_btn_style(active: bool) -> String {
    let (background, border, color) = if active {
        (
            "rgba(79,140,255,0.20)",
            "1px solid rgba(79,140,255,0.50)",
            "#93bbff",
        )
    } else {
        (
            "rgba(255,255,255,0.05)",
            "1px solid rgba(255,255,255,0.10)",
            "#aaa",
        )
    };

    format!(
        "flex:1; padding:5px 0; border-radius:5px; border:{border}; \
         background:{background}; color:{color}; font-size:11px; font-weight:500; \
         cursor:pointer; text-align:center; white-space:nowrap;"
    )
}

#[component]
fn SliderRow(
    label: String,
    value_label: String,
    min: f32,
    max: f32,
    step: f32,
    value: f32,
    on_change: EventHandler<f32>,
) -> Element {
    rsx! {
        div { style: "display:flex; flex-direction:column; gap:5px; margin-bottom:10px;",
            div { style: "display:flex; align-items:center; justify-content:space-between; gap:10px;",
                span { style: "font-size:11px; color:#cbd5e1;", "{label}" }
                span { style: "font-size:11px; color:#94a3b8; font-variant-numeric: tabular-nums;", "{value_label}" }
            }
            input {
                r#type: "range",
                min: "{min}",
                max: "{max}",
                step: "{step}",
                value: "{value}",
                oninput: move |event| {
                    if let Ok(value) = event.value().parse::<f32>() {
                        on_change.call(value);
                    }
                },
            }
        }
    }
}

#[component]
fn ColorRow(
    label: String,
    value: PaletteColor,
    on_change: EventHandler<PaletteColor>,
) -> Element {
    let hex = rgba_to_hex(value);
    rsx! {
        label {
            style: "display:flex; align-items:center; gap:10px; margin-bottom:8px; cursor:pointer;",
            span { style: "flex:1; font-size:11px; color:#cbd5e1;", "{label}" }
            span {
                style: "width:16px; height:16px; border-radius:999px; border:1px solid rgba(255,255,255,0.16); background:{hex}; box-shadow: inset 0 0 0 1px rgba(0,0,0,0.22);",
            }
            input {
                r#type: "color",
                value: "{hex}",
                oninput: move |event| {
                    if let Some(mut rgba) = hex_to_rgba(&event.value()) {
                        rgba[3] = value[3];
                        on_change.call(rgba);
                    }
                },
            }
        }
    }
}

fn update_graph_theme(
    mut store: ThemeStore,
    mutate: impl FnOnce(&mut GraphThemeSettings),
) {
    let mut next = store.graph_theme();
    mutate(&mut next);
    store.set_graph_theme(next);
}

#[component]
pub(super) fn GraphSettingsOverlay(
    props: GraphSettingsOverlayProps
) -> Element {
    let mut open: Signal<bool> = use_hook(|| Signal::new(false));
    let theme_store = use_context::<ThemeStore>();
    let graph_theme = theme_store.graph_theme();
    let cur_layout = props.layout_mode;
    let cur_proj = props.projection;

    let on_layout_mode_change = props.on_layout_mode_change.clone();
    let on_projection_change = props.on_projection_change.clone();

    rsx! {
        div {
            class: "graph-settings-overlay",
            style: "position: absolute; bottom: 12px; right: 12px; z-index: 100; display: flex; flex-direction: column; align-items: flex-end;",
            if *open.read() {
                div {
                    "data-graph-passthrough": "false",
                    style: "
                        margin-bottom: 6px;
                        min-width: 280px;
                        max-width: min(360px, calc(100vw - 24px));
                        max-height: min(72vh, 640px);
                        overflow-y: auto;
                        background: rgba(18, 20, 28, 0.88);
                        border: 1px solid rgba(255,255,255,0.10);
                        border-radius: 9px;
                        padding: 12px 14px;
                        box-shadow: 0 6px 24px rgba(0,0,0,0.5);
                        font-family: sans-serif;
                        font-size: 12px;
                        color: #ccc;
                        backdrop-filter: blur(8px);
                        -webkit-backdrop-filter: blur(8px);
                    ",
                    onclick: move |event: MouseEvent| event.stop_propagation(),
                    onmousedown: move |event: MouseEvent| event.stop_propagation(),
                    onwheel: move |event: WheelEvent| event.stop_propagation(),
                    if on_layout_mode_change.is_some() {
                        {
                            let on_hierarchical = on_layout_mode_change.clone();
                            let on_flat = on_layout_mode_change.clone();
                            rsx! {
                                div {
                                    style: "font-size:10px; font-weight:700; letter-spacing:0.07em; text-transform:uppercase; color:#666; margin-bottom:7px;",
                                    "Layout"
                                }
                                div { style: "display:flex; gap:6px; margin-bottom:10px;",
                                    button {
                                        style: "{opt_btn_style(cur_layout == LayoutMode::Hierarchical3D)}",
                                        onclick: move |_| {
                                            if let Some(ref callback) = on_hierarchical {
                                                callback.call(LayoutMode::Hierarchical3D);
                                            }
                                        },
                                        "Hierarchical 3D"
                                    }
                                    button {
                                        style: "{opt_btn_style(cur_layout == LayoutMode::Flat2D)}",
                                        onclick: move |_| {
                                            if let Some(ref callback) = on_flat {
                                                callback.call(LayoutMode::Flat2D);
                                            }
                                        },
                                        "Flat 2D"
                                    }
                                }
                            }
                        }
                    }
                    if on_projection_change.is_some() {
                        {
                            let on_perspective = on_projection_change.clone();
                            let on_orthographic = on_projection_change.clone();
                            rsx! {
                                div {
                                    style: "font-size:10px; font-weight:700; letter-spacing:0.07em; text-transform:uppercase; color:#666; margin-bottom:7px;",
                                    "Projection"
                                }
                                div { style: "display:flex; gap:6px;",
                                    button {
                                        style: "{opt_btn_style(cur_proj == Projection::Perspective)}",
                                        onclick: move |_| {
                                            if let Some(ref callback) = on_perspective {
                                                callback.call(Projection::Perspective);
                                            }
                                        },
                                        "Perspective"
                                    }
                                    button {
                                        style: "{opt_btn_style(cur_proj == Projection::Orthographic)}",
                                        onclick: move |_| {
                                            if let Some(ref callback) = on_orthographic {
                                                callback.call(Projection::Orthographic);
                                            }
                                        },
                                        "Orthographic"
                                    }
                                }
                            }
                        }
                    }
                    div {
                        style: "font-size:10px; font-weight:700; letter-spacing:0.07em; text-transform:uppercase; color:#666; margin:12px 0 7px;",
                        "Edge theme"
                    }
                    SliderRow {
                        label: "Overlay opacity".to_string(),
                        value_label: format!("{:.0}%", graph_theme.edge_overlay_opacity * 100.0),
                        min: 0.20,
                        max: 1.0,
                        step: 0.05,
                        value: graph_theme.edge_overlay_opacity,
                        on_change: {
                            let store = theme_store;
                            move |value| update_graph_theme(store, |theme| theme.edge_overlay_opacity = value)
                        },
                    }
                    div {
                        style: "font-size:11px; color:#cbd5e1; margin-bottom:6px;",
                        "Blend mode"
                    }
                    div { style: "display:flex; gap:6px; margin-bottom:10px;",
                        for mode in GraphEdgeBlendMode::ALL {
                            button {
                                key: "edge-blend-{mode.css_value()}",
                                style: "{opt_btn_style(graph_theme.edge_blend_mode == mode)}",
                                onclick: {
                                    let store = theme_store;
                                    move |_| update_graph_theme(store, |theme| theme.edge_blend_mode = mode)
                                },
                                "{mode.label()}"
                            }
                        }
                    }
                    ColorRow {
                        label: "Dependency edge".to_string(),
                        value: graph_theme.edge_dependency,
                        on_change: {
                            let store = theme_store;
                            move |color| update_graph_theme(store, |theme| theme.edge_dependency = color)
                        },
                    }
                    ColorRow {
                        label: "Blocking edge".to_string(),
                        value: graph_theme.edge_blocking,
                        on_change: {
                            let store = theme_store;
                            move |color| update_graph_theme(store, |theme| theme.edge_blocking = color)
                        },
                    }
                    ColorRow {
                        label: "Structural edge".to_string(),
                        value: graph_theme.edge_structural,
                        on_change: {
                            let store = theme_store;
                            move |color| update_graph_theme(store, |theme| theme.edge_structural = color)
                        },
                    }
                    ColorRow {
                        label: "Default edge".to_string(),
                        value: graph_theme.edge_default,
                        on_change: {
                            let store = theme_store;
                            move |color| update_graph_theme(store, |theme| theme.edge_default = color)
                        },
                    }

                    div {
                        style: "font-size:10px; font-weight:700; letter-spacing:0.07em; text-transform:uppercase; color:#666; margin:12px 0 7px;",
                        "Node theme"
                    }
                    ColorRow {
                        label: "Card surface".to_string(),
                        value: graph_theme.node_surface,
                        on_change: {
                            let store = theme_store;
                            move |color| update_graph_theme(store, |theme| theme.node_surface = color)
                        },
                    }
                    ColorRow {
                        label: "Card border".to_string(),
                        value: graph_theme.node_border,
                        on_change: {
                            let store = theme_store;
                            move |color| update_graph_theme(store, |theme| theme.node_border = color)
                        },
                    }
                    ColorRow {
                        label: "Card text".to_string(),
                        value: graph_theme.node_text,
                        on_change: {
                            let store = theme_store;
                            move |color| update_graph_theme(store, |theme| theme.node_text = color)
                        },
                    }
                    SliderRow {
                        label: "Shadow strength".to_string(),
                        value_label: format!("{:.0}%", graph_theme.node_shadow_alpha * 100.0),
                        min: 0.0,
                        max: 0.7,
                        step: 0.02,
                        value: graph_theme.node_shadow_alpha,
                        on_change: {
                            let store = theme_store;
                            move |value| update_graph_theme(store, |theme| theme.node_shadow_alpha = value)
                        },
                    }
                }
            }
            button {
                title: "Graph settings",
                style: "
                    width: 28px; height: 28px;
                    border-radius: 6px;
                    border: 1px solid rgba(255,255,255,0.08);
                    background: rgba(0,0,0,0.30);
                    color: rgba(255,255,255,0.45);
                    font-size: 14px;
                    cursor: pointer;
                    display: flex; align-items: center; justify-content: center;
                    backdrop-filter: blur(4px);
                    -webkit-backdrop-filter: blur(4px);
                    padding: 0;
                    line-height: 1;
                ",
                onclick: move |event| {
                    event.stop_propagation();
                    let current = *open.read();
                    *open.write() = !current;
                },
                "\u{2699}"
            }
        }
    }
}
