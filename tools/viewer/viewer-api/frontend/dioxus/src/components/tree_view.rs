//! TreeView and FileTree components.
//!
//! [`TreeView`] renders a recursive, keyboard-accessible tree of [`TreeNode`]s
//! with expand/collapse and single-selection state stored in Dioxus signals.
//!
//! **Multi-select** (opt-in via `show_checkboxes = true`):
//!  - A checkbox is rendered before each node label.
//!  - Selected IDs are maintained in a `Signal<BTreeSet<String>>` and exposed
//!    via the `on_selection_change` callback.
//!  - Shift+click selects the range between the last-clicked node and the
//!    current one, ordered by visible render position.
//!  - Space-bar toggles the focused node.
//!  - Arrow-up / Arrow-down move keyboard focus.
//!  - Ctrl+A selects all visible nodes.
//!
//! [`FileTree`] wraps [`TreeView`] and adds:
//!  - A multi-key sort header (`.file-tree__sort-header`).
//!  - Filter buttons with per-filter item counts and color badges
//!    (`.file-tree__filter-header`).
//!  - Loading and empty states.
//!
//! CSS class names mirror the TypeScript viewer-api package.
use std::collections::BTreeSet;

use dioxus::prelude::*;

use crate::components::{ChevronRightIcon, FileIcon, FilterIcon, FolderIcon, FolderOpenIcon, Spinner};

// ── Data types ────────────────────────────────────────────────────────────────

/// A single node in the tree.
#[derive(Clone, PartialEq)]
pub struct TreeNode {
    /// Unique identifier used as the React-style `key` and selection value.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Optional badge text shown after the label (e.g. a count).
    pub badge: Option<String>,
    /// Optional tooltip text.
    pub tooltip: Option<String>,
    /// CSS colour string applied to the badge.
    pub badge_color: Option<String>,
    /// Whether this node is a directory / group or a leaf.
    pub is_dir: bool,
    /// Child nodes (only meaningful when `is_dir` is true).
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    /// Create a leaf node.
    pub fn leaf(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            badge: None,
            tooltip: None,
            badge_color: None,
            is_dir: false,
            children: vec![],
        }
    }

    /// Create a directory node.
    pub fn dir(id: impl Into<String>, label: impl Into<String>, children: Vec<TreeNode>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            badge: None,
            tooltip: None,
            badge_color: None,
            is_dir: true,
            children,
        }
    }
}

// ── Sort keys ─────────────────────────────────────────────────────────────────

/// Multi-key sort descriptor for [`FileTree`].
#[derive(Clone, PartialEq, Debug)]
pub struct SortKey {
    /// Column identifier / label shown in the sort header button.
    pub key: String,
    /// Human-readable display label.
    pub label: String,
    /// Ascending (`true`) or descending.
    pub ascending: bool,
}

impl SortKey {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            ascending: true,
        }
    }
}

// ── Filter definitions ────────────────────────────────────────────────────────

/// A filter button descriptor for [`FileTree`].
#[derive(Clone, PartialEq)]
pub struct FilterDef {
    /// Unique key used to track active state.
    pub key: String,
    /// Display label on the button.
    pub label: String,
    /// Number of items matching this filter.
    pub count: usize,
    /// CSS colour for the count badge (e.g. `"var(--accent-green)"`).
    pub color: Option<String>,
}

