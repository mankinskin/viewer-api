use std::rc::Rc;

use dioxus::prelude::*;

#[derive(Clone, PartialEq, Default)]
pub enum NodeIcon {
    #[default]
    Auto,
    Folder,
    File,
    Doc,
    Crate,
    Module,
    SourceFile,
}

#[derive(Clone)]
pub struct TreeNode {
    pub id: String,
    pub label: String,
    pub badge: Option<String>,
    pub tooltip: Option<String>,
    pub tooltip_render: Option<Rc<dyn Fn() -> Element>>,
    pub badge_color: Option<String>,
    pub is_dir: bool,
    pub icon: NodeIcon,
    pub children: Vec<TreeNode>,
}

impl PartialEq for TreeNode {
    fn eq(&self, other: &Self) -> bool {
        let tooltip_equal = match (&self.tooltip_render, &other.tooltip_render) {
            (None, None) => true,
            (Some(left), Some(right)) => Rc::ptr_eq(left, right),
            _ => false,
        };

        tooltip_equal
            && self.id == other.id
            && self.label == other.label
            && self.badge == other.badge
            && self.tooltip == other.tooltip
            && self.badge_color == other.badge_color
            && self.is_dir == other.is_dir
            && self.icon == other.icon
            && self.children == other.children
    }
}

impl TreeNode {
    pub fn leaf(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            badge: None,
            tooltip: None,
            tooltip_render: None,
            badge_color: None,
            is_dir: false,
            icon: NodeIcon::Auto,
            children: vec![],
        }
    }

    pub fn dir(id: impl Into<String>, label: impl Into<String>, children: Vec<TreeNode>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            badge: None,
            tooltip: None,
            tooltip_render: None,
            badge_color: None,
            is_dir: true,
            icon: NodeIcon::Auto,
            children,
        }
    }

    pub fn with_tooltip_render<F>(mut self, render: F) -> Self
    where
        F: Fn() -> Element + 'static,
    {
        self.tooltip_render = Some(Rc::new(render));
        self
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct SortKey {
    pub key: String,
    pub label: String,
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

#[derive(Clone, PartialEq)]
pub struct FilterDef {
    pub key: String,
    pub label: String,
    pub count: usize,
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