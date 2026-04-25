import type { Page } from '@playwright/test';

/** Read selected row labels from a viewer-api TreeView instance. */
export async function getSelectedTreeLabels(page: Page): Promise<string[]> {
  return page.evaluate(() =>
    Array.from(document.querySelectorAll('.tree-item-row.selected .tree-label')).map((el) =>
      (el.textContent ?? '').trim(),
    ),
  );
}

/** Read a hash parameter value from the current page URL. */
export async function getHashParam(page: Page, key: string): Promise<string | null> {
  return page.evaluate((hashKey) => {
    const hash = window.location.hash.startsWith('#')
      ? window.location.hash.slice(1)
      : window.location.hash;
    const params = new URLSearchParams(hash);
    return params.get(hashKey);
  }, key);
}
