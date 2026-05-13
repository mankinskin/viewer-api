# T6: Architecture — Extract viewer-api-leptos Crate

## Problem

The log-viewer Leptos frontend currently lives as a monolith in `tools/viewer/log-viewer/frontend-leptos/`. Shared UI primitives (ResizeHandle, TreeView, TabBar, CodeViewer, ThemeSettings, WgpuOverlay, Layout) are embedded directly in this crate. The TS ecosystem solved this with `@context-engine/viewer-api-frontend` — a shared package exported from `tools/viewer/viewer-api/frontend/src/index.ts`. The Leptos ecosystem needs an equivalent crate at `tools/viewer/viewer-api/frontend-leptos/` so that doc-viewer (T7) and ticket-viewer (T8) can reuse components without duplication.

## Reference: TS viewer-api-frontend

### index.ts exports (tools/viewer/viewer-api/frontend/src/index.ts)
- **L1–5**: Re-exports preact + @preact/signals (single instance)
- **L7–20**: Common components: TreeView, Spinner, TabBar, Icons, Header, Sidebar, ResizeHandle, Layout, Panel, CodeViewer, FileContentViewer, FileTree
- **L22–27**: Session + URL state management
- **L29–30**: Theme store + GPU overlay + Scene3D
- **L32–35**: Graph3DView + HypergraphViewCore types
- **L37–38**: ThemeSettings component + store types

### Current Leptos module structure (log-viewer/frontend-leptos/src/)
- lib.rs: modules — actions, api, app, components, gpu, store, theme, types
- components/: header.rs, hypergraph_view.rs, log_viewer.rs, mod.rs, sidebar.rs, tab_bar.rs, theme_selector.rs
- gpu/: overlay.rs, shaders/

## Design

### Step 1: Create the crate

```
tools/viewer/viewer-api/frontend-leptos/
├── Cargo.toml
├── src/
│   ├── lib.rs            # Public re-exports
│   ├── components/
│   │   ├── mod.rs
│   │   ├── resize_handle.rs   # From T4
│   │   ├── tree_view.rs       # From T4
│   │   ├── tab_bar.rs         # Extracted from log-viewer
│   │   ├── header.rs          # Generic header with slot props
│   │   ├── sidebar.rs         # Generic sidebar wrapper
│   │   ├── layout.rs          # Tri-pane layout shell
│   │   ├── icons.rs           # SVG icon components (From T4)
│   │   ├── spinner.rs         # Loading spinner
│   │   ├── code_viewer.rs     # From T5
│   │   ├── code_snippet.rs    # From T5
│   │   └── file_content_viewer.rs  # From T5
│   ├── gpu/
│   │   ├── mod.rs
│   │   ├── overlay.rs         # WgpuOverlay (extracted from log-viewer)
│   │   └── shaders/           # WGSL shader files
│   ├── theme/
│   │   ├── mod.rs
│   │   ├── types.rs           # ThemeColors, EffectSettings, PaletteData, ThemePreset
│   │   ├── presets.rs         # All 17 presets
│   │   ├── css_inject.rs      # CSS variable injection
│   │   ├── store.rs           # create_theme_store() factory
│   │   └── settings.rs        # ThemeSettings overlay UI
│   ├── state/
│   │   ├── mod.rs
│   │   ├── session.rs         # /api/session GET/POST helpers
│   │   └── url.rs             # URL hash state sync
│   └── types.rs               # Shared types (TreeNode, Tab, etc.)
```

### Step 2: Cargo.toml

```toml
[package]
name = "viewer-api-leptos"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[features]
default = ["gpu"]
gpu = ["web-sys/GpuDevice", "web-sys/GpuAdapter", "web-sys/GpuCanvasContext",
       "web-sys/GpuRenderPipeline", "web-sys/GpuBuffer", "web-sys/GpuTexture",
       "web-sys/GpuCommandEncoder", "web-sys/GpuRenderPassEncoder",
       "web-sys/GpuShaderModule", "web-sys/GpuBindGroup"]
syntect = ["dep:syntect"]

[dependencies]
leptos = { version = "0.8", features = ["csr"] }
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = [
    "Window", "Document", "HtmlElement", "HtmlCanvasElement",
    "HtmlInputElement", "CssStyleDeclaration", "DomRect",
    "Element", "MouseEvent", "KeyboardEvent", "VisualViewport",
    "ScrollIntoViewOptions", "ScrollBehavior", "ScrollLogicalPosition",
    "Navigator"
]}
js-sys = "0.3"
send_wrapper = "0.6"
gloo-net = "0.7"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
console_log = "1"
log = "0.4"

# Optional syntax highlighting
syntect = { version = "5", default-features = false, features = ["default-syntaxes", "default-themes", "html"], optional = true }
```

### Step 3: create_theme_store() factory

Mirror the TS `createThemeStore(storageKey, defaults, enableGpuOverrides)` pattern:

