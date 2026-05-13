# Feature page — tab bar

Implement the demo page that showcases the `viewer-api/components/tab-bar` feature surface.

## Scope

TabBar with add/close/dirty/reorder/overflow.

## Frontend

- Module: `frontend/dioxus/src/pages/tab_bar.rs`.
- Add a route `/tab-bar` to the Dioxus router and a sidebar nav entry.
- Reuse existing `viewer-api` components — no novel ones.
- Page header includes a "Spec" link to spec-viewer for `viewer-api/components/tab-bar`.

## Acceptance criteria

- See the "Demo behavior" and "Acceptance behavior" sections of the
  `viewer-api/components/tab-bar` spec; every bullet must be exercised.
- No console errors during normal use.

## E2E test

- File: `tools/viewer/e2e/tests/demo-viewer/tab-bar.spec.ts`.
- After the test passes, register it as a `code_ref` on the spec:
  `spec refs add 348e17f7-23a8-4e11-bbb3-224cf0bbe9d6 --path tools/viewer/e2e/tests/demo-viewer/tab-bar.spec.ts --kind test`.

## Validation

- E2E green.
- Manual smoke per acceptance bullets.
