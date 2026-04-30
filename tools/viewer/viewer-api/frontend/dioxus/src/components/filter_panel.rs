//! [`FilterPanel`] — collapsible filter shell with preset chips, a custom
//! query input, and an optional results list.
//!
//! This component is **viewer-agnostic**: it owns presentation but no
//! filtering logic.  Each consuming viewer wires its own backend to:
//!   1. Translate the [`FilterState`] map into a query/JQ expression.
//!   2. Execute the query (server-side or in-memory).
//!   3. Push the resulting [`FilterResult`] list back into the panel.
//!
//! Browser-side state (collapsed/expanded, currently-typed query) is held
//! in `state: Signal<FilterState>` so the parent can read or mutate it.
use std::collections::BTreeMap;

use dioxus::prelude::*;

use crate::components::{ChevronDownIcon, ChevronRightIcon, CloseIcon, FilterIcon, Spinner, SpinnerSize};

/// Mutable filter state shared between the panel and its host.
///
/// `dropdowns` holds opaque `key → value` pairs for each "basic" filter
/// dropdown; the host decides what keys are meaningful.  `query` holds
/// the custom JQ / DSL string the user has typed.
#[derive(Clone, Default, PartialEq, Debug)]
pub struct FilterState {
    pub dropdowns: BTreeMap<String, String>,
    pub query: String,
    pub collapsed: bool,
}

impl FilterState {
    /// `true` when no dropdowns are set and the query is empty.
    pub fn is_empty(&self) -> bool {
        self.dropdowns.is_empty() && self.query.trim().is_empty()
    }

    /// Reset all dropdowns and the query string (collapsed state preserved).
    pub fn clear(&mut self) {
        self.dropdowns.clear();
        self.query.clear();
    }

    /// Number of active (non-empty) entries.
    pub fn active_count(&self) -> usize {
        self.dropdowns.values().filter(|v| !v.is_empty()).count()
            + usize::from(!self.query.trim().is_empty())
    }
}

/// A canned query the user can apply with one click.
#[derive(Clone, PartialEq, Debug)]
pub struct FilterPreset {
    pub label: String,
    pub jq: String,
}

impl FilterPreset {
    pub fn new(label: impl Into<String>, jq: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            jq: jq.into(),
        }
    }
}

/// One row in the results list.
#[derive(Clone, PartialEq, Debug)]
pub struct FilterResult {
    pub id: String,
    pub title: String,
    pub summary: Option<String>,
}

impl FilterResult {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            summary: None,
        }
    }

    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }
}

/// Filter UI shell.  See module docs for the consumer wiring contract.
#[component]
pub fn FilterPanel(
    /// Shared, mutable filter state.
    state: Signal<FilterState>,
    /// Optional preset chips rendered above the custom query input.
    #[props(default)]
    presets: Vec<FilterPreset>,
    /// Fires whenever `state.query` changes (debouncing is the caller's
    /// responsibility).
    #[props(default)]
    on_query_change: EventHandler<String>,
    /// Optional results list.  `None` hides the list section entirely;
    /// `Some(vec![])` shows an "no matches" empty state.
    #[props(default)]
    results: Option<Vec<FilterResult>>,
    /// Fires with the result id when a row is clicked.
    #[props(default)]
    on_result_click: EventHandler<String>,
    /// Show a spinner over the results list.
    #[props(default = false)]
    loading: bool,
    /// Optional title shown in the panel header.
    #[props(default = "Filters".to_string())]
    title: String,
    /// Extra CSS classes appended to the root `.filter-panel`.
    #[props(default)]
    class: String,
) -> Element {
    let active_count = state.read().active_count();
    let collapsed = state.read().collapsed;

    let root_class = if class.is_empty() {
        "filter-panel".to_string()
    } else {
        format!("filter-panel {class}")
    };

    rsx! {
        section {
            class: "{root_class}",

            // ── Header ──────────────────────────────────────────────────
            header {
                class: "filter-panel__header",
                button {
                    r#type: "button",
                    class: "filter-panel__toggle",
                    aria_expanded: !collapsed,
                    onclick: move |_| {
                        let mut s = state.write();
                        s.collapsed = !s.collapsed;
                    },
                    if collapsed {
                        ChevronRightIcon { size: 14 }
                    } else {
                        ChevronDownIcon { size: 14 }
                    }
                    FilterIcon { size: 14 }
                    span { class: "filter-panel__title", "{title}" }
                    if active_count > 0 {
                        span { class: "filter-panel__badge", "{active_count}" }
                    }
                }
                if active_count > 0 {
                    button {
                        r#type: "button",
                        class: "filter-panel__clear",
                        title: "Clear filters",
                        aria_label: "Clear filters",
                        onclick: move |_| {
                            state.write().clear();
                            on_query_change.call(String::new());
                        },
                        CloseIcon { size: 12 }
                    }
                }
            }

            // ── Body ────────────────────────────────────────────────────
            if !collapsed {
                div {
                    class: "filter-panel__body",

                    if !presets.is_empty() {
                        FilterPresets {
                            presets,
                            state,
                            on_query_change,
                        }
                    }

                    div {
                        class: "filter-panel__field",
                        label {
                            class: "filter-panel__label",
                            r#for: "filter-panel-query",
                            "Custom query"
                        }
                        textarea {
                            id: "filter-panel-query",
                            class: "filter-panel__input",
                            rows: 3,
                            placeholder: ".[] | select(.state == \"open\")",
                            value: "{state.read().query}",
                            oninput: move |e| {
                                let value = e.value();
                                state.write().query = value.clone();
                                on_query_change.call(value);
                            },
                        }
                    }

                    if let Some(results) = results {
                        FilterResultsList {
                            results,
                            on_result_click,
                            loading,
                        }
                    } else if loading {
                        div {
                            class: "filter-panel__loading",
                            Spinner { size: SpinnerSize::Sm }
                        }
                    }
                }
            }
        }
    }
}

