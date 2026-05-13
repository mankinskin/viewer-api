# Feature page — dev proxy

Implement the demo page that showcases the `viewer-api/dev-proxy` feature surface.

## Backend

Wire DevProxyConfig behind --dev flag; add a /__dev_probe path; demo page reports mode + probes proxy.

## Frontend

- Module: `frontend/dioxus/src/pages/dev_proxy.rs`.
- Add a route `/dev-proxy` to the Dioxus router and a sidebar nav entry.
- Page header includes a "Spec" link to spec-viewer for `viewer-api/dev-proxy`.
- All UI built from existing viewer-api components (`Layout`, `Panel`,
  `Spinner`, `CodeViewer`, etc.) — no novel components.

## Acceptance criteria

- All endpoints listed above are reachable and return the documented
  shape (see `viewer-api/dev-proxy` spec body for exact behavior).
- The page renders without console errors.
- Manual: open `http://localhost:3099/dev-proxy` and exercise every control.

## E2E test

- File: `tools/viewer/e2e/tests/demo-viewer/dev-proxy.spec.ts`.
- Cover **every** acceptance bullet listed in the spec body.
- After the test passes, register it as a `code_ref` on the spec:
  `spec refs add b748e117-a847-474d-92ee-b58723cee612 --path tools/viewer/e2e/tests/demo-viewer/dev-proxy.spec.ts --kind test`.

## Validation

- E2E green.
- Manual smoke per acceptance bullets.
