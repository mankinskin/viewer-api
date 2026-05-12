import type { Page } from '@playwright/test';

const VIEWER_HOST = '127.0.0.1';

export interface ViewerConfig {
  name: string;
  url: string;
  readySelector: string;
  readyTimeout: number;
}

export const LOG_VIEWER: ViewerConfig = {
  name: 'log-viewer',
  url: `http://${VIEWER_HOST}:3000`,
  readySelector: '.tab-bar',
  readyTimeout: 20_000,
};

export const DOC_VIEWER: ViewerConfig = {
  name: 'doc-viewer',
  url: `http://${VIEWER_HOST}:3001`,
  readySelector: '.app',
  readyTimeout: 20_000,
};

export const SPEC_VIEWER: ViewerConfig = {
  name: 'spec-viewer',
  url: `http://${VIEWER_HOST}:4002`,
  readySelector: 'header.header',
  readyTimeout: 60_000,
};

export async function gotoAndWaitForViewer(page: Page, viewer: ViewerConfig): Promise<void> {
  await page.goto(viewer.url, { waitUntil: 'domcontentloaded' });
  await page.locator(viewer.readySelector).first().waitFor({
    state: 'visible',
    timeout: viewer.readyTimeout,
  });
}