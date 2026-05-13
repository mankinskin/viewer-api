# Feature page — wgpu overlay (WebGPU)

Implement the demo page that showcases the `viewer-api/effects/wgpu-overlay` feature surface.

## Scope

WebGPU overlay playground: per-effect toggles mirrored from ThemeSettings; FPS/RAF tick badge; canvas snapshot button.

## Frontend

- Module: `frontend/dioxus/src/pages/wgpu_overlay.rs`.
- Add a route `/wgpu-overlay` to the Dioxus router and a sidebar nav entry.
- Page must gracefully handle missing WebGPU (red badge / SVG fallback).
- Page header includes a "Spec" link to spec-viewer for `viewer-api/effects/wgpu-overlay`.

## Acceptance criteria

- See the "Demo behavior" and "Acceptance behavior" sections of the
  `viewer-api/effects/wgpu-overlay` spec; every bullet must be exercised.
- No console errors during normal use (filter the unrelated 404s as in
  `tools/viewer/e2e/tests/dioxus/graph3d-right-drag.spec.ts`).

## E2E test

- File: `tools/viewer/e2e/tests/demo-viewer/wgpu-overlay.spec.ts`.
- Use the `withWebGpu(test)` helper from `_helpers.ts` (Vulkan /
  swiftshader / `headless: false`).
- After the test passes, register it as a `code_ref` on the spec:
  `spec refs add f153483c-f984-4564-94ac-36234b5cbe3f --path tools/viewer/e2e/tests/demo-viewer/wgpu-overlay.spec.ts --kind test`.

## Validation

- E2E green (requires WebGPU-enabled Chromium profile).
- Manual smoke per acceptance bullets.
