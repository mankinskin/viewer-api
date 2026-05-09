use std::collections::BTreeSet;

use dioxus::prelude::*;

use super::types::{
    NodeIcon,
    TreeNode,
};
use crate::components::{
    ChevronRightIcon,
    CrateIcon,
    DocumentIcon,
    FileIcon,
    FolderIcon,
    FolderOpenIcon,
    ModuleIcon,
    SourceFileIcon,
};

#[component]
pub(super) fn TreeItem(
    node: TreeNode,
    depth: usize,
    expanded_ids: Signal<Vec<String>>,
    selected_id: Option<String>,
    on_select: EventHandler<String>,
    show_checkboxes: bool,
    multi_selected: Signal<BTreeSet<String>>,
    focused_id: Signal<Option<String>>,
    last_clicked: Signal<Option<String>>,
    visible_order: Memo<Vec<String>>,
    on_selection_change: EventHandler<BTreeSet<String>>,
) -> Element {
    let expanded_id = node.id.clone();
    let selected_id_key = node.id.clone();
    let multi_id = node.id.clone();
    let focus_id_key = node.id.clone();
    let is_expanded =
        use_memo(move || expanded_ids.read().contains(&expanded_id));
    let is_selected = selected_id
        .as_deref()
        .is_some_and(|selected| selected == selected_id_key);
    let is_in_multi =
        use_memo(move || multi_selected.read().contains(&multi_id));
    let is_focused = use_memo(move || {
        focused_id.read().as_deref() == Some(focus_id_key.as_str())
    });

    let expanded = *is_expanded.read();
    let in_multi = *is_in_multi.read();
    let focused = *is_focused.read();
    let has_children = !node.children.is_empty();
    let is_dir_node = node.is_dir;
    let indent = format!("padding-left: {}px", depth * 16);
    let row_selected = if show_checkboxes {
        in_multi
    } else {
        is_selected
    };
    let row_class = tree_row_class(row_selected, focused);
    let toggle_class = tree_toggle_class(has_children, expanded);
    let icon = render_node_icon(&node, expanded);
    let badge = render_badge(&node);
    let tooltip = render_tooltip(&node);
    let children = render_children(
        expanded,
        &node.children,
        depth,
        expanded_ids,
        selected_id.clone(),
        on_select,
        show_checkboxes,
        multi_selected,
        focused_id,
        last_clicked,
        visible_order,
        on_selection_change,
    );

    let toggle_id = node.id.clone();
    let row_select_id = node.id.clone();
    let toggle_label_id = node.id.clone();
    let click_label_id = node.id.clone();

    rsx! {
        div {
            class: "tree-item",
            div {
                class: "{row_class}",
                style: "{indent}",
                role: "treeitem",
                aria_expanded: if node.is_dir { Some(expanded) } else { None },
                aria_selected: if show_checkboxes { in_multi } else { is_selected },
                title: node.tooltip.as_deref().unwrap_or(""),
                onclick: move |_| {
                    handle_row_click(
                        is_dir_node,
                        has_children,
                        show_checkboxes,
                        expanded_ids,
                        toggle_id.clone(),
                        on_select.clone(),
                        row_select_id.clone(),
                    );
                },
                span {
                    class: "{toggle_class}",
                    onclick: move |event| {
                        event.stop_propagation();
                        if has_children {
                            toggle_expanded(expanded_ids, &toggle_label_id);
                        }
                    },
                    ChevronRightIcon { size: 12 }
                }
                if show_checkboxes {
                    input {
                        r#type: "checkbox",
                        class: "tree-checkbox",
                        checked: in_multi,
                        onclick: move |event| event.stop_propagation(),
                    }
                }
                {icon}
                span {
                    class: "tree-label",
                    onclick: move |event: Event<MouseData>| {
                        event.stop_propagation();
                        handle_label_click(
                            event.modifiers().contains(Modifiers::SHIFT),
                            show_checkboxes,
                            is_dir_node,
                            has_children,
                            expanded_ids,
                            click_label_id.clone(),
                            on_select.clone(),
                            multi_selected,
                            focused_id,
                            last_clicked,
                            visible_order,
                            on_selection_change.clone(),
                        );
                    },
                    "{node.label}"
                }
                {badge}
                {tooltip}
            }
            {children}
        }
    }
}

fn handle_row_click(
    is_dir_node: bool,
    has_children: bool,
    show_checkboxes: bool,
    expanded_ids: Signal<Vec<String>>,
    toggle_id: String,
    on_select: EventHandler<String>,
    select_id: String,
) {
    if is_dir_node && has_children {
        toggle_expanded(expanded_ids, &toggle_id);
    } else if !is_dir_node && !show_checkboxes {
        on_select.call(select_id);
    }
}

fn handle_label_click(
    shift: bool,
    show_checkboxes: bool,
    is_dir_node: bool,
    has_children: bool,
    expanded_ids: Signal<Vec<String>>,
    node_id: String,
    on_select: EventHandler<String>,
    mut multi_selected: Signal<BTreeSet<String>>,
    mut focused_id: Signal<Option<String>>,
    last_clicked: Signal<Option<String>>,
    visible_order: Memo<Vec<String>>,
    on_selection_change: EventHandler<BTreeSet<String>>,
) {
    if show_checkboxes {
        let new_selection =
            next_multi_selection(&node_id, shift, last_clicked, visible_order);
        multi_selected.set(new_selection.clone());
        focused_id.set(Some(node_id));
        on_selection_change.call(new_selection);
        return;
    }

    if is_dir_node && has_children {
        toggle_expanded(expanded_ids, &node_id);
    } else {
        on_select.call(node_id);
    }
}

