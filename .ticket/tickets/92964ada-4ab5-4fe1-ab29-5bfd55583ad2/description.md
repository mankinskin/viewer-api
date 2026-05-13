## Problem

`viewer-api-dioxus` currently bundles three logically distinct concerns:

1. **Theme system** — `store/theme.rs`, `ThemeProvider`, `ThemeStore`, `ThemePreset`, presets (PAPER/DARK/ARCADIA/SCRATCHBOARD), CSS-variable writer, theme-settings UI, and shared `public/css/variables.css` + `theme-settings.css`.
2. **Reusable widgets** — `Header`, `Sidebar`, `Panel`, `GlassPanel`, `TabBar`, `TreeView`, `Modal`, `ResizeHandle`, `Cards`, `Spinner`, plus the matching CSS files in `public/css/`.
3. **Application shell + WebGPU effects** — `ViewerShell`, `WgpuOverlay`, `graph3d`, `effects/` modules.

Other viewers (`spec-viewer`, `ticket-viewer`, future `demo-viewer`) only need #1 and #2 most of the time, but currently must depend on the entire `viewer-api-dioxus` crate which pulls WebGPU + WGSL + `wgpu` + every shader.

## Acceptance criteria

1. New crate `tools/viewer/viewer-theme/` containing:
   - `ThemeStore`, `ThemeColors`, `ThemePreset`, `ThemeProvider` component.
   - All CSS files currently under `viewer-api/frontend/dioxus/public/css/{variables,theme-settings,glass-panel}.css` (move, not copy).
   - The `ThemeSettings` modal component.
2. New crate `tools/viewer/viewer-widgets/` containing:
   - `Header`, `Sidebar`, `Panel`, `Layout`, `Modal`, `ResizeHandle`, `TabBar`, `TreeView`, `Cards`, `Spinner`, `Breadcrumbs`, `Chip`, `IconButton`, `Icons`.
   - All CSS for these widgets (`buttons.css`, `layout.css`, `cards.css`, `tree.css`, `tabs.css`, `chip.css`, `spinner.css`, `breadcrumbs.css`, `modal.css`).
3. `viewer-api-dioxus` re-exports both crates so existing consumers `use viewer_api_dioxus::Header` continue to compile (transitional shim).
4. `ticket-viewer` and `spec-viewer` `Cargo.toml` switch to depend on `viewer-theme` + `viewer-widgets` directly (deprecate the heavy `viewer-api-dioxus` import, keep only for `WgpuOverlay`/`graph3d`).
5. Build artifacts shrink — the WebGPU shader sources should no longer compile into the dependency tree of consumers that only want widgets/theme.
6. Existing Playwright tests under `tools/viewer/e2e/` still pass.

## Implementation notes

- This is a workspace refactor; expect roughly 30–60 files moved/edited.
- Tools/viewer Cargo workspace already exists — add new crates to the root `[workspace] members` list.
- CSS file paths are referenced from each viewer's `index.html` via `data-trunk rel="css" href="..."` — these paths need updating.
- Consider an `extension` story: each viewer-extension (e.g. ticket-viewer specific cards) could become its own crate that depends on `viewer-widgets` and contributes `register_extension()` style hooks. Out of scope here — leave as a follow-up ticket.

## Parent

Epic `4e2b2b0b-9f56-4786-991c-8f10e653f4c3`.

## Status

Marked `new` per user direction — refinement only at this stage, implementation not part of the current iteration.
