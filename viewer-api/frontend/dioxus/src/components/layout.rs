//! Layout components — Header, Layout, Sidebar (collapsible + resizable +
//! mobile-drawer), Panel (left/right/top/bottom placement), and GlassPanel.
//!
//! CSS class names match the TypeScript viewer-api package so that the shared
//! `viewer-api.css` stylesheet applies without modification.

mod header;
mod panel;
mod sidebar;

pub use self::{
    header::{
        Header,
        Layout,
    },
    panel::{
        GlassPanel,
        Panel,
        PanelPlacement,
    },
    sidebar::{
        is_mobile_sidebar_viewport,
        Sidebar,
        SIDEBAR_MOBILE_BREAKPOINT_PX,
    },
};
