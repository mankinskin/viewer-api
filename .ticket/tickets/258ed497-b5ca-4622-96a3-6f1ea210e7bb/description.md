# Feature page — sse

Implement the demo page that showcases the `viewer-api/sse` feature surface.

## Backend

GET /api/demo/sse/stream emitting one event per second + GET /api/demo/sse/stats for active stream count.

## Frontend

- Module: `frontend/dioxus/src/pages/sse.rs`.
- Add a route `/sse` to the Dioxus router and a sidebar nav entry.
- Page header includes a "Spec" link to spec-viewer for `viewer-api/sse`.
- All UI built from existing viewer-api components (`Layout`, `Panel`,
  `Spinner`, `CodeViewer`, etc.) — no novel components.

## Acceptance criteria

- All endpoints listed above are reachable and return the documented
  shape (see `viewer-api/sse` spec body for exact behavior).
- The page renders without console errors.
- Manual: open `http://localhost:3099/sse` and exercise every control.

## E2E test

- File: `tools/viewer/e2e/tests/demo-viewer/sse.spec.ts`.
- Cover **every** acceptance bullet listed in the spec body.
- After the test passes, register it as a `code_ref` on the spec:
  `spec refs add 54800731-e07f-4fb2-8802-fd7d2acc8c05 --path tools/viewer/e2e/tests/demo-viewer/sse.spec.ts --kind test`.

## Validation

- E2E green.
- Manual smoke per acceptance bullets.
