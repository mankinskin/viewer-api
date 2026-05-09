use std::collections::BTreeSet;

use dioxus::prelude::*;

use super::{
    item::TreeItem,
    types::TreeNode,
};

fn collect_visible_ids(
    nodes: &[TreeNode],
    expanded_ids: &[String],
) -> Vec<String> {
    let mut result = Vec::new();
    for node in nodes {
        result.push(node.id.clone());
        if node.is_dir && expanded_ids.contains(&node.id) {
            result.extend(collect_visible_ids(&node.children, expanded_ids));
        }
    }
    result
}

#[component]
pub fn TreeView(
    nodes: Vec<TreeNode>,
    #[props(default)] selected_id: Option<String>,
    #[props(default)] on_select: EventHandler<String>,
    #[props(default)] initially_expanded: Vec<String>,
    #[props(default)] class: String,
    #[props(default = false)] show_checkboxes: bool,
    #[props(default)] on_selection_change: EventHandler<BTreeSet<String>>,
) -> Element {
    let expanded_ids = use_signal(|| initially_expanded);
    let mut multi_selected: Signal<BTreeSet<String>> =
        use_signal(BTreeSet::new);
    let mut focused_id: Signal<Option<String>> = use_signal(|| None);
    let last_clicked: Signal<Option<String>> = use_signal(|| None);

    let nodes_for_order = nodes.clone();
    let visible_order: Memo<Vec<String>> = use_memo(move || {
        collect_visible_ids(&nodes_for_order, &expanded_ids.read())
    });

    let combined = if class.is_empty() {
        "tree-view".to_string()
    } else {
        format!("tree-view {class}")
    };

    rsx! {
        div {
            class: "{combined}",
            role: "tree",
            tabindex: if show_checkboxes { 0 } else { -1 },
            onkeydown: move |event: Event<KeyboardData>| {
                if !show_checkboxes {
                    return;
                }

                let order = visible_order.read();
                if order.is_empty() {
                    return;
                }

                let current_focus = focused_id.read().clone();
                match event.key() {
                    Key::ArrowDown => {
                        event.prevent_default();
                        let next = match &current_focus {
                            Some(id) => order
                                .iter()
                                .position(|candidate| candidate == id)
                                .and_then(|position| order.get(position + 1))
                                .cloned(),
                            None => order.first().cloned(),
                        };
                        if let Some(id) = next {
                            focused_id.set(Some(id));
                        }
                    }
                    Key::ArrowUp => {
                        event.prevent_default();
                        let previous = match &current_focus {
                            Some(id) => order
                                .iter()
                                .position(|candidate| candidate == id)
                                .filter(|&position| position > 0)
                                .and_then(|position| order.get(position - 1))
                                .cloned(),
                            None => order.last().cloned(),
                        };
                        if let Some(id) = previous {
                            focused_id.set(Some(id));
                        }
                    }
                    Key::Character(ref key) if key == " " => {
                        event.prevent_default();
                        let focused = current_focus.clone();
                        drop(order);
                        if let Some(id) = focused {
                            {
                                let mut selection = multi_selected.write();
                                if selection.contains(&id) {
                                    selection.remove(&id);
                                } else {
                                    selection.insert(id);
                                }
                            }
                            on_selection_change.call(multi_selected.read().clone());
                        }
                    }
                    Key::Character(ref key) if key == "a" => {
                        if event.modifiers().contains(Modifiers::CONTROL) {
                            event.prevent_default();
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