/// Internal: chip row for preset queries.
#[component]
fn FilterPresets(
    presets: Vec<FilterPreset>,
    state: Signal<FilterState>,
    on_query_change: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            class: "filter-panel__presets",
            for preset in presets {
                {
                    let label = preset.label.clone();
                    let jq = preset.jq.clone();
                    let active = state.read().query == jq;
                    let chip_class = if active {
                        "filter-panel__preset filter-panel__preset--active"
                    } else {
                        "filter-panel__preset"
                    };
                    rsx! {
                        button {
                            r#type: "button",
                            key: "{label}",
                            class: "{chip_class}",
                            onclick: move |_| {
                                let q = jq.clone();
                                state.write().query = q.clone();
                                on_query_change.call(q);
                            },
                            "{label}"
                        }
                    }
                }
            }
        }
    }
}

/// Internal: vertical list of result rows.
#[component]
fn FilterResultsList(
    results: Vec<FilterResult>,
    on_result_click: EventHandler<String>,
    loading: bool,
) -> Element {
    rsx! {
        div {
            class: "filter-panel__results",

            if loading {
                div {
                    class: "filter-panel__results-loading",
                    Spinner { size: SpinnerSize::Sm }
                }
            }

            if results.is_empty() && !loading {
                p { class: "filter-panel__empty", "No matches." }
            }

            for r in results {
                {
                    let id = r.id.clone();
                    rsx! {
                        button {
                            r#type: "button",
                            key: "{r.id}",
                            class: "filter-panel__result",
                            onclick: move |_| on_result_click.call(id.clone()),
                            div { class: "filter-panel__result-title", "{r.title}" }
                            if let Some(summary) = r.summary {
                                div {
                                    class: "filter-panel__result-summary",
                                    "{summary}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_state_detection() {
        let s = FilterState::default();
        assert!(s.is_empty());
        assert_eq!(s.active_count(), 0);
    }

    #[test]
    fn active_count_combines_dropdowns_and_query() {
        let mut s = FilterState::default();
        s.dropdowns.insert("state".into(), "open".into());
        s.dropdowns.insert("type".into(), String::new()); // empty value: ignored
        s.query = ".x | select(.y)".into();
        assert_eq!(s.active_count(), 2);
    }

    #[test]
    fn clear_resets_query_and_dropdowns_but_keeps_collapsed() {
        let mut s = FilterState {
            collapsed: true,
            ..Default::default()
        };
        s.dropdowns.insert("a".into(), "b".into());
        s.query = "q".into();
        s.clear();
        assert!(s.dropdowns.is_empty());
        assert_eq!(s.query, "");
        assert!(s.collapsed, "collapsed flag must survive clear()");
    }
}
