Move the WgpuOverlay component, WGSL shaders, effects, and 3D math utilities from `log-viewer/frontend/src/` to `viewer-api/frontend/src/` so they become shared infrastructure. Also moves the SVG GraphView from ticket-viewer to viewer-api as a generic `Graph2DView`.

This ticket also introduces two architectural upgrades:
1. **App schema system** — replaces the hardcoded `ELEMENT_SELECTORS` array with a typed `AppSchema` (selectors + kinds + optional particle ranges). viewer-api ships `MINIMAL_SCHEMA`. Log-viewer and ticket-viewer each define derived schemas passed via `WgpuOverlay`'s `schema` prop.
2. **Isolated rendering contexts** — replaces module-level globals in `overlay-api.ts` (`gpuOverlayEnabled`, `fxEnabled`, renderer callbacks) with `createOverlayContext()` factory + `<OverlayProvider>` Preact context + `useOverlayContext()` hook. Each app gets its own GPU context and signal state with no shared mutable globals.

## New Files in viewer-api

- `components/WgpuOverlay/schemas.ts` — `AppSchema`, `MINIMAL_SCHEMA`, kind constants
- `components/WgpuOverlay/overlay-context.ts` — `createOverlayContext()`, `OverlayProvider`, `useOverlayContext()`
- `components/Graph2DView/Graph2DView.tsx` — generic SVG graph (moved from ticket-viewer GraphView.tsx)

## Files to Move

**WgpuOverlay (8 files):**
- `components/WgpuOverlay/WgpuOverlay.tsx` — Preact component (canvas behind DOM, z-index -1)
- `components/WgpuOverlay/gpu-init.ts` — WebGPU device + pipeline creation
- `components/WgpuOverlay/gpu-buffers.ts` — Uniform/element/particle/palette GPU buffers
- `components/WgpuOverlay/gpu-render-loop.ts` — 3-pass render loop (compute, background, overlays+particles)
- `components/WgpuOverlay/element-scanner.ts` — DOM element tracking via MutationObserver/IntersectionObserver
- `components/WgpuOverlay/element-types.ts` — CSS selectors, element kind constants, buffer layouts
- `components/WgpuOverlay/overlay-api.ts` — Signals (gpuOverlayEnabled, fxEnabled) + renderer callbacks
- `components/WgpuOverlay/thumbnail-capture.ts` — JPEG frame capture for theme previews

**Shaders + effects (3+ files):**
- `effects/palette.ts` — `buildPaletteBuffer()` → Float32Array for GPU
- `effects/palette.wgsl` — ThemePalette struct (24 vec4 slots)
- `effects/particle-shading.wgsl` — Spark/ember/beam/glitter RGBA functions
- All WGSL files concatenated by `gpu-init.ts` (background, particles, compute, types, noise)

**3D math:**
- `components/Scene3D/math3d.ts` → `utils/math3d.ts`

## Post-Move

Update all log-viewer imports to reference `@context-engine/viewer-api-frontend` instead of local paths. Replace all imports of `overlay-api` globals with `useOverlayContext()`. Both log-viewer and ticket-viewer `App.tsx` wrap with `<OverlayProvider>`. Log-viewer must still build and function identically.

## Risk

High — many files, shader import paths change, Vite build config may need `?raw` adjustments for WGSL files. `OverlayContext` migration touches ~10 files in log-viewer.
