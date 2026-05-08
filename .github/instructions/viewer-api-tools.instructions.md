<!-- rule-api:file generated=true -->

<!-- rule-api:entry id=469e4b4c-5c0f-42b5-a8a5-c31cb4e5641e slug=shared/instructions/viewer-api-tools/viewer-api-tools-instructions/l1 -->
---
description: "Use when editing viewer-api-driven tools (viewer-api, log-viewer, doc-viewer, ticket-viewer). Covers frontend conventions and integration boundaries with context-api, ticket-api, and filesystem-backed sources."
applyTo: "tools/viewer/viewer-api/**,tools/viewer/log-viewer/**,tools/viewer/doc-viewer/**,tools/viewer/ticket-viewer/**"
---

<!-- rule-api:entry id=9039f3da-3df2-4bba-8d91-69788fea1765 slug=shared/instructions/viewer-api-tools/viewer-api-tools-guidance/scope/l8 -->
## Scope

Applies to viewer-api-driven HTTP/SPA tools and shared frontend packages.

<!-- rule-api:entry id=8f9fde57-96cc-465a-863f-0d865aa73bda slug=shared/instructions/viewer-api-tools/viewer-api-tools-guidance/viewer-api-shared-patterns/l12 -->
## Viewer-API Shared Patterns

- Initialize server apps via `ServerConfig::new(name, port)` and prefer `.with_host()` / `.with_static_dir()` helpers.
- Use `init_tracing_full()` with `TracingConfig::from_env(...)` instead of custom tracing setup.
- Reuse `default_cors()` and `with_static_files()` from viewer-api for consistent HTTP behavior.
- For stable error responses in viewer-style APIs, prefer the shared API error envelope patterns from viewer-api or tool-local wrappers.
- For SSE output, use viewer-api SSE helpers instead of ad hoc event formatting.
- For JQ/filter behavior, reuse viewer-api query primitives; avoid introducing parallel query stacks.

<!-- rule-api:entry id=a66e4443-6299-48f7-ba76-a8224da89dcb slug=shared/instructions/viewer-api-tools/viewer-api-tools-guidance/integration-boundaries/l21 -->
## Integration Boundaries

- Treat `context-api` and `ticket-api` as system-of-record contracts.
- Keep business logic in API crates/services; viewers should focus on transport, presentation, and interaction.
- Preserve request/response compatibility unless the task explicitly requires an API change.
- If API behavior changes, update dependent viewer routes and docs in the same change.

<!-- rule-api:entry id=97087b14-aaf2-4ae4-9812-0691a44fca55 slug=shared/instructions/viewer-api-tools/viewer-api-tools-guidance/state-management-patterns/l28 -->
## State Management Patterns

- For shared mutable backend state, follow `Arc<Mutex<_>>` app-state patterns used by adapter tools.
- For hot-reloadable runtime config (for example auth tokens), follow arc-swap style patterns used by HTTP tools.
- Keep session-like frontend/backend coordination in shared utilities when available.

<!-- rule-api:entry id=888100f3-1b66-4c3b-b7ef-23196f6768ef slug=shared/instructions/viewer-api-tools/viewer-api-tools-guidance/frontend-rules/l34 -->
## Frontend Rules

- Reuse shared UI primitives/styles from `tools/viewer/viewer-api/frontend/ts` where possible.
- Keep viewer-specific features modular; avoid duplicating shared components.
- Prefer explicit loading/error/empty states for all async data views.
- Keep theme/effects integration centralized so log-viewer and ticket-viewer can share behavior.
- Preserve keyboard navigation and visible focus states for interactive controls.
- Verify responsive behavior for desktop, tablet, and narrow mobile widths.

<!-- rule-api:entry id=cb0f3062-03e8-4b6b-85e1-b899d12fc459 slug=shared/instructions/viewer-api-tools/viewer-api-tools-guidance/filesystem-and-source-access/l43 -->
## Filesystem and Source Access

- Constrain filesystem reads to configured roots (workspace/log/static roots).
- Normalize and validate paths before access; prevent path traversal.
- Prefer existing viewer-api helpers/utilities before adding new path logic.
- When local source is unavailable (remote/deployed mode), use configured remote source resolution paths.

<!-- rule-api:entry id=14cf31b5-439f-4535-87b1-1042c4ed4457 slug=shared/instructions/viewer-api-tools/viewer-api-tools-guidance/http-and-runtime-conventions/l50 -->
## HTTP and Runtime Conventions

- Reuse `viewer-api` server utilities for tracing, CORS, static files, and CLI args.
- Keep router composition explicit and stable; avoid breaking endpoint names without migration.
- Preserve single-process viewer startup assumptions where present (for example ticket-viewer embedding ticket-http).

<!-- rule-api:entry id=39ceeb19-60ee-41db-8147-7f06d20ae0a9 slug=shared/instructions/viewer-api-tools/viewer-api-tools-guidance/single-process-viewer-pattern/l56 -->
## Single-Process Viewer Pattern

- `ticket-viewer` embeds `ticket-http` routes directly and serves SPA + API on one process.
- For new viewers, prefer importing/mounting existing HTTP routers rather than introducing extra backend daemons.
- Keep static fallback routing behavior consistent with current SPA serving pattern.

<!-- rule-api:entry id=c5f3353c-54e7-4664-b435-58470f3e4f81 slug=shared/instructions/viewer-api-tools/viewer-api-tools-guidance/dev-proxy-pattern/l62 -->
## Dev Proxy Pattern

- In development, prefer viewer-api dev proxy/static-missing patterns for Vite workflows.
- Keep production static serving path simple and deterministic.

<!-- rule-api:entry id=fe953ae1-ec4d-4b77-a262-e833a684d9cb slug=shared/instructions/viewer-api-tools/viewer-api-tools-guidance/validation/l67 -->
## Validation

Use this required validation ladder for frontend-impacting changes:

<!-- rule-api:entry id=43572b7a-4ea6-4543-9d4e-8bd487b59bda slug=shared/instructions/viewer-api-tools/viewer-api-tools-guidance/validation/l71 -->
1. Static checks: run lint and typecheck for each affected frontend package.
2. Component checks: run nearest unit/component tests for changed UI modules.
3. Browser checks: run at least one browser flow that exercises changed UX paths.
4. Integration checks: verify viewer interaction with context-api and ticket-api contracts, or filesystem-backed endpoints, for changed paths.
5. Regression checks: when fixing a bug, include a reproducer assertion before or with the fix.
6. Evidence summary: report commands run, pass/fail outcome, and which UX states were validated.

<!-- rule-api:entry id=36bbe2ba-c51f-4b79-841f-86fc61755744 slug=shared/instructions/viewer-api-tools/viewer-api-tools-guidance/ux-definition-of-done/l78 -->
## UX Definition of Done

For changed UI flows, confirm all of the following:

<!-- rule-api:entry id=ce272ac4-a185-4aa4-a9cd-194a4c42e4b2 slug=shared/instructions/viewer-api-tools/viewer-api-tools-guidance/ux-definition-of-done/l82 -->
- Loading, empty, success, and error states exist and are user-visible.
- Error messages are actionable and include recovery options when possible.
- Keyboard-only interaction works for affected controls and navigation.
- Focus placement is deterministic after route or data transitions.
- Rendering remains usable at desktop, tablet, and mobile widths.

<!-- rule-api:entry id=03dfd81d-1eec-4894-9876-1becc4b928c5 slug=shared/instructions/viewer-api-tools/viewer-api-tools-guidance/ux-definition-of-done/l88 -->
If MCP-facing behavior or docs changed, run documentation validation workflows.
