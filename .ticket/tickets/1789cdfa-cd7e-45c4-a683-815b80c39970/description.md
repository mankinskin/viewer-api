Extract the entire GPU rendering pipeline from log-viewer into viewer-api as shared infrastructure, then use it to build a GPU-rendered dependency graph in ticket-viewer.

## Scope

This is the parent feature ticket covering three sequential implementation sub-tickets:

1. **B1 (b3d250d5):** Move WgpuOverlay, shaders, effects, and 3D math from `log-viewer/frontend` to `viewer-api/frontend`. Update log-viewer imports to consume from the shared package.
2. **B2 (c826869a):** Extract a generic `Graph3DView` component to viewer-api — encapsulating layout engine, camera, mouse interaction, node animation, and DOM positioning without any log-viewer-specific concepts.
3. **B3 (6f71ca0b):** Build a GPU dependency graph in ticket-viewer using `Graph3DView`, replacing the current SVG `GraphView.tsx`.

## Architecture

```
viewer-api/frontend/
├── components/WgpuOverlay/   ← GPU init, buffers, render loop, element scanner
├── components/Graph3DView/   ← Generic 3D graph (layout, camera, interaction)
├── effects/                  ← Palette, WGSL shaders (particles, background)
└── utils/math3d.ts           ← Vec3/Mat4 math library

log-viewer/frontend/
└── components/HypergraphView/  ← Stays (search paths, nesting, decomposition)
    └── imports WgpuOverlay + Graph3DView from viewer-api

ticket-viewer/frontend/
└── components/DependencyGraph/ ← New, uses Graph3DView
```

## Risk

High — touches 20+ files across 3 packages, involves WebGPU shader code, and requires the log-viewer to keep working throughout extraction.
