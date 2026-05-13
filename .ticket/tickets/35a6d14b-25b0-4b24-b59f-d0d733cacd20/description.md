# Epic: Dioxus Viewer Platform

Port the viewer-api frontend library and ticket-viewer SPA from TypeScript/Preact to Rust/Dioxus 0.7, compiled to WASM via `trunk` (Trunk WASM bundler). Adds full ticket mutation capabilities powered by new ticket-http write endpoints.

## Motivation

The current TypeScript frontends (viewer-api/frontend + ticket-viewer/frontend) use Preact with @preact/signals. The Dioxus port unifies the stack in Rust, enabling:

- Shared types with ticket-api and context-api (no generated TS types needed)
- Direct use of ticket-api validation logic in the frontend
- Consistent toolchain with context-editor (already Dioxus 0.7)
- WebGPU integration via the same web-sys/wasm-bindgen bridge used by context-editor
- Future: embedded ticket-api in WASM for fully client-side operation

## Architecture Vision

The viewer is designed to work locally, with a future path to distributing server-side logic into the WASM package for fully client-side ticket storage. The architecture maintains close distance to the native Rust implementation: both a native Rust executable and a minimal WASM view layer, talking through the same native memory.

**Short-term:** HTTP client → ticket-http backend
**Long-term:** `TicketBackend` trait abstraction that both HTTP-client and embedded-store implement, enabling the WASM build to be a standalone app without a server.

### GPU Rendering Architecture

All content should ideally render through a GPU-accelerated canvas. The approach: draw native HTML/CSS flexbox layout to GPU buffers, render a full-screen WebGPU canvas background behind interactive HTML elements. This "glass panel compositing" pattern is most completely implemented in log-viewer.

```
tools/viewer/viewer-api-dioxus/     # Shared component library (Rust/Dioxus)
  src/
    components/     # Layout, TreeView, ResizeHandle, CodeViewer, TabBar, etc.
    effects/        # WgpuOverlay, particle system, CRT shader
    theme/          # ThemeStore, CSS variables, presets, save/load
    utils/          # URL state, session, math3D, palette buffer
  Cargo.toml

tools/viewer/ticket-viewer/dioxus-frontend/   # ticket-viewer SPA
  src/
    app.rs          # Root tri-pane layout
    components/     # WorkspacePicker, TicketTree, TicketContent, DependencyGraph
    store/          # Reactive state management with Dioxus signals
    api/            # TicketBackend trait + HTTP client impl
  Dioxus.toml
  index.html
  Cargo.toml
```

## Key Technical Decisions (From Interview)

| Decision | Answer | Impact |
|----------|--------|--------|
| CSS strategy | **Keep CSS custom properties — 1:1 port** | Port all 9 CSS files, preserve `--theme-*` system |
| Markdown rendering | **Pure Rust stack** (`pulldown-cmark` + `syntect`) | No JS deps; meeting if no Rust alternative exists |
| Auth model | **Reuse viewer-api auth backend** | Multi-user ready; auth middleware on all write endpoints |
| Description editor | **`<textarea>` + same rendering pipeline** | Reuse `pulldown-cmark` for preview; no WYSIWYG |
| GPU features | **High priority — core architecture** | Full-screen WebGPU canvas compositing; study log-viewer pipeline |
| State persistence | **localStorage + `TicketBackend` trait** | Future: embedded ticket-api in WASM for offline operation |
| Creation form | **Dynamic fields from schema endpoint** | Schema endpoint is hard prerequisite for creation form |
| Batch operations | **Yes: multi-select, queue, bulk apply** | New tickets created: `b21604c1` (UI) + `8034efd8` (API) |
| Mobile responsive | **Required — improve over existing** | Collapsible sidebar, stacked panels, touch-friendly |
| Build tooling | **`trunk serve`** | Hot-reload; `Trunk.toml` + `index.html` config; consistent with log-viewer-leptos |
| Update strategy | **Optimistic with gate** | Apply immediately, block further mutations until server confirms |
| TypeScript fate | **Freeze → archive → remove** | No new TS features; remove after Dioxus is proven |

## Track Breakdown

