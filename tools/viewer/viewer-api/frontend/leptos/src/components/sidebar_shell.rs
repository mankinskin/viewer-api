/// SidebarShell — generic sidebar wrapper with header, badge, collapse toggle,
/// and an integrated [`ResizeHandle`] on the trailing edge.
///
/// Embed your sidebar content as `children`.
///
/// # CSS classes
///   `.va-sidebar-shell`, `.va-sidebar-shell-body`, `.va-sidebar-shell-header`,
///   `.va-sidebar-shell-title`, `.va-sidebar-shell-badge`, `.va-sidebar-collapse-btn`
use leptos::prelude::*;

use crate::components::resize_handle::ResizeHandle;

/// Default width if none provided.
const DEFAULT_WIDTH: f64 = 260.0;
const MIN_WIDTH: f64 = 80.0;

/// Generic sidebar shell.
///
/// # Props
/// * `title` — header title text.
/// * `badge` (optional) — numeric badge shown next to the title.
/// * `default_width` (optional) — initial width in CSS pixels (default `260.0`).
/// * `children` — slot for sidebar body content.
#[component]
pub fn SidebarShell(
    #[prop(into)] title: String,
    #[prop(optional)] badge: Option<usize>,
    #[prop(optional)] default_width: Option<f64>,
    children: Children,
) -> impl IntoView {
    let width = RwSignal::new(default_width.unwrap_or(DEFAULT_WIDTH));
    let collapsed = RwSignal::new(false);

    let on_resize = move |delta: f64| {
        width.update(|w| *w = (*w + delta).max(MIN_WIDTH));
    };

    let style = move || {
        if collapsed.get() {
            "width: 0; overflow: hidden; min-width: 0;".to_string()
        } else {
            format!("width: {}px; position: relative;", width.get())
        }
    };

    view! {
        <div class="va-sidebar-shell" style=style>
            <div class="va-sidebar-shell-header">
                <span class="va-sidebar-shell-title">
                    {title}
                    {badge.map(|n| view! {
                        <span class="va-sidebar-shell-badge">{n}</span>
                    })}
                </span>
                <button
                    class="va-sidebar-collapse-btn"
                    title=move || if collapsed.get() { "Expand" } else { "Collapse" }
                    on:click=move |_| collapsed.update(|c| *c = !*c)
                >
                    {move || if collapsed.get() { "›" } else { "‹" }}
                </button>
            </div>
            <div class="va-sidebar-shell-body">
                {children()}
            </div>
            <ResizeHandle on_resize=on_resize />
        </div>
    }
}
