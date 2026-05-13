# Feature page — pagination query

Implement the demo page that showcases the `viewer-api/pagination-query` feature surface.

## Backend

GET /api/demo/items, GET /api/demo/query, 50 seed items, cursor round-trip, clamp behavior.

## Frontend

- Module: `frontend/dioxus/src/pages/pagination_query.rs`.
- Add a route `/pagination-query` to the Dioxus router and a sidebar nav entry.
- Page header includes a "Spec" link to spec-viewer for `viewer-api/pagination-query`.
- All UI built from existing viewer-api components (`Layout`, `Panel`,
  `Spinner`, `CodeViewer`, etc.) — no novel components.

## Acceptance criteria

- All endpoints listed above are reachable and return the documented
  shape (see `viewer-api/pagination-query` spec body for exact behavior).
- The page renders without console errors.
- Manual: open `http://localhost:3099/pagination-query` and exercise every control.

## E2E test

- File: `tools/viewer/e2e/tests/demo-viewer/pagination-query.spec.ts`.
- Cover **every** acceptance bullet listed in the spec body.
- After the test passes, register it as a `code_ref` on the spec:
  `spec refs add c9b40e5d-1239-4ad6-99b1-0b759a9c4c49 --path tools/viewer/e2e/tests/demo-viewer/pagination-query.spec.ts --kind test`.

## Validation

- E2E green.
- Manual smoke per acceptance bullets.
