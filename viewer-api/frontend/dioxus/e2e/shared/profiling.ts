import { mkdir } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';

import type { Browser, Page } from '@playwright/test';

/**
 * Tracing categories required to capture both raw CPU samples and the
 * `tracing-wasm` timeline marks emitted by `profile_scope!` spans.
 *
 * `blink.user_timing` is MANDATORY: it is the category under which
 * `performance.measure` calls (produced by the tracing-wasm console layer when
 * the viewer is built with `--features profile-browser`) are recorded.
 */
export const PROFILE_TRACE_CATEGORIES = [
  'disabled-by-default-v8.cpu_profiler',
  'blink.user_timing',
  'devtools.timeline',
] as const;

/** Default directory for captured Chromium trace artifacts. */
export const PROFILE_OUTPUT_DIR = resolve(process.cwd(), 'playwright-report', 'profiles');

export interface BrowserTraceOptions {
  /** Output path for the `chrome-profile.json` trace. */
  path: string;
  /** Override the default capture categories. */
  categories?: readonly string[];
  /** Capture per-frame screenshots into the trace (heavier). */
  screenshots?: boolean;
}

/**
 * Wrap an in-page workload with `chromium.startTracing` / `stopTracing` and
 * write the resulting Chromium trace to `options.path`.
 *
 * The returned object exposes the absolute artifact path so the caller can
 * attach it to the test report or post-process it with the DevTools timeline
 * tooling.
 *
 * Requires the viewer under test to be served from a build compiled with
 * `--features profile-browser` for the WASM `performance.measure` marks to
 * appear; the CPU profiler samples are captured regardless.
 */
export async function withBrowserTrace<T>(
  browser: Browser,
  page: Page,
  options: BrowserTraceOptions,
  workload: () => Promise<T>,
): Promise<{ result: T; tracePath: string }> {
  const tracePath = resolve(options.path);
  await mkdir(dirname(tracePath), { recursive: true });

  await browser.startTracing(page, {
    path: tracePath,
    screenshots: options.screenshots ?? false,
    categories: [...(options.categories ?? PROFILE_TRACE_CATEGORIES)],
  });

  let result: T;
  try {
    result = await workload();
  } finally {
    await browser.stopTracing();
  }

  return { result, tracePath };
}

/** Build a stable trace artifact path under {@link PROFILE_OUTPUT_DIR}. */
export function profileArtifactPath(name: string): string {
  return join(PROFILE_OUTPUT_DIR, `${name}.json`);
}
