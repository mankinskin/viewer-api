//! TreeView and FileTree components.
//!
//! [`TreeView`] renders a recursive, keyboard-accessible tree of [`TreeNode`]s.
//! [`FileTree`] wraps it with sort and filter controls.

mod explorer_shell;
mod file_tree;
mod filter_toggle_button;
mod item;
mod types;
mod view;

pub use self::{
    explorer_shell::{
        ExplorerShell,
        SidebarSearch,
    },
    file_tree::FileTree,
    filter_toggle_button::FilterToggleButton,
    types::{
        FilterDef,
        NodeIcon,
        SortKey,
        TreeNode,
    },
    view::TreeView,
};