### Track 1: viewer-api-dioxus Foundation (9 tickets)
Port shared UI primitives from viewer-api/frontend to Dioxus components.

| Ticket | Component | Priority |
|--------|-----------|----------|
| `7346feae` | Crate scaffold + `trunk serve` build | Critical |
| `b3f9878d` | Layout: Header, Sidebar, Panel, GlassPanel | Critical |
| `9dec4f23` | ResizeHandle with rAF drag | High |
| `31739fc3` | TreeView + FileTree with sort/filter | High |
| `11f77899` | TabBar, Spinner, Icons | Medium |
| `7330aa36` | CodeViewer + FileContentViewer | High |
| `46864375` | Theme system: store, CSS vars, presets | High |
| `503eecc9` | URL state + session utilities | Medium |
| `2405a83e` | CSS stylesheets port | Medium |

### Track 2: ticket-viewer Dioxus Frontend (7 tickets)
Port the ticket-viewer SPA using viewer-api-dioxus components.

| Ticket | Component | Priority |
|--------|-----------|----------|
| `44d22e8f` | Crate scaffold with `trunk serve` | Critical |
| `80b4b77f` | WorkspacePicker + auth token | High |
| `3e79be12` | TicketTree: state groups, search, filter, sort | Critical |
| `af19b0f6` | TicketContent: Markdown + TOML tabs | Critical |
| `5711c397` | SVG dependency graph fallback | High |
| `8672684c` | SSE real-time updates | High |
| `c2f04936` | State persistence: localStorage + URL routing | Medium |

### Track 3: ticket-http Write API (4 tickets)
Add mutation endpoints to the currently read-only ticket-http server.

| Ticket | Component | Priority |
|--------|-----------|----------|
| `69abd1c7` | CRUD: create, update, close, cancel, delete | Critical |
| `15871ee6` | Edge mutation: add, remove | High |
| `3fda11c3` | History + revert endpoints | Medium |
| `189a6068` | Schema endpoint: types, states, transitions | High |

### Track 4: New Ticket Features (9 tickets)
Add interactive ticket management features not in the current read-only viewer.

| Ticket | Component | Priority |
|--------|-----------|----------|
| `3e069173` | Ticket creation form (dynamic from schema) | Critical |
| `15ee34c6` | Inline field editing | High |
| `4143b314` | State transition UI with state machine | Critical |
| `19383fed` | Edge management from graph view | Medium |
| `9d0c7931` | Description editor (textarea + preview) | High |
| `dd80a182` | History timeline with diffs | Medium |
| `4a228c24` | Full-text search UI | Medium |
| `b21604c1` | Batch operations: multi-select, queue, bulk apply | High |
| `8034efd8` | Batch API endpoint (transactional) | High |

### Track 5: Advanced / GPU Features (3 tickets) — HIGH PRIORITY
GPU rendering is core architecture, not optional polish.

| Ticket | Component | Priority |
|--------|-----------|----------|
| `12d3c38b` | GPU 3D dependency graph (WebGPU) | High |
| `dbd048a0` | WgpuOverlay: full-screen canvas compositing | High |
| `512986e0` | Theme settings UI with live preview | High |

## Dependency Graph Summary

- **Track 1** is the foundation; all other tracks depend on it
- **Track 2** depends on Track 1 components + scaffold
- **Track 3** is independent of frontend tracks (pure backend)
- **Track 4** depends on both Track 2 (UI) and Track 3 (API)
- **Track 5** depends on Track 1 (theme) and Track 2 (graph)
- **GPU compositing** (Track 5) should inform scaffold design from the start

## Supersedes

- `7f41940d` — Epic: Leptos Viewer Platform (cancelled)
- `d83f8e52` — Port: ticket-viewer Leptos frontend (cancelled)
- `ca0f6ccc` — Arch: Extract viewer-api-leptos (cancelled)

## Done Condition

All 5 tracks complete. The Dioxus ticket-viewer serves as the default frontend with full read/write capabilities, GPU-accelerated rendering, batch operations, mobile responsive layout, and feature parity with the TypeScript version. TypeScript frontend archived.
