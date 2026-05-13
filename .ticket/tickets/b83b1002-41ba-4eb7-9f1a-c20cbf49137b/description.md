# E2E infrastructure

Add the demo-viewer e2e harness so per-feature tickets only need to
write the test body.

## Files

- `tools/viewer/e2e/tests/demo-viewer/_helpers.ts`
  - `DEMO_VIEWER` URL constant (driven by env `DEMO_VIEWER_URL`,
    default `http://localhost:3099`).
  - `gotoDemo(page, slug)` helper.
  - `expectNoConsoleErrors(page)` collector + assertion (filter the
    "Failed to load resource 404" noise as in
    `tools/viewer/e2e/tests/dioxus/graph3d-right-drag.spec.ts`).
  - `withWebGpu(test)` profile: re-uses the Vulkan/swiftshader launch
    flags + `headless: false` from the existing graph3d test.
- `tools/viewer/e2e/tests/demo-viewer/smoke.spec.ts`
  - Asserts `/api/demo/health` returns 200 and the SPA shell renders
    every feature nav entry.
- `tools/viewer/e2e/playwright.config.ts` — register a `demo-viewer`
  project that auto-starts `viewer-ctl start demo-viewer` (managed
  webServer, port 3099, `reuseExistingServer: true`).

## Acceptance criteria

- `npx playwright test --project=demo-viewer tests/demo-viewer/smoke.spec.ts`
  passes from a clean checkout (assuming `viewer-ctl prepare demo-viewer`
  has been run).
- The helpers compile under the existing tsconfig.
