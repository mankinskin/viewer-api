# viewer-api-dioxus

Dioxus 0.7 viewer platform for the context-engine toolchain.

## Architecture

This crate is the foundation of the Dioxus Viewer Platform epic. It provides:

- **Root application component** — mounts into the browser DOM via `dioxus::launch`.
- **Full-screen WebGPU canvas** (`#webgpu-canvas`) — required from startup for the `WgpuOverlay` GPU compositing layer. Downstream component ports depend on this element being stable in the DOM.
- **UI overlay root** (`#ui-root`) — viewer components (TreeView, TabBar, Layout, etc.) mount here on top of the canvas.

### Crate Layering

```
viewer-api-dioxus  (this crate — scaffold + layout shell)
     │
     ├── WgpuOverlay          (GPU compositing, acquires #webgpu-canvas)
     ├── TreeView component   (Track 1 port)
     ├── TabBar component     (Track 1 port)
     └── Layout component     (Track 1 port)
```

## Usage

### Development (hot-reload)

```bash
cd tools/viewer/viewer-api/frontend/dioxus
trunk serve
```

Opens a dev server at `http://localhost:8092` with hot-reload.

### Production build

```bash
cd tools/viewer/viewer-api/frontend/dioxus
trunk build --release
```

Outputs an optimised WASM bundle to `dist/`.

### Cargo check (WASM target)

```bash
cargo check --target wasm32-unknown-unknown -p viewer-api-dioxus
```

## Requirements

- `rustup target add wasm32-unknown-unknown`
- `cargo install trunk` (provides the `trunk` command)
