# Feature page — source

Implement the demo page that showcases the `viewer-api/source` feature surface.

## Backend

GET /api/demo/source?path=&start=&end=, FileContentViewer integration, traversal/404/forbidden mappings.

## Frontend

- Module: `frontend/dioxus/src/pages/source.rs`.
- Add a route `/source` to the Dioxus router and a sidebar nav entry.
- Page header includes a "Spec" link to spec-viewer for `viewer-api/source`.
- All UI built from existing viewer-api components (`Layout`, `Panel`,
  `Spinner`, `CodeViewer`, etc.) — no novel components.

## Acceptance criteria

- All endpoints listed above are reachable and return the documented
  shape (see `viewer-api/source` spec body for exact behavior).
- The page renders without console errors.
- Manual: open `http://localhost:3099/source` and exercise every control.

## E2E test

- File: `tools/viewer/e2e/tests/demo-viewer/source.spec.ts`.
- Cover **every** acceptance bullet listed in the spec body.
- After the test passes, register it as a `code_ref` on the spec:
  `spec refs add 04a264ca-5dd6-44d4-ab5a-165822d85079 --path tools/viewer/e2e/tests/demo-viewer/source.spec.ts --kind test`.

## Validation

- E2E green.
- Manual smoke per acceptance bullets.
