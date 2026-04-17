/// Shared Leptos UI components for viewer tools.
///
/// Provides reusable components:
/// - [`TreeView`] — hierarchical tree with expand/collapse
/// - [`ResizeHandle`] — drag-to-resize handle with rAF batching
/// - [`SidebarShell`] — sidebar shell with header, collapse, and resize

pub mod components;

pub use components::resize_handle::ResizeHandle;
pub use components::sidebar_shell::SidebarShell;
pub use components::tree_view::{NodeIcon, TreeNode, TreeView};

#[cfg(test)]
mod tests;
