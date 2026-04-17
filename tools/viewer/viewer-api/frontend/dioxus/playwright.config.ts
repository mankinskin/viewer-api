import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for viewer-api-dioxus e2e tests.
 *
 * Tests target the `dx serve` demo app.  Start the server before running:
 *
 *   cd tools/viewer/viewer-api-dioxus
 *   dx serve --port 18080 --open false
 *
 * Then in a separate terminal:
 *
 *   npm test:e2e           (headless)
 *   npm test:e2e:headed    (visible browser)
 *   npm test:e2e:ui        (Playwright UI mode)
 *
 * No `webServer` block is used because the first WASM compilation can take
 * 60–120 s.  Run `dx serve` manually so you can see build progress.
 */

const DEV_SERVER_URL = 'http://localhost:18080';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: false, // share the single dx-serve process
  forbidOnly: !!process.env['CI'],
  retries: process.env['CI'] ? 1 : 0,
  workers: 1,
  reporter: [['list'], ['html', { outputFolder: 'playwright-report', open: 'never' }]],

  use: {
    baseURL: DEV_SERVER_URL,
    // Traces help diagnose WASM-specific timing issues.
    trace: 'on-first-retry',
    // WASM hydration is slower than JS-only apps.
    actionTimeout: 20_000,
    navigationTimeout: 30_000,
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});