fn next_multi_selection(
    node_id: &str,
    shift: bool,
    mut last_clicked: Signal<Option<String>>,
    visible_order: Memo<Vec<String>>,
) -> BTreeSet<String> {
    if !shift {
        last_clicked.set(Some(node_id.to_string()));
        return std::iter::once(node_id.to_string()).collect();
    }

    let order = visible_order.read();
    let anchor = last_clicked
        .read()
        .clone()
        .unwrap_or_else(|| node_id.to_string());
    collect_range_selection(&order, &anchor, node_id)
}

fn collect_range_selection(
    order: &[String],
    anchor: &str,
    node_id: &str,
) -> BTreeSet<String> {
    let from = order.iter().position(|id| id == anchor);
    let to = order.iter().position(|id| id == node_id);

    match (from, to) {
        (Some(start), Some(end)) => {
            let (start, end) = if start <= end {
                (start, end)
            } else {
                (end, start)
            };
            order[start..=end].iter().cloned().collect()
        },
        _ => std::iter::once(node_id.to_string()).collect(),
    }
}

fn toggle_expanded(
    mut expanded_ids: Signal<Vec<String>>,
    node_id: &str,
) {
    let mut ids = expanded_ids.write();
    if let Some(position) = ids.iter().position(|id| id == node_id) {
        ids.remove(position);
    } else {
        ids.push(node_id.to_string());
    }
}

fn tree_row_class(
    row_selected: bool,
    is_focused: bool,
) -> &'static str {
    match (row_selected, is_focused) {
        (true, true) => "tree-item-row selected focused",
        (true, false) => "tree-item-row selected",
        (false, true) => "tree-item-row focused",
        (false, false) => "tree-item-row",
    }
}

fn tree_toggle_class(
    has_children: bool,
    is_expanded: bool,
) -> &'static str {
    if !has_children {
        return "tree-toggle empty";
    }

    if is_expanded {
        "tree-toggle expanded"
    } else {
        "tree-toggle"
    }
}

fn render_node_icon(
    node: &TreeNode,
    is_expanded: bool,
) -> Element {
    match node.icon {
        NodeIcon::Auto =>
            if node.is_dir {
                render_folder_icon(is_expanded)
            } else {
                rsx! { FileIcon { size: 16, class: "tree-icon file" } }
            },
        NodeIcon::Folder => render_folder_icon(is_expanded),
        NodeIcon::File =>
            rsx! { FileIcon { size: 16, class: "tree-icon file" } },
        NodeIcon::Doc =>
            rsx! { DocumentIcon { size: 16, class: "tree-icon doc" } },
        NodeIcon::Crate =>
            rsx! { CrateIcon { size: 16, class: "tree-icon crate" } },
        NodeIcon::Module =>
            rsx! { ModuleIcon { size: 16, class: "tree-icon module" } },
        NodeIcon::SourceFile => {
            rsx! { SourceFileIcon { size: 16, class: "tree-icon source-file" } }
        },
    }
}

fn render_folder_icon(is_expanded: bool) -> Element {
    if is_expanded {
        rsx! { FolderOpenIcon { size: 16, class: "tree-icon folder" } }
    } else {
        rsx! { FolderIcon { size: 16, class: "tree-icon folder" } }
    }
}

fn render_badge(node: &TreeNode) -> Element {
    let Some(badge) = node.badge.as_ref() else {
        return rsx! {};
    };

    let style = node
        .badge_color
        .as_deref()
        .map(|color| format!("color: {color}"))
        .unwrap_or_default();

    rsx! {
        span {
            class: "tree-badge",
            style: "{style}",
            "{badge}"
        }
    }
}

fn render_tooltip(node: &TreeNode) -> Element {
    let Some(render) = node.tooltip_render.as_ref() else {
        return rsx! {};
    };

    rsx! {
        div {
            class: "tree-tooltip",
            role: "tooltip",
            {render()}
        }
    }
}

fn render_children(
    expanded: bool,
    children: &[TreeNode],
    depth: usize,
    expanded_ids: Signal<Vec<String>>,
    selected_id: Option<String>,
    on_select: EventHandler<String>,
    show_checkboxes: bool,
    multi_selected: Signal<BTreeSet<String>>,
    focused_id: Signal<Option<String>>,
    last_clicked: Signal<Option<String>>,
    visible_order: Memo<Vec<String>>,
    on_selection_change: EventHandler<BTreeSet<String>>,
) -> Element {
    if !expanded || children.is_empty() {
        return rsx! {};
    }

    let child_nodes = children.to_vec();
    rsx! {
        div {
            class: "tree-children",
            for child in child_nodes {
                TreeItem {
                    key: "{child.id}",
                    node: child,
                    depth: depth + 1,
                    expanded_ids,
                    selected_id: selected_id.clone(),
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
