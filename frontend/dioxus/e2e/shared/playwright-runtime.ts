import { createRequire } from 'node:module';

const requireFromRunner = createRequire(`${process.cwd()}/package.json`);

export const { test, expect } = requireFromRunner('@playwright/test') as typeof import('@playwright/test');

const SESSION_KEY = 'viewer-api-session-id';

export function buildE2eCorrelationId(testTitle: string): string {
	const normalized = testTitle
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, '-')
		.replace(/^-+|-+$/g, '')
		.slice(0, 48);
	const suffix = Math.random().toString(16).slice(2, 10);
	return `e2e-${normalized || 'test'}-${suffix}`;
}

export async function applyE2eSessionCorrelation(
	page: import('@playwright/test').Page,
	testTitle: string,
): Promise<string> {
	const correlationId = buildE2eCorrelationId(testTitle);
	await page.addInitScript(
		([key, value]) => {
			try {
				window.sessionStorage.setItem(key, value);
			} catch {
				// Best-effort only; tests still continue if storage is unavailable.
			}
		},
		[SESSION_KEY, correlationId] as const,
	);
	return correlationId;
}