import { loadAndInspectViewer } from '../../test_apis';
import { test, expect } from '../playwright-runtime';
import type { ViewerConfig } from '../managed-viewers';

export function registerCommonViewerSuite(viewer: ViewerConfig): void {
  test.describe(`${viewer.name} — common suite`, () => {
    test('renders without console errors or uncaught exceptions', async ({ page }) => {
      test.setTimeout(90_000);

      const { errors } = await loadAndInspectViewer(
        page,
        viewer.url,
        viewer.readySelector,
        viewer.readyTimeout,
      );

      expect(errors, `${viewer.name} produced JS errors after loading`).toEqual([]);
    });

    test('no missing static assets (no 404 for JS/CSS/WASM)', async ({ page }) => {
      test.setTimeout(90_000);

      const { missingAssets } = await loadAndInspectViewer(
        page,
        viewer.url,
        viewer.readySelector,
        viewer.readyTimeout,
      );

      expect(missingAssets, `${viewer.name} has missing static assets`).toEqual([]);
    });

    test('ready-selector is visible after load', async ({ page }) => {
      test.setTimeout(90_000);

      await page.goto(viewer.url, { waitUntil: 'domcontentloaded' });
      await expect(page.locator(viewer.readySelector).first()).toBeVisible({
        timeout: viewer.readyTimeout,
      });
    });
  });
}