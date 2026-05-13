# Feature page — icons spinner

Implement the demo page that showcases the `viewer-api/components/icons-spinner` feature surface.

## Scope

Icon gallery and Spinner sizes.

## Frontend

- Module: `frontend/dioxus/src/pages/icons_spinner.rs`.
- Add a route `/icons-spinner` to the Dioxus router and a sidebar nav entry.
- Reuse existing `viewer-api` components — no novel ones.
- Page header includes a "Spec" link to spec-viewer for `viewer-api/components/icons-spinner`.

## Acceptance criteria

- See the "Demo behavior" and "Acceptance behavior" sections of the
  `viewer-api/components/icons-spinner` spec; every bullet must be exercised.
- No console errors during normal use.

## E2E test

- File: `tools/viewer/e2e/tests/demo-viewer/icons-spinner.spec.ts`.
- After the test passes, register it as a `code_ref` on the spec:
  `spec refs add 7b43dfd1-39aa-4585-b5fe-dc57c6d57eba --path tools/viewer/e2e/tests/demo-viewer/icons-spinner.spec.ts --kind test`.

## Validation

- E2E green.
- Manual smoke per acceptance bullets.
