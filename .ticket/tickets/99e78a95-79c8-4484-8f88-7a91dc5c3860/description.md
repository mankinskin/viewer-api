## Objective
Build stable viewer evidence using API fixtures and Playwright screenshots without data-dependent skips.

## Context
Known bad test: `memory-viewers/log-viewer/e2e/tests/theme-and-readability.spec.ts:300-302` uses `test.skip()` on empty data, turning regressions into silent passes.

## Acceptance criteria
- Use stable API fixtures and screenshot evidence.
- Eliminate data-dependent skip behavior.

## Out of scope
- Viewer redesign.