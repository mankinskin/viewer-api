# Feature page — code viewer

Implement the demo page that showcases the `viewer-api/components/code-viewer` feature surface.

## Scope

CodeViewer + FileContentViewer with language switcher, highlight, wrap toggle.

## Frontend

- Module: `frontend/dioxus/src/pages/code_viewer.rs`.
- Add a route `/code-viewer` to the Dioxus router and a sidebar nav entry.
- Reuse existing `viewer-api` components — no novel ones.
- Page header includes a "Spec" link to spec-viewer for `viewer-api/components/code-viewer`.

## Acceptance criteria

- See the "Demo behavior" and "Acceptance behavior" sections of the
  `viewer-api/components/code-viewer` spec; every bullet must be exercised.
- No console errors during normal use.

## E2E test

- File: `tools/viewer/e2e/tests/demo-viewer/code-viewer.spec.ts`.
- After the test passes, register it as a `code_ref` on the spec:
  `spec refs add df67eee9-08a0-4a6e-b1ff-b483599d232d --path tools/viewer/e2e/tests/demo-viewer/code-viewer.spec.ts --kind test`.

## Validation

- E2E green.
- Manual smoke per acceptance bullets.
