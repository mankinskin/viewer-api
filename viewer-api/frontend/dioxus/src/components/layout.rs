//! Layout components — Header, Layout, Sidebar (collapsible + resizable +
//! mobile-drawer), Panel (left/right/top/bottom placement), and GlassPanel.
//!
//! CSS class names match the TypeScript viewer-api package so that the shared
//! `viewer-api.css` stylesheet applies without modification.

mod header;
mod panel;
mod sidebar;

pub use self::header::{Header, Layout};
pub use self::panel::{GlassPanel, Panel, PanelPlacement};
pub use self::sidebar::{Sidebar, SIDEBAR_MOBILE_BREAKPOINT_PX, is_mobile_sidebar_viewport};
