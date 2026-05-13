# Feature page — tree view

Implement the demo page that showcases the `viewer-api/components/tree-view` feature surface.

## Scope

TreeView with ~80 mock nodes, search, filters, sort, keyboard navigation.

## Frontend

- Module: `frontend/dioxus/src/pages/tree_view.rs`.
- Add a route `/tree-view` to the Dioxus router and a sidebar nav entry.
- Reuse existing `viewer-api` components — no novel ones.
- Page header includes a "Spec" link to spec-viewer for `viewer-api/components/tree-view`.

## Acceptance criteria

- See the "Demo behavior" and "Acceptance behavior" sections of the
  `viewer-api/components/tree-view` spec; every bullet must be exercised.
- No console errors during normal use.

## E2E test

- File: `tools/viewer/e2e/tests/demo-viewer/tree-view.spec.ts`.
- After the test passes, register it as a `code_ref` on the spec:
  `spec refs add a20a0395-4f3b-4b55-ba7a-a0c38ba9f7a6 --path tools/viewer/e2e/tests/demo-viewer/tree-view.spec.ts --kind test`.

## Validation

- E2E green.
- Manual smoke per acceptance bullets.
