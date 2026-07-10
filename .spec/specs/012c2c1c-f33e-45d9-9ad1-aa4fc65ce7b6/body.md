# Summary

A minimal viewer-template that serves as the canonical bootstrap for new viewers (for example a rule-viewer or an audit-viewer). It provides the standard viewer shell so a new domain viewer starts from a working skeleton instead of copying an existing viewer.

## Behavior Story

A developer creating a new viewer starts from viewer-template and immediately has a working shell: a left entity tree explorer, a center main view with tabs, a right panel, a bottom panel, and floating panels layered over the main view. They swap in their domain's entity source and component content without rebuilding layout, panel, or shell plumbing.

## Provided Surface Contracts

- Left **entity tree explorer** backed by a generic entity/tree source (reuses viewer-api TreeView, a20a0395).
- Center **main view with a tab bar** (reuses viewer-api TabBar, 348e17f7) hosting one or more tabbed content surfaces.
- A **right panel** and a **bottom panel** as first-class dockable regions of the shell (reuses viewer-api layout components, b3362691).
- **Floating panels** that layer over the main view without displacing its content.
- A documented, minimal wiring path: entity source -> tree -> selection -> main/tab content, so a new viewer only supplies domain data + content.

## Required Validation

- Executable: the template crate builds and runs standalone and renders all five regions (tree, tabbed main, right panel, bottom panel, floating panel) with placeholder data.
- Natural-language: documentation positions viewer-template as the starting point for new viewers and enumerates the swap points.
- Code/API: references the shared viewer-api components it composes rather than re-implementing them.

## Related Implementation Tickets

- memory-viewers ticket: "[viewer-api] Create minimal viewer-template bootstrap (tree explorer, tabbed main, right/bottom panels, floating panels)".
- Consumed by the demo-viewer (spec viewer-api/demo-viewer, 4c3b62b4) which exercises all components across domains.

## Background Knowledge References

- viewer-api layout components (b3362691), TreeView (a20a0395), TabBar (348e17f7), demo-viewer (4c3b62b4).
- Shared-shell convergence work: reusable explorer shell (763f8c13), page header shell (bb1c32f5), converge shared Dioxus viewer shells (d1e4ab96), extract viewer-theme/viewer-widgets (92964ada).
