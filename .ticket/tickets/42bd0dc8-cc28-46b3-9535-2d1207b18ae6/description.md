# Feature page — graph3d (WebGPU)

Implement the demo page that showcases the `viewer-api/components/graph3d` feature surface.

## Scope

Hard-coded 12-node / 18-edge sample graph from /api/demo/graph; reuse Graph3D component; right-drag suppression check.

## Frontend

- Module: `frontend/dioxus/src/pages/graph3d.rs`.
- Add a route `/graph3d` to the Dioxus router and a sidebar nav entry.
- Page must gracefully handle missing WebGPU (red badge / SVG fallback).
- Page header includes a "Spec" link to spec-viewer for `viewer-api/components/graph3d`.

## Acceptance criteria

- See the "Demo behavior" and "Acceptance behavior" sections of the
  `viewer-api/components/graph3d` spec; every bullet must be exercised.
- No console errors during normal use (filter the unrelated 404s as in
  `tools/viewer/e2e/tests/dioxus/graph3d-right-drag.spec.ts`).

## E2E test

- File: `tools/viewer/e2e/tests/demo-viewer/graph3d.spec.ts`.
- Use the `withWebGpu(test)` helper from `_helpers.ts` (Vulkan /
  swiftshader / `headless: false`).
- After the test passes, register it as a `code_ref` on the spec:
  `spec refs add 4f14356f-c4bd-4554-be1e-35361de241da --path tools/viewer/e2e/tests/demo-viewer/graph3d.spec.ts --kind test`.

## Validation

- E2E green (requires WebGPU-enabled Chromium profile).
- Manual smoke per acceptance bullets.