impl FilterDef {
    pub fn new(key: impl Into<String>, label: impl Into<String>, count: usize) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            count,
            color: None,
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Flatten all *visible* node IDs into DFS render order, respecting the
/// current expanded set.  Used for shift+click range selection and keyboard
/// navigation.
fn collect_visible_ids(nodes: &[TreeNode], expanded_ids: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    for node in nodes {
        result.push(node.id.clone());
        if node.is_dir && expanded_ids.contains(&node.id) {
            result.extend(collect_visible_ids(&node.children, expanded_ids));
        }
    }
    result
}

// ── Internal recursive item ───────────────────────────────────────────────────

/// Renders a single tree row plus its (optionally visible) children.
#[component]
fn TreeItem(
    node: TreeNode,
    depth: usize,
    expanded_ids: Signal<Vec<String>>,
    selected_id: Signal<Option<String>>,
    on_select: EventHandler<String>,
    // ── Multi-select ──────────────────────────────────────────────────────────
    /// When true a checkbox is shown and multi-select logic is active.
    show_checkboxes: bool,
    /// Set of currently selected IDs (multi-select mode).
    multi_selected: Signal<BTreeSet<String>>,
    /// ID of the keyboard-focused node.
    focused_id: Signal<Option<String>>,
    /// Anchor node for shift+click range selection.
    last_clicked: Signal<Option<String>>,
    /// Visible node IDs in render order (computed by [`TreeView`]).
    visible_order: Memo<Vec<String>>,
    /// Called with the new selection set whenever it changes (multi-select mode).
    on_selection_change: EventHandler<BTreeSet<String>>,
) -> Element {
    let node_id = node.id.clone();
    let node_id_sel = node.id.clone();
    let node_id_multi = node.id.clone();
    let node_id_focus = node.id.clone();
    let is_expanded = use_memo(move || expanded_ids.read().contains(&node_id));
    let is_selected = use_memo(move || {
        selected_id
            .read()
            .as_deref()
            .map_or(false, |s| s == node_id_sel)
    });
    let is_in_multi = use_memo(move || multi_selected.read().contains(&node_id_multi));
    let is_focused = use_memo(move || {
        focused_id.read().as_deref() == Some(node_id_focus.as_str())
    });

    let row_class = use_memo(move || {
        let selected = if show_checkboxes {
            *is_in_multi.read()
        } else {
            *is_selected.read()
        };
        match (selected, *is_focused.read()) {
            (true, true) => "tree-item-row selected focused",
            (true, false) => "tree-item-row selected",
            (false, true) => "tree-item-row focused",
            (false, false) => "tree-item-row",
        }
    });

    let toggle_id = node.id.clone();
    let indent = format!("padding-left: {}px", depth * 16);

    // Icon selection
    let icon: Element = if node.is_dir {
        if *is_expanded.read() {
            rsx! { FolderOpenIcon { size: 16, class: "tree-icon folder" } }
        } else {
            rsx! { FolderIcon { size: 16, class: "tree-icon folder" } }
        }
    } else {
        rsx! { FileIcon { size: 16, class: "tree-icon file" } }
    };

    let click_id = node.id.clone();
    let click_id2 = node.id.clone();
    let has_children = !node.children.is_empty();

    rsx! {
        div {
            class: "tree-item",
            // Row
            div {
                class: "{row_class}",
                style: "{indent}",
                role: "treeitem",
                aria_expanded: if node.is_dir { Some(*is_expanded.read()) } else { None },
                aria_selected: if show_checkboxes { *is_in_multi.read() } else { *is_selected.read() },
                title: node.tooltip.as_deref().unwrap_or(""),
                // Toggle chevron
                span {
                    class: if has_children {
                        if *is_expanded.read() { "tree-toggle expanded" } else { "tree-toggle" }
                    } else {
                        "tree-toggle empty"
                    },
                    onclick: move |e| {
                        e.stop_propagation();
                        if has_children {
                            let mut ids = expanded_ids.write();
                            if let Some(pos) = ids.iter().position(|id| id == &toggle_id) {
                                ids.remove(pos);
                            } else {
                                ids.push(toggle_id.clone());
                            }
                        }
                    },
                    ChevronRightIcon { size: 12 }
                }
                // Checkbox (multi-select mode only)
                if show_checkboxes {
                    input {
                        r#type: "checkbox",
                        class: "tree-checkbox",
                        checked: *is_in_multi.read(),
                        // Row label onclick handles actual toggling; prevent double-fire.
                        onclick: move |e| e.stop_propagation(),
                    }
                }
                // Item icon
                {icon}
                // Label — handles both single-select and multi-select clicks
                span {
                    class: "tree-label",
                    onclick: move |evt: Event<MouseData>| {
                        if show_checkboxes {
                            let shift = evt.modifiers().contains(Modifiers::SHIFT);
                            let new_selection: BTreeSet<String> = if shift {
                                // Range selection: find anchor..current in visible order.
                                let order = visible_order.read();
                                let anchor = last_clicked
                                    .read()
                                    .clone()
                                    .unwrap_or_else(|| click_id.clone());
                                let from = order.iter().position(|id| id == &anchor);
                                let to = order.iter().position(|id| id == &click_id);
                                match (from, to) {
                                    (Some(a), Some(b)) => {
                                        let (start, end) = if a <= b { (a, b) } else { (b, a) };
                                        order[start..=end].iter().cloned().collect()
                                    }
                                    _ => std::iter::once(click_id.clone()).collect(),
                                }
                            } else {
                                last_clicked.set(Some(click_id.clone()));
                                std::iter::once(click_id.clone()).collect()
                            };
                            multi_selected.set(new_selection.clone());
                            focused_id.set(Some(click_id.clone()));
                            on_selection_change.call(new_selection);
                        } else {
                            selected_id.set(Some(click_id2.clone()));
                            on_select.call(click_id2.clone());
                        }
                    },
                    "{node.label}"
                }
                // Optional badge
                if let Some(badge) = &node.badge {
                    span {
                        class: "tree-badge",
                        style: node.badge_color.as_deref().map(|c| format!("color: {c}")).unwrap_or_default(),
                        "{badge}"
                    }
                }
            }
            // Children (only rendered when expanded)
            if *is_expanded.read() && !node.children.is_empty() {
                div {
                    class: "tree-children",
                    for child in node.children.clone() {
                        TreeItem {
                            key: "{child.id}",
                            node: child,
                            depth: depth + 1,
                            expanded_ids,
                            selected_id,
                            on_select,
                            show_checkboxes,
                            multi_selected,
                            focused_id,
                            last_clicked,
                            visible_order,
                            on_selection_change,
                        }
                    }
                }
            }
        }
    }
}

// ── TreeView ──────────────────────────────────────────────────────────────────

/// Recursive tree with Dioxus-signal-backed expand/collapse and selection.
///
/// Pass pre-built [`TreeNode`] roots in `nodes`.  Optionally control the
/// selected node via `selected_id` and listen to changes with `on_select`.
///
/// ## Multi-select
///
/// Set `show_checkboxes = true` to enable multi-select mode.  In that mode:
/// - A checkbox is rendered before each node label.
/// - The selection is maintained internally as a `BTreeSet<String>` of IDs.
/// - `on_selection_change` fires whenever the selection changes.
/// - Shift+click extends the selection to a range.
/// - Space toggles the focused node; ↑↓ move focus; Ctrl+A selects all.
#[component]
pub fn TreeView(
    /// Root-level tree nodes.
    nodes: Vec<TreeNode>,
    /// Currently selected node id (uncontrolled if `None`, single-select mode).
    #[props(default)]
    selected_id: Option<String>,
    /// Called when a node is clicked (single-select mode).
    #[props(default)]
    on_select: EventHandler<String>,
    /// Initially expanded node ids.
    #[props(default)]
    initially_expanded: Vec<String>,
    /// Extra CSS classes on the `.tree-view` div.
    #[props(default)]
    class: String,
    /// When true, a checkbox is shown before each label and multi-select is active.
    #[props(default = false)]
    show_checkboxes: bool,
    /// Called with the full selection set whenever it changes (multi-select mode).
    #[props(default)]
    on_selection_change: EventHandler<BTreeSet<String>>,
) -> Element {
    let expanded_ids = use_signal(|| initially_expanded);
    let sel_id: Signal<Option<String>> = use_signal(|| selected_id);
    let mut multi_selected: Signal<BTreeSet<String>> = use_signal(BTreeSet::new);
    let mut focused_id: Signal<Option<String>> = use_signal(|| None);
    let last_clicked: Signal<Option<String>> = use_signal(|| None);

    // Flatten visible IDs in render order; recomputed when expansion changes.
    let nodes_for_order = nodes.clone();
    let visible_order: Memo<Vec<String>> =
        use_memo(move || collect_visible_ids(&nodes_for_order, &expanded_ids.read()));

    let combined = if class.is_empty() {
        "tree-view".to_string()
    } else {
        format!("tree-view {class}")
    };

    rsx! {
        div {
            class: "{combined}",
            role: "tree",
            // tabindex makes the container focusable for keyboard events.
            tabindex: if show_checkboxes { 0 } else { -1 },
            onkeydown: move |e: Event<KeyboardData>| {
                if !show_checkboxes {
                    return;
                }
                let order = visible_order.read();
                if order.is_empty() {
                    return;
                }
                let current_focus = focused_id.read().clone();
                match e.key() {
                    Key::ArrowDown => {
                        e.prevent_default();
                        let next = match &current_focus {
                            Some(id) => order
                                .iter()
                                .position(|x| x == id)
                                .and_then(|pos| order.get(pos + 1))
                                .cloned(),
                            None => order.first().cloned(),
                        };
                        if let Some(id) = next {
                            focused_id.set(Some(id));
                        }
                    }
                    Key::ArrowUp => {
                        e.prevent_default();
                        let prev = match &current_focus {
                            Some(id) => order
                                .iter()
                                .position(|x| x == id)
                                .filter(|&pos| pos > 0)
                                .and_then(|pos| order.get(pos - 1))
                                .cloned(),
                            None => order.last().cloned(),
                        };
                        if let Some(id) = prev {
                            focused_id.set(Some(id));
                        }
                    }
                    Key::Character(ref s) if s == " " => {
                        e.prevent_default();
                        let focused = current_focus.clone();
                        drop(order);
                        if let Some(id) = focused {
                            {
                                let mut sel = multi_selected.write();
                                if sel.contains(&id) {
                                    sel.remove(&id);
                                } else {
                                    sel.insert(id);
                                }
                            }
                            on_selection_change.call(multi_selected.read().clone());
                        }
                    }
                    Key::Character(ref s) if s == "a" => {
                        if e.modifiers().contains(Modifiers::CONTROL) {
                            e.prevent_default();
                            let all: BTreeSet<String> = order.iter().cloned().collect();
                            drop(order);
                            multi_selected.set(all.clone());
                            on_selection_change.call(all);
                        }
                    }
                    _ => {}
                }
            },
            for node in nodes {
                TreeItem {
                    key: "{node.id}",
                    node,
                    depth: 0,
                    expanded_ids,
                    selected_id: sel_id,
                    on_select,
                    show_checkboxes,
                    multi_selected,
                    focused_id,
                    last_clicked,
                    visible_order,
                    on_selection_change,
                }
            }
        }
    }
}

// ── FileTree ──────────────────────────────────────────────────────────────────

/// FileTree: wraps [`TreeView`] with sort/filter headers and loading/empty states.
///
/// Renders:
/// - `.file-tree__sort-header` — row of sort buttons; active key indicated by
///   `.file-tree__sort-btn--active`.
/// - `.file-tree__filter-header` — column of filter toggle buttons with count
///   badges and per-filter accent colours.
/// - Loading state (`.file-tree__loading`) when `loading` is true.
/// - Empty state (`.file-tree__empty`) when `nodes` is empty and not loading.
/// - [`TreeView`] otherwise.
#[component]
pub fn FileTree(
    /// Tree data.
    nodes: Vec<TreeNode>,
    /// Available sort columns.  The first active SortKey is the primary sort.
    #[props(default)]
    sort_keys: Vec<SortKey>,
    /// Called when a sort key button is clicked.  Callers apply the sort.
    #[props(default)]
    on_sort: EventHandler<String>,
    /// Available filter buttons.
    #[props(default)]
    filters: Vec<FilterDef>,
    /// Currently active filter keys.
    #[props(default)]
    active_filters: Vec<String>,
    /// Called when a filter button is toggled.
    #[props(default)]
    on_filter: EventHandler<String>,
    /// Currently selected node id.
    #[props(default)]
    selected_id: Option<String>,
    /// Called when a node is selected.
    #[props(default)]
    on_select: EventHandler<String>,
    /// Initially expanded node ids.
    #[props(default)]
    initially_expanded: Vec<String>,
    /// Show loading spinner instead of tree content.
    #[props(default = false)]
    loading: bool,
    /// Extra CSS classes on the outermost `.file-tree` div.
    #[props(default)]
    class: String,
    /// When true, checkboxes are shown and multi-select is active.
    #[props(default = false)]
    show_checkboxes: bool,
    /// Called with the full selection set whenever it changes (multi-select mode).
    #[props(default)]
    on_selection_change: EventHandler<BTreeSet<String>>,
) -> Element {
    let combined = if class.is_empty() {
        "file-tree".to_string()
    } else {
        format!("file-tree {class}")
    };

    rsx! {
        div {
            class: "{combined}",

            // ── Sort header ──
            if !sort_keys.is_empty() {
                div {
                    class: "file-tree__sort-header",
                    for sk in &sort_keys {
                        {
                            let key = sk.key.clone();
                            let btn_class = if sk.ascending {
                                "file-tree__sort-btn file-tree__sort-btn--active"
                            } else {
                                "file-tree__sort-btn"
                            };
                            rsx! {
                                button {
                                    key: "{sk.key}",
                                    class: "{btn_class}",
                                    onclick: move |_| on_sort.call(key.clone()),
                                    "{sk.label}"
                                    if sk.ascending { " ↑" } else { " ↓" }
                                }
                            }
                        }
                    }
                }
            }

            // ── Filter header ──
            if !filters.is_empty() {
                div {
                    class: "file-tree__filter-header",
                    for f in &filters {
                        {
                            let fkey = f.key.clone();
                            let is_active = active_filters.contains(&f.key);
                            let btn_class = if is_active {
                                "file-tree__filter-btn file-tree__filter-btn--active"
                            } else {
                                "file-tree__filter-btn"
                            };
                            let badge_style = f.color.as_deref()
                                .map(|c| format!("color: {c}"))
                                .unwrap_or_default();
                            rsx! {
                                button {
                                    key: "{f.key}",
                                    class: "{btn_class}",
                                    onclick: move |_| on_filter.call(fkey.clone()),
                                    span {
                                        class: "file-tree__filter-icon",
                                        FilterIcon { size: 12 }
                                    }
                                    span { class: "file-tree__filter-label", "{f.label}" }
                                    span {
                                        class: "tree-badge",
                                        style: "{badge_style}",
                                        "{f.count}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ── Body ──
            if loading {
                div {
                    class: "file-tree__loading",
                    Spinner {}
                    span { " Loading…" }
                }
            } else if nodes.is_empty() {
                div {
                    class: "file-tree__empty",
                    "No items to display."
                }
            } else {
                TreeView {
                    nodes,
                    selected_id,
                    on_select,
                    initially_expanded,
                    show_checkboxes,
                    on_selection_change,
                }
            }
        }
    }
}
