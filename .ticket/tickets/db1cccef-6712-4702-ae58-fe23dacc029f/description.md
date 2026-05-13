# Feature page — client log

Implement the demo page that showcases the `viewer-api/client-log` feature surface.

## Backend

Reuse client_log_router; add /api/demo/client-log/recent in-memory tail; WASM buttons that emit at each log level.

## Frontend

- Module: `frontend/dioxus/src/pages/client_log.rs`.
- Add a route `/client-log` to the Dioxus router and a sidebar nav entry.
- Page header includes a "Spec" link to spec-viewer for `viewer-api/client-log`.
- All UI built from existing viewer-api components (`Layout`, `Panel`,
  `Spinner`, `CodeViewer`, etc.) — no novel components.

## Acceptance criteria

- All endpoints listed above are reachable and return the documented
  shape (see `viewer-api/client-log` spec body for exact behavior).
- The page renders without console errors.
- Manual: open `http://localhost:3099/client-log` and exercise every control.

## E2E test

- File: `tools/viewer/e2e/tests/demo-viewer/client-log.spec.ts`.
- Cover **every** acceptance bullet listed in the spec body.
- After the test passes, register it as a `code_ref` on the spec:
  `spec refs add c6e3cc79-a2de-49f1-9c99-effe1b64a873 --path tools/viewer/e2e/tests/demo-viewer/client-log.spec.ts --kind test`.

## Validation

- E2E green.
- Manual smoke per acceptance bullets.
