# Feature page — theme settings

Implement the demo page that showcases the `viewer-api/theme-settings` feature surface.

## Scope

Mount the canonical ThemeSettings panel; verify presets + per-effect toggles.

## Frontend

- Module: `frontend/dioxus/src/pages/theme_settings.rs`.
- Add a route `/theme-settings` to the Dioxus router and a sidebar nav entry.
- Reuse existing `viewer-api` components — no novel ones.
- Page header includes a "Spec" link to spec-viewer for `viewer-api/theme-settings`.

## Acceptance criteria

- See the "Demo behavior" and "Acceptance behavior" sections of the
  `viewer-api/theme-settings` spec; every bullet must be exercised.
- No console errors during normal use.

## E2E test

- File: `tools/viewer/e2e/tests/demo-viewer/theme-settings.spec.ts`.
- After the test passes, register it as a `code_ref` on the spec:
  `spec refs add 36ebdecb-0b9e-47be-9f44-fe575aa6ad6f --path tools/viewer/e2e/tests/demo-viewer/theme-settings.spec.ts --kind test`.

## Validation

- E2E green.
- Manual smoke per acceptance bullets.
