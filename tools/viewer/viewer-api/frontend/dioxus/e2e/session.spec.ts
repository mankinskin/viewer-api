/**
 * E2E tests for session ID utilities.
 *
 * All tests drive the SessionDemo harness rendered inside the viewer-api-dioxus
 * demo app.  The dx serve server must be running on port 8080 before these
 * tests are executed.
 *
 * Coverage:
 *  - get_session_id: returns a valid RFC 4122 v4 UUID on first load
 *  - Persistence: session ID survives a same-tab page reload (sessionStorage)
 *  - clear_session + get_session_id: generates a fresh UUID after clearing
 *  - with_session: injects the X-Session-Id header alongside existing headers
 */

import { test, expect, Page } from '@playwright/test';

// UUID v4 pattern: xxxxxxxx-xxxx-4xxx-[89ab]xxx-xxxxxxxxxxxx
const UUID_V4_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

// ── Hydration helper ──────────────────────────────────────────────────────────

async function waitForApp(page: Page): Promise<void> {
  await page.waitForSelector('[data-testid="session-demo"]', { timeout: 30_000 });
}

// ── Fixtures ──────────────────────────────────────────────────────────────────

test.beforeEach(async ({ page }) => {
  // Clear sessionStorage so every test gets a fresh session unless the test
  // explicitly wants persistence across reload.
  await page.goto('/');
  await waitForApp(page);
  await page.evaluate(() => {
    window.sessionStorage.removeItem('viewer-api-session-id');
  });
  // Reload so the Rust code reads the now-absent key and generates a fresh UUID.
  await page.reload();
  await waitForApp(page);
});

// ── Tests ─────────────────────────────────────────────────────────────────────

test.describe('session — get_session_id', () => {
  test('displays a valid UUID v4 on first load', async ({ page }) => {
    const id = (
      await page.locator('[data-testid="session-id"]').textContent()
    )?.trim() ?? '';

    expect(id).toMatch(UUID_V4_RE);
  });

  test('calling refresh returns the same ID (sessionStorage persistence)', async ({
    page,
  }) => {
    const id1 = (
      await page.locator('[data-testid="session-id"]').textContent()
    )?.trim() ?? '';

    // Click refresh — calls get_session_id() again which reads sessionStorage.
    await page.click('[data-testid="session-refresh-btn"]');

    const id2 = (
      await page.locator('[data-testid="session-id"]').textContent()
    )?.trim() ?? '';

    expect(id1).toBe(id2);
    expect(id2).toMatch(UUID_V4_RE);
  });
});

test.describe('session — persistence across reload', () => {
  test('same UUID is restored after a page reload', async ({ page }) => {
    const id1 = (
      await page.locator('[data-testid="session-id"]').textContent()
    )?.trim() ?? '';

    await page.reload();
    await waitForApp(page);

    const id2 = (
      await page.locator('[data-testid="session-id"]').textContent()
    )?.trim() ?? '';

    // sessionStorage survives same-tab reload — UUID must be identical.
    expect(id1).toBe(id2);
  });
});

test.describe('session — clear_session + get_session_id', () => {
  test('generates a different UUID after clearing', async ({ page }) => {
    const idBefore = (
      await page.locator('[data-testid="session-id"]').textContent()
    )?.trim() ?? '';
    expect(idBefore).toMatch(UUID_V4_RE);

    // Clear the session (removes the sessionStorage entry).
    await page.click('[data-testid="session-clear-btn"]');

    // Refresh: calls get_session_id() which now finds no stored ID and
    // generates a fresh one.
    await page.click('[data-testid="session-refresh-btn"]');

    const idAfter = (
      await page.locator('[data-testid="session-id"]').textContent()
    )?.trim() ?? '';
    expect(idAfter).toMatch(UUID_V4_RE);
    expect(idAfter).not.toBe(idBefore);
  });

  test('new UUID after clear is persisted (second refresh returns same ID)', async ({
    page,
  }) => {
    await page.click('[data-testid="session-clear-btn"]');
    await page.click('[data-testid="session-refresh-btn"]');

    const id1 = (
      await page.locator('[data-testid="session-id"]').textContent()
    )?.trim() ?? '';

    // Second refresh should still return the same ID (now stored in sessionStorage).
    await page.click('[data-testid="session-refresh-btn"]');

    const id2 = (
      await page.locator('[data-testid="session-id"]').textContent()
    )?.trim() ?? '';

    expect(id1).toBe(id2);
  });
});

test.describe('session — with_session', () => {
  test('with_session output contains X-Session-Id header', async ({ page }) => {
    const output = (
      await page.locator('[data-testid="with-session-output"]').textContent()
    )?.trim() ?? '';

    expect(output).toContain('X-Session-Id:');
  });

  test('with_session output contains Content-Type header', async ({ page }) => {
    const output = (
      await page.locator('[data-testid="with-session-output"]').textContent()
    )?.trim() ?? '';

    expect(output).toContain('Content-Type: application/json');
  });

  test('X-Session-Id value in headers matches the displayed session ID', async ({
    page,
  }) => {
    const sessionId = (
      await page.locator('[data-testid="session-id"]').textContent()
    )?.trim() ?? '';

    const output = (
      await page.locator('[data-testid="with-session-output"]').textContent()
    )?.trim() ?? '';

    // The injected header value must match the displayed session ID.
    expect(output).toContain(`X-Session-Id: ${sessionId}`);
  });

  test('after clear+refresh the X-Session-Id in headers updates too', async ({
    page,
  }) => {
    const idBefore = (
      await page.locator('[data-testid="session-id"]').textContent()
    )?.trim() ?? '';

    await page.click('[data-testid="session-clear-btn"]');
    await page.click('[data-testid="session-refresh-btn"]');

    const idAfter = (
      await page.locator('[data-testid="session-id"]').textContent()
    )?.trim() ?? '';
    const output = (
      await page.locator('[data-testid="with-session-output"]').textContent()
    )?.trim() ?? '';

    expect(idAfter).not.toBe(idBefore);
    expect(output).toContain(`X-Session-Id: ${idAfter}`);
    expect(output).not.toContain(idBefore);
  });
});
