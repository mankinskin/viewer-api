# viewer-api: WebGPU overlay (effects subsystem)

Canonical specification for the WebGPU overlay subsystem under
`viewer-api/frontend/dioxus/src/effects/wgpu_overlay/` — the shared device,
render loop, particle pipelines, glass / smoke / CRT effects, and element
scanner that powers the GPU-accelerated visual layer of every viewer.

## Public surface

- `effects::wgpu_overlay::mount_overlay()` — installs the canvas + RAF loop.
- `bootstrap_ctx()` — creates the shared `wgpu::Device`/`Queue` and exposes
  it via `SharedDeviceContext` for downstream consumers (e.g. Graph3D).
- Settings (theme-driven): per-effect toggles, colours, sliders for
  particles (Metal Sparks, Embers/Ash, Angelic Beams, Glitter), Cinder
  Palette, Background Smoke, Glass Panels, CRT Effect.
- `element_scanner` — observes DOM elements tagged with
  `data-wgpu-glass="…"` and feeds their bounding boxes to the GPU passes.

## Demo behavior

The `pages/wgpu_overlay.rs` page renders a controlled visual playground:

1. The page sets `gpuOverlayEnabled = true` on entry.
2. Per-effect toggle rows mirror the ThemeSettings panel.
3. A "stress test" button enables every effect simultaneously and shows the
   measured FPS pulled from the render loop.
4. A canvas-capture button takes a still and renders it inline so the test
   can compare a CPU off vs effects-on screenshot.
5. A WebGPU support badge (red if `navigator.gpu` is missing).

## Acceptance behavior (validated by e2e)

- With WebGPU available, the overlay canvas is present and has a
  non-empty `requestAnimationFrame` cadence (a `__rafTicks` counter
  installed by the demo page increments by ≥30 over 1 s).
- Disabling the master GPU toggle via the demo UI freezes the canvas
  (`__rafTicks` stops increasing).
- Re-enabling resumes the loop without page reload.
- Stress-test mode does not produce console errors.
- A WebGPU-unsupported probe (browser launched without the WebGPU flags)
  shows the red badge and the page does not crash.

## Code references

- `tools/viewer/viewer-api/frontend/dioxus/src/effects/wgpu_overlay/`
- `tools/viewer/e2e/tests/demo-viewer/wgpu-overlay.spec.ts`
- Reuses WebGPU launch flags pattern from
  `tools/viewer/e2e/tests/dioxus/graph3d-right-drag.spec.ts`.