```rust
// src/theme/store.rs

pub struct ThemeStoreConfig {
    pub storage_key: &'static str,
    pub default_preset: ThemePreset,
    pub enable_gpu_overrides: bool,
}

pub struct ThemeStore {
    pub colors: RwSignal<ThemeColors>,
    pub effects: RwSignal<EffectSettings>,
    pub palette: RwSignal<PaletteData>,
    pub active_preset: RwSignal<String>,
    pub saved_themes: RwSignal<Vec<SavedTheme>>,
    config: ThemeStoreConfig,
}

impl ThemeStore {
    pub fn new(config: ThemeStoreConfig) -> Self { ... }
    pub fn apply_preset(&self, name: &str) { ... }
    pub fn update_color<F: FnOnce(&mut ThemeColors)>(&self, f: F) { ... }
    pub fn inject_css_variables(&self) { ... }
    // Save/load/export/import (T9 fills these in)
}
```

### Step 4: Generic components with slot props

Components must be viewer-agnostic. Use Leptos `children` and callbacks for customization:

```rust
// src/components/layout.rs
#[component]
pub fn TriPaneLayout(
    #[prop(into)] header: ViewFn,
    #[prop(into)] sidebar: ViewFn,
    #[prop(into)] main_content: ViewFn,
    #[prop(optional, into)] right_panel: Option<ViewFn>,
    sidebar_width: RwSignal<f64>,
    #[prop(optional)] right_panel_width: Option<RwSignal<f64>>,
) -> impl IntoView { ... }
```

```rust
// src/components/header.rs
#[component]
pub fn Header(
    #[prop(into)] title: String,
    #[prop(optional)] children: Option<Children>,  // Right-side slot for buttons
) -> impl IntoView { ... }
```

### Step 5: Extraction from log-viewer

Refactor log-viewer to depend on viewer-api-leptos:

```toml
# tools/viewer/log-viewer/frontend-leptos/Cargo.toml
[dependencies]
viewer-api-leptos = { path = "../../viewer-api/frontend-leptos", features = ["gpu", "syntect"] }
```

Replace local component definitions with re-exports:
```rust
// log-viewer src/components/mod.rs
pub use viewer_api_leptos::components::{
    ResizeHandle, TreeView, TabBar, Header, CodeViewer, FileContentViewer, Icons,
};
// Keep log-viewer-specific components local:
pub mod log_viewer;      // LogViewer table
pub mod sidebar;         // Log-specific file tree
pub mod hypergraph_view; // 3D node view (uses shared WgpuOverlay)
```

### Step 6: Feature flag gating

```rust
// src/lib.rs
pub mod components;
pub mod theme;
pub mod state;
pub mod types;

#[cfg(feature = "gpu")]
pub mod gpu;

#[cfg(feature = "syntect")]
pub mod syntax;  // highlight_code() etc.
```

Consumers enable only what they need:
- **log-viewer**: `features = ["gpu", "syntect"]` (full)
- **doc-viewer** (T7): `features = ["syntect"]` (code highlighting, no GPU)
- **ticket-viewer** (T8): `features = ["gpu"]` (graph rendering, no syntect)

## Files to Create

| File | Purpose |
|------|---------|
| `tools/viewer/viewer-api/frontend-leptos/Cargo.toml` | Crate manifest with feature flags |
| `tools/viewer/viewer-api/frontend-leptos/src/lib.rs` | Public module declarations + re-exports |
| `tools/viewer/viewer-api/frontend-leptos/src/components/*.rs` | All shared components |
| `tools/viewer/viewer-api/frontend-leptos/src/gpu/*.rs` | WgpuOverlay + shaders (feature-gated) |
| `tools/viewer/viewer-api/frontend-leptos/src/theme/*.rs` | Theme store factory, types, presets, CSS injection, settings UI |
| `tools/viewer/viewer-api/frontend-leptos/src/state/*.rs` | Session/URL helpers |
| `tools/viewer/viewer-api/frontend-leptos/src/types.rs` | Shared types |

## Files to Modify

| File | Change |
|------|--------|
| `tools/viewer/log-viewer/frontend-leptos/Cargo.toml` | Add `viewer-api-leptos` dependency |
| `tools/viewer/log-viewer/frontend-leptos/src/components/mod.rs` | Replace local components with re-exports |
| `tools/viewer/log-viewer/frontend-leptos/src/gpu/` | Move overlay.rs + shaders to shared crate |
| `tools/viewer/log-viewer/frontend-leptos/src/theme.rs` | Delegate to shared theme module |
| `tools/viewer/log-viewer/frontend-leptos/src/store.rs` | Use ThemeStore from shared crate |
| Workspace `Cargo.toml` | Add `viewer-api-leptos` to workspace members |

## Acceptance Criteria

1. New crate `viewer-api-leptos` compiles to WASM (trunk-compatible, `cdylib` + `rlib`)
2. All shared components extracted: ResizeHandle, TreeView, TabBar, Header, Sidebar shell, Layout, CodeViewer, FileContentViewer, Icons, Spinner
3. `gpu` feature gates WgpuOverlay and all web-sys WebGPU features
4. `syntect` feature gates syntax highlighting
5. `create_theme_store()` factory works with custom storage keys and default presets
6. Theme CSS variable injection works from shared crate
7. log-viewer-leptos compiles and renders identically after extraction
8. No circular dependencies between crates
9. doc-viewer (T7) and ticket-viewer (T8) can depend on this crate with selective features
10. Component APIs use slot props / callbacks — no log-viewer-specific assumptions baked in
