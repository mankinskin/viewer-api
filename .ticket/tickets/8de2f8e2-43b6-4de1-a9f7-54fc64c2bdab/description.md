# Feature page — server infra

Implement the demo page that showcases the `viewer-api/server-infra` feature surface.

## Backend

GET /api/demo/health, /api/demo/echo + page rendering config + static-files probe + CORS probe.

## Frontend

- Module: `frontend/dioxus/src/pages/server_infra.rs`.
- Add a route `/server-infra` to the Dioxus router and a sidebar nav entry.
- Page header includes a "Spec" link to spec-viewer for `viewer-api/server-infra`.
- All UI built from existing viewer-api components (`Layout`, `Panel`,
  `Spinner`, `CodeViewer`, etc.) — no novel components.

## Acceptance criteria

- All endpoints listed above are reachable and return the documented
  shape (see `viewer-api/server-infra` spec body for exact behavior).
- The page renders without console errors.
- Manual: open `http://localhost:3099/server-infra` and exercise every control.

## E2E test

- File: `tools/viewer/e2e/tests/demo-viewer/server-infra.spec.ts`.
- Cover **every** acceptance bullet listed in the spec body.
- After the test passes, register it as a `code_ref` on the spec:
  `spec refs add 59979a95-a4cb-4aa3-9a79-486b029532a3 --path tools/viewer/e2e/tests/demo-viewer/server-infra.spec.ts --kind test`.

## Validation

- E2E green.
- Manual smoke per acceptance bullets.
