# Feature page — session

Implement the demo page that showcases the `viewer-api/session` feature surface.

## Backend

GET /api/demo/session, POST /api/demo/session/kv, viewer-session cookie round-trip + new-session flow.

## Frontend

- Module: `frontend/dioxus/src/pages/session.rs`.
- Add a route `/session` to the Dioxus router and a sidebar nav entry.
- Page header includes a "Spec" link to spec-viewer for `viewer-api/session`.
- All UI built from existing viewer-api components (`Layout`, `Panel`,
  `Spinner`, `CodeViewer`, etc.) — no novel components.

## Acceptance criteria

- All endpoints listed above are reachable and return the documented
  shape (see `viewer-api/session` spec body for exact behavior).
- The page renders without console errors.
- Manual: open `http://localhost:3099/session` and exercise every control.

## E2E test

- File: `tools/viewer/e2e/tests/demo-viewer/session.spec.ts`.
- Cover **every** acceptance bullet listed in the spec body.
- After the test passes, register it as a `code_ref` on the spec:
  `spec refs add 51c69e48-c8a1-4d45-b050-e06671fe7d71 --path tools/viewer/e2e/tests/demo-viewer/session.spec.ts --kind test`.

## Validation

- E2E green.
- Manual smoke per acceptance bullets.
