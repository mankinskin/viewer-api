Extend the existing shared widgets with capabilities doc-viewer uses but Dioxus viewers cannot express today.

## Deliverables

1. `tree_view.rs` — rich tooltip
   - Add `tooltip_render: Option<Rc<dyn Fn() -> Element>>` to `TreeNode`
   - Renderer prefers `tooltip_render`; falls back to `tooltip: String` (existing) or no tooltip
   - CSS: `.tree-node:hover .tree-tooltip` floating overlay (no JS positioning in v1; just absolute below the node)
   - Backwards compatible: existing call sites unaffected

2. `layout.rs` — mobile sidebar audit
   - Verify `Sidebar` component supports `mobile_open: bool` + `on_mobile_close: EventHandler<()>` matching doc-viewer's SharedSidebar
   - If missing, backfill the props and the burger-overlay pattern
   - Expose burger toggle wiring via a `MenuToggle` helper if useful

3. `components/header.rs` — `HeaderActions` helper
   - New convenience component: `HeaderActions { on_home, on_refresh, on_filter_toggle, on_clear, on_theme_toggle, filter_active: bool, has_active_filters: bool }`
   - All handlers `Option<EventHandler<()>>` so viewers can opt in
   - Renders the standard button row using existing `.btn` / `.btn-active` classes from `buttons.css`
   - Re-exported from `components/mod.rs` and `lib.rs`

## Acceptance criteria

- `cargo check -p viewer-api-dioxus --target wasm32-unknown-unknown` passes
- `cargo check -p spec-viewer-dioxus --target wasm32-unknown-unknown` passes (no breakage)
- Spec-viewer's existing `TreeNode` usages compile unchanged
- A snapshot of the spec-viewer header (or a demo route) renders the `HeaderActions` button row correctly
