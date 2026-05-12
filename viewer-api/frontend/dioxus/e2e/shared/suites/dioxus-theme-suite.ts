import type { ViewerConfig } from '../managed-viewers';
import { gotoAndWaitForViewer } from '../managed-viewers';
import { test, expect } from '../playwright-runtime';

export function registerDioxusThemeSuite(viewer: ViewerConfig): void {
  test.describe(`${viewer.name} — GPU overlay & theme settings`, () => {
    test('WebGPU canvas element is present in the DOM', async ({ page }) => {
      test.setTimeout(90_000);

      await gotoAndWaitForViewer(page, viewer);

      await expect(page.locator('#webgpu-canvas')).toBeAttached({ timeout: 5_000 });
    });

    test('theme settings panel opens and closes via the palette button', async ({ page }) => {
      test.setTimeout(90_000);

      await gotoAndWaitForViewer(page, viewer);

      const themeBtn = page.locator('button[aria-label="Theme settings"]');
      await expect(themeBtn).toBeVisible({ timeout: 30_000 });

      const panel = page.locator('.theme-settings');
      await expect(panel).not.toBeVisible();

      await themeBtn.click();
      await expect(panel).toBeVisible({ timeout: 5_000 });
      await expect(panel.locator('.glass-panel__title')).toContainText('Theme Settings');

      await panel.locator('button[aria-label="Close theme settings"]').click();
      await expect(panel).not.toBeVisible({ timeout: 5_000 });
    });

    test('GPU overlay master toggle defaults ON and toggles off/on without errors', async ({ page }) => {
      test.setTimeout(90_000);

      await gotoAndWaitForViewer(page, viewer);

      await page.evaluate(() => localStorage.removeItem('viewer-api-gpu-enabled'));
      await page.reload({ waitUntil: 'domcontentloaded' });
      await page.locator(viewer.readySelector).first().waitFor({
        state: 'visible',
        timeout: viewer.readyTimeout,
      });

      const themeBtn = page.locator('button[aria-label="Theme settings"]');
      await expect(themeBtn).toBeVisible({ timeout: 30_000 });
      await themeBtn.click();

      const panel = page.locator('.theme-settings');
      await expect(panel).toBeVisible({ timeout: 5_000 });

      const label = panel.locator('.theme-settings__effect-label', {
        hasText: 'Enable GPU overlay',
      });
      await expect(label).toBeVisible();

      const checkbox = panel.locator('.theme-settings__toggle-switch input[type="checkbox"]').first();
      await expect.poll(() => checkbox.evaluate((el: HTMLInputElement) => el.checked)).toBe(true);

      const jsErrors: string[] = [];
      page.on('pageerror', (err) => jsErrors.push(`pageerror: ${err.message}`));
      page.on('console', (msg) => {
        if (msg.type() !== 'error') return;
        const text = msg.text();
        if (text.includes('Failed to load resource')) return;
        jsErrors.push(`console.error: ${text}`);
      });

      const slider = panel.locator('.theme-settings__toggle-slider').first();
      await slider.click({ force: true });
      await expect.poll(() => checkbox.evaluate((el: HTMLInputElement) => el.checked)).toBe(false);

      await expect
        .poll(() => page.evaluate(() => localStorage.getItem('viewer-api-gpu-enabled')))
        .toBe('false');

      await slider.click({ force: true });
      await expect.poll(() => checkbox.evaluate((el: HTMLInputElement) => el.checked)).toBe(true);
      await expect
        .poll(() => page.evaluate(() => localStorage.getItem('viewer-api-gpu-enabled')))
        .toBe('true');

      expect(jsErrors, `${viewer.name} produced JS errors during toggle interaction`).toEqual([]);
    });

    test('every theme-settings toggle and button can be activated without JS errors', async ({ page }) => {
      test.setTimeout(120_000);

      const jsErrors: string[] = [];
      page.on('pageerror', (err) => jsErrors.push(`pageerror: ${err.message}`));
      page.on('console', (msg) => {
        if (msg.type() !== 'error') return;
        const text = msg.text();
        if (text.includes('Failed to load resource')) return;
        jsErrors.push(`console.error: ${text}`);
      });

      await gotoAndWaitForViewer(page, viewer);

      const themeBtn = page.locator('button[aria-label="Theme settings"]');
      await expect(themeBtn).toBeVisible({ timeout: 30_000 });
      await themeBtn.click();

      const panel = page.locator('.theme-settings');
      await expect(panel).toBeVisible({ timeout: 5_000 });

      const sliders = panel.locator('.theme-settings__toggle-slider');
      const sliderCount = await sliders.count();
      for (let i = 0; i < sliderCount; i += 1) {
        const slider = sliders.nth(i);
        await slider.click({ force: true });
        await page.waitForTimeout(50);
        await slider.click({ force: true });
        await page.waitForTimeout(50);
      }

      const presetCards = panel.locator('.theme-preset-card, .theme-settings__preset-button');
      const presetCount = await presetCards.count();
      for (let i = 0; i < presetCount; i += 1) {
        await presetCards.nth(i).click();
        await page.waitForTimeout(50);
      }

      const ranges = panel.locator('input[type="range"]');
      const rangeCount = await ranges.count();
      for (let i = 0; i < rangeCount; i += 1) {
        const range = ranges.nth(i);
        const max = await range.evaluate((el: HTMLInputElement) => el.max);
        const min = await range.evaluate((el: HTMLInputElement) => el.min);

        await range.evaluate((el: HTMLInputElement, value: string) => {
          el.value = value;
          el.dispatchEvent(new Event('input', { bubbles: true }));
          el.dispatchEvent(new Event('change', { bubbles: true }));
        }, min);
        await page.waitForTimeout(20);

        await range.evaluate((el: HTMLInputElement, value: string) => {
          el.value = value;
          el.dispatchEvent(new Event('input', { bubbles: true }));
          el.dispatchEvent(new Event('change', { bubbles: true }));
        }, max);
        await page.waitForTimeout(20);
      }

      const colorPickers = panel.locator('input[type="color"]');
      const colorCount = await colorPickers.count();
      for (let i = 0; i < colorCount; i += 1) {
        const colorPicker = colorPickers.nth(i);
        await colorPicker.evaluate((el: HTMLInputElement) => {
          el.value = '#abcdef';
          el.dispatchEvent(new Event('input', { bubbles: true }));
          el.dispatchEvent(new Event('change', { bubbles: true }));
        });
        await page.waitForTimeout(20);
      }

      const closeBtn = panel.locator('button[aria-label="Close theme settings"]');
      if ((await closeBtn.count()) > 0) {
        await closeBtn.click();
        await expect(panel).not.toBeVisible({ timeout: 5_000 });
      }

      expect(
        jsErrors,
        `${viewer.name} produced JS errors while exercising theme controls`,
      ).toEqual([]);
    });
  });
}