import type { Page } from '@playwright/test';

/** Static asset extensions — 404s for these indicate a broken build. */
export const STATIC_EXTENSIONS = /\.(js|ts|css|wasm|png|svg|ico|woff2?)(\?.*)?$/i;

export interface ViewerLoadResult {
  /** console.error() messages and uncaught page exceptions. */
  errors: string[];
  /** URLs of responses with HTTP 404 that match static asset patterns. */
  missingAssets: string[];
}

/**
 * Navigate to `url`, wait for a ready selector, and collect runtime errors +
 * missing static assets during initial load.
 */
export async function loadAndInspectViewer(
  page: Page,
  url: string,
  readySelector: string,
  readyTimeout: number,
  settleMs = 2_000,
): Promise<ViewerLoadResult> {
  const errors: string[] = [];
  const missingAssets: string[] = [];

  page.on('pageerror', (err) => {
    errors.push(`pageerror: ${err.message}`);
  });

  page.on('console', (msg) => {
    if (msg.type() === 'error') {
      errors.push(`console.error: ${msg.text()}`);
    }
  });

  page.on('response', (response) => {
    if (response.status() === 404) {
      const responseUrl = response.url();
      if (STATIC_EXTENSIONS.test(responseUrl)) {
        missingAssets.push(responseUrl);
      }
    }
  });

  await page.goto(url, { waitUntil: 'domcontentloaded' });
  await page.locator(readySelector).first().waitFor({
    state: 'visible',
    timeout: readyTimeout,
  });

  await page.waitForTimeout(settleMs);
  return { errors, missingAssets };
}
