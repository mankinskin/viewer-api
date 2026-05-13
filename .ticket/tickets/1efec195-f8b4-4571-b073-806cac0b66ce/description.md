# Feature page — store primitives

Implement the demo page that showcases the `viewer-api/store-primitives` feature surface.

## Scope

Persistent counter / text / json signals; localStorage live readout.

## Frontend

- Module: `frontend/dioxus/src/pages/store_primitives.rs`.
- Add a route `/store-primitives` to the Dioxus router and a sidebar nav entry.
- Reuse existing `viewer-api` components — no novel ones.
- Page header includes a "Spec" link to spec-viewer for `viewer-api/store-primitives`.

## Acceptance criteria

- See the "Demo behavior" and "Acceptance behavior" sections of the
  `viewer-api/store-primitives` spec; every bullet must be exercised.
- No console errors during normal use.

## E2E test

- File: `tools/viewer/e2e/tests/demo-viewer/store-primitives.spec.ts`.
- After the test passes, register it as a `code_ref` on the spec:
  `spec refs add baaa35ff-4eb6-4288-b4d3-257311b98aa4 --path tools/viewer/e2e/tests/demo-viewer/store-primitives.spec.ts --kind test`.

## Validation

- E2E green.
- Manual smoke per acceptance bullets.
