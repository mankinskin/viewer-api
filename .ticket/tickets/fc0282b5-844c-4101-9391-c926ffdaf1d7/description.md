# Feature page — layout

Implement the demo page that showcases the `viewer-api/components/layout` feature surface.

## Scope

Layout / Header / Sidebar / Panel / GlassPanel / ResizeHandle showcase.

## Frontend

- Module: `frontend/dioxus/src/pages/layout.rs`.
- Add a route `/layout` to the Dioxus router and a sidebar nav entry.
- Reuse existing `viewer-api` components — no novel ones.
- Page header includes a "Spec" link to spec-viewer for `viewer-api/components/layout`.

## Acceptance criteria

- See the "Demo behavior" and "Acceptance behavior" sections of the
  `viewer-api/components/layout` spec; every bullet must be exercised.
- No console errors during normal use.

## E2E test

- File: `tools/viewer/e2e/tests/demo-viewer/layout.spec.ts`.
- After the test passes, register it as a `code_ref` on the spec:
  `spec refs add b3362691-09a0-4028-8daa-13b4c4102c15 --path tools/viewer/e2e/tests/demo-viewer/layout.spec.ts --kind test`.

## Validation

- E2E green.
- Manual smoke per acceptance bullets.
