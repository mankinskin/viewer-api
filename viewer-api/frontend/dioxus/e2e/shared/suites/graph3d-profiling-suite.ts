import type { ViewerConfig } from '../managed-viewers';
import { test, expect } from '../playwright-runtime';
import { profileArtifactPath, withBrowserTrace } from '../profiling';

/**
 * Suite: capture a Chromium performance trace while the 3-D graph loads and
 * renders its first frames.
 *
 * The trace is written to `playwright-report/profiles/<viewer>-graph3d.json`
 * and attached to the Playwright report. When the viewer is served from a
 * `--features profile-browser` build, the trace also contains the
 * `tracing-wasm` `performance.measure` marks for `graph3d::render_frame`
 * (category `blink.user_timing`).
 *
 * The assertion is intentionally lightweight (the graph mounts and the trace
 * file is produced) so this stays a profiling-capture suite, not a perf gate.
 * Threshold-based regression gates belong in the wasm micro-benchmarks.
 */
export function registerGraph3dProfilingSuite(viewer: ViewerConfig): void {
  test.describe(`${viewer.name} — graph3d profiling capture`, () => {
    test('captures a Chromium trace of graph load + first frames', async ({ page, browser }) => {
      test.setTimeout(120_000);

      await page.goto(viewer.url, { waitUntil: 'domcontentloaded' });
      await page.locator(viewer.readySelector).first().waitFor({
        state: 'visible',
        timeout: viewer.readyTimeout,
      });

      const tracePath = profileArtifactPath(`${viewer.name}-graph3d`);
      const { tracePath: written } = await withBrowserTrace(
        browser,
        page,
        { path: tracePath, screenshots: false },
        async () => {
          // Let the WebGPU canvas mount and run a few render frames so the
          // per-frame `render_frame` spans are exercised under tracing.
          await page.waitForTimeout(3_000);
        },
      );

      await test.info().attach('graph3d-chrome-profile', {
        path: written,
        contentType: 'application/json',
      });

      expect(written).toContain('graph3d');
    });
  });
}
