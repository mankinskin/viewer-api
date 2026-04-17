/// TreeView — collapsible, icon-decorated tree component.
///
/// Each node carries an optional badge (e.g. file size, item count).
/// Nodes with children show a chevron `›` that rotates 90° when expanded.
/// Depth indentation is 8 px per level plus a 8 px base left padding.
///
/// CSS classes emitted (prefix `va-tree-`):
///   `.va-tree-view`, `.va-tree-item-group`,
///   `.va-tree-item`, `.va-tree-item-selected`,
///   `.va-tree-chevron`, `.va-tree-chevron-expanded`,
///   `.va-tree-icon`, `.va-tree-label`, `.va-tree-badge`
use leptos::prelude::*;

// ── Public types ──────────────────────────────────────────────────────────────

/// Icon variant used by [`TreeNode`].
#[derive(Clone, Debug, PartialEq, Default)]
pub enum NodeIcon {
    /// Folder — rendered as 📁
    Folder,
    /// File — rendered as 📄
    File,
    /// No icon
    #[default]
    None,
}

/// A single node in the tree.
///
/// Nodes are cloned when the tree re-renders; keep them cheap to clone
/// (use `Arc<str>` instead of `String` for large trees).
#[derive(Clone, Debug, Default)]
pub struct TreeNode {
    /// Unique, stable node identifier used as the expand/select key.
    pub id: String,
    /// Display label shown next to the icon.
    pub label: String,
    /// Optional icon variant.
    pub icon: NodeIcon,
    /// Optional right-aligned badge text (e.g. `"42"`, `"1.2 MiB"`).
    pub badge: Option<String>,
    /// Nested children; empty for leaf nodes.
    pub children: Vec<TreeNode>,
}

// ── Callback type alias ───────────────────────────────────────────────────────

/// `StoredValue` backed by `LocalStorage` so it is `Copy + Send + Sync` while
/// safely holding a `!Send` closure (wasm is single-threaded).
type SelectCb = StoredValue<Box<dyn Fn(String) + 'static>, LocalStorage>;

// ── Internal recursive renderer ───────────────────────────────────────────────

fn tree_item(
    node: TreeNode,
    depth: usize,
    expanded: RwSignal<std::collections::HashSet<String>>,
    selected: RwSignal<Option<String>>,
    on_select: Option<SelectCb>,
) -> AnyView {
    let has_children = !node.children.is_empty();

    let id = node.id.clone();
    let label = node.label.clone();
    let icon = node.icon.clone();
    let badge = node.badge.clone();
    let children = node.children.clone();

    // Derived signals — two closures needed (each move captures by value)
    let id_exp1 = id.clone();
    let id_exp2 = id.clone();
    let is_expanded = move || expanded.with(|e| e.contains(&id_exp1));
    let is_expanded2 = move || expanded.with(|e| e.contains(&id_exp2));

    let id_sel = id.clone();
    let is_selected = move || selected.with(|s| s.as_deref() == Some(id_sel.as_str()));

    // Interaction
    let id_toggle = id.clone();
    let id_select = id.clone();
    let on_click = move |_| {
        if has_children {
            expanded.update(|e| {
                if e.contains(&id_toggle) {
                    e.remove(&id_toggle);
                } else {
                    e.insert(id_toggle.clone());
                }
            });
        }
        selected.set(Some(id_select.clone()));
        if let Some(cb) = on_select {
            cb.with_value(|f| f(id_select.clone()));
        }
    };

    let indent = format!("padding-left: {}px", depth * 8 + 8);

    let icon_char = match icon {
        NodeIcon::Folder => "📁",
        NodeIcon::File => "📄",
        NodeIcon::None => "",
    };

    // Render children list (conditionally shown).
    // `on_select` is `Option<SelectCb>` which is `Copy`, so the closure is `Send`.
    let children_depth = depth + 1;
    let children_view = move || {
        if !is_expanded2() || children.is_empty() {
            return None;
        }
        let views: Vec<AnyView> = children
            .iter()
            .map(|child| {
                tree_item(child.clone(), children_depth, expanded, selected, on_select)
            })
            .collect();
        Some(views)
    };

    view! {
        <div class="va-tree-item-group">
            <div
                class="va-tree-item"
                class:va-tree-item-selected=is_selected
                style=indent
                on:click=on_click
            >
                <span
                    class="va-tree-chevron"
                    class:va-tree-chevron-expanded=is_expanded
                    style=if has_children { "" } else { "visibility:hidden" }
                >
                    "›"
                </span>
                {(!icon_char.is_empty()).then(|| view! {
                    <span class="va-tree-icon">{icon_char}</span>
                })}
                <span class="va-tree-label">{label}</span>
                {badge.map(|b| view! {
                    <span class="va-tree-badge">{b}</span>
                })}
            </div>
            {children_view}
        </div>
    }
    .into_any()
}

// ── Public component ──────────────────────────────────────────────────────────

/// Collapsible icon-decorated tree view.
///
/// # Props
/// * `nodes` — root-level nodes.
/// * `on_select` (optional) — callback invoked with the selected node's `id`.
#[component]
pub fn TreeView(
    nodes: Vec<TreeNode>,
    #[prop(optional, into)] on_select: Option<Box<dyn Fn(String) + 'static>>,
) -> impl IntoView {
    let expanded: RwSignal<std::collections::HashSet<String>> =
        RwSignal::new(std::collections::HashSet::new());
    let selected: RwSignal<Option<String>> = RwSignal::new(None);

    // `StoredValue::new_local` only requires `T: 'static` (no Send+Sync needed),
    // yet `StoredValue` itself is `Copy + Send + Sync`, safe for `<For>` children closures.
    let cb_sv: Option<SelectCb> = on_select.map(StoredValue::new_local);

    view! {
        <div class="va-tree-view">
            <For
                each=move || nodes.clone()
                key=|n| n.id.clone()
                children=move |node| tree_item(node, 0, expanded, selected, cb_sv)
            />
        </div>
    }
}
