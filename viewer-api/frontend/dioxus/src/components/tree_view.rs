//! TreeView and FileTree components.
//!
//! [`TreeView`] renders a recursive, keyboard-accessible tree of [`TreeNode`]s.
//! [`FileTree`] wraps it with sort and filter controls.

mod file_tree;
mod item;
mod types;
mod view;

pub use self::file_tree::FileTree;
pub use self::types::{FilterDef, NodeIcon, SortKey, TreeNode};
pub use self::view::TreeView;
