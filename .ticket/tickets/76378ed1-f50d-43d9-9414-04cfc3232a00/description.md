# Feature page — auth middleware

Implement the demo page that showcases the `viewer-api/auth-middleware` feature surface.

## Backend

GET /api/demo/secured, GET /api/demo/error/:kind, full ApiError gallery, x-request-id assertion.

## Frontend

- Module: `frontend/dioxus/src/pages/auth_middleware.rs`.
- Add a route `/auth-middleware` to the Dioxus router and a sidebar nav entry.
- Page header includes a "Spec" link to spec-viewer for `viewer-api/auth-middleware`.
- All UI built from existing viewer-api components (`Layout`, `Panel`,
  `Spinner`, `CodeViewer`, etc.) — no novel components.

## Acceptance criteria

- All endpoints listed above are reachable and return the documented
  shape (see `viewer-api/auth-middleware` spec body for exact behavior).
- The page renders without console errors.
- Manual: open `http://localhost:3099/auth-middleware` and exercise every control.

## E2E test

- File: `tools/viewer/e2e/tests/demo-viewer/auth-middleware.spec.ts`.
- Cover **every** acceptance bullet listed in the spec body.
- After the test passes, register it as a `code_ref` on the spec:
  `spec refs add 52521803-fd21-4b40-a4e5-6801b823d59d --path tools/viewer/e2e/tests/demo-viewer/auth-middleware.spec.ts --kind test`.

## Validation

- E2E green.
- Manual smoke per acceptance bullets.
