use dioxus::prelude::*;

use super::{LayoutMode, Projection};

#[derive(Props, Clone, PartialEq)]
pub(super) struct GraphSettingsOverlayProps {
    pub layout_mode: LayoutMode,
    pub projection: Projection,
    pub on_layout_mode_change: Option<EventHandler<LayoutMode>>,
    pub on_projection_change: Option<EventHandler<Projection>>,
}

fn opt_btn_style(active: bool) -> String {
    let (background, border, color) = if active {
        ("rgba(79,140,255,0.20)", "1px solid rgba(79,140,255,0.50)", "#93bbff")
    } else {
        ("rgba(255,255,255,0.05)", "1px solid rgba(255,255,255,0.10)", "#aaa")
    };

    format!(
        "flex:1; padding:5px 0; border-radius:5px; border:{border}; \
         background:{background}; color:{color}; font-size:11px; font-weight:500; \
         cursor:pointer; text-align:center; white-space:nowrap;"
    )
}

#[component]
pub(super) fn GraphSettingsOverlay(props: GraphSettingsOverlayProps) -> Element {
    let mut open: Signal<bool> = use_hook(|| Signal::new(false));
    let cur_layout = props.layout_mode;
    let cur_proj = props.projection;

    let has_callbacks =
        props.on_layout_mode_change.is_some() || props.on_projection_change.is_some();
    if !has_callbacks {
        return rsx! {};
    }

    let on_layout_mode_change = props.on_layout_mode_change.clone();
    let on_projection_change = props.on_projection_change.clone();

    rsx! {
        div {
            style: "position: absolute; bottom: 12px; right: 12px; z-index: 100; display: flex; flex-direction: column; align-items: flex-end;",
            if *open.read() {
                div {
                    style: "
                        margin-bottom: 6px;
                        min-width: 200px;
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