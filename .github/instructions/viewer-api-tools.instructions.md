---
description: "Use when editing viewer-api-driven tools (viewer-api, log-viewer, doc-viewer, ticket-viewer). Covers frontend conventions and integration boundaries with context-api, ticket-api, and filesystem-backed sources."
applyTo: "tools/viewer-api/**,tools/log-viewer/**,tools/doc-viewer/**,tools/ticket-viewer/**"
---

# Viewer API Tools Guidance

## Scope

Applies to viewer-api-driven HTTP/SPA tools and shared frontend packages.

## Viewer-API Shared Patterns

- Initialize server apps via `ServerConfig::new(name, port)` and prefer `.with_host()` / `.with_static_dir()` helpers.
- Use `init_tracing_full()` with `TracingConfig::from_env(...)` instead of custom tracing setup.
- Reuse `default_cors()` and `with_static_files()` from viewer-api for consistent HTTP behavior.
- For stable error responses in viewer-style APIs, prefer the shared API error envelope patterns from viewer-api or tool-local wrappers.
- For SSE output, use viewer-api SSE helpers instead of ad hoc event formatting.
- For JQ/filter behavior, reuse viewer-api query primitives; avoid introducing parallel query stacks.

## Integration Boundaries

- Treat `context-api` and `ticket-api` as system-of-record contracts.
- Keep business logic in API crates/services; viewers should focus on transport, presentation, and interaction.
- Preserve request/response compatibility unless the task explicitly requires an API change.
- If API behavior changes, update dependent viewer routes and docs in the same change.

## State Management Patterns

- For shared mutable backend state, follow `Arc<Mutex<_>>` app-state patterns used by adapter tools.
- For hot-reloadable runtime config (for example auth tokens), follow arc-swap style patterns used by HTTP tools.
- Keep session-like frontend/backend coordination in shared utilities when available.

## Frontend Rules

- Reuse shared UI primitives/styles from `tools/viewer-api/frontend` where possible.
- Keep viewer-specific features modular; avoid duplicating shared components.
- Prefer explicit loading/error/empty states for all async data views.
- Keep theme/effects integration centralized so log-viewer and ticket-viewer can share behavior.
- Preserve keyboard navigation and visible focus states for interactive controls.
- Verify responsive behavior for desktop, tablet, and narrow mobile widths.

## Filesystem and Source Access

- Constrain filesystem reads to configured roots (workspace/log/static roots).
- Normalize and validate paths before access; prevent path traversal.
- Prefer existing viewer-api helpers/utilities before adding new path logic.
- When local source is unavailable (remote/deployed mode), use configured remote source resolution paths.

## HTTP and Runtime Conventions

- Reuse `viewer-api` server utilities for tracing, CORS, static files, and CLI args.
- Keep router composition explicit and stable; avoid breaking endpoint names without migration.
- Preserve single-process viewer startup assumptions where present (for example ticket-viewer embedding ticket-http).

## Single-Process Viewer Pattern

- `ticket-viewer` embeds `ticket-http` routes directly and serves SPA + API on one process.
- For new viewers, prefer importing/mounting existing HTTP routers rather than introducing extra backend daemons.
- Keep static fallback routing behavior consistent with current SPA serving pattern.

## Dev Proxy Pattern

- In development, prefer viewer-api dev proxy/static-missing patterns for Vite workflows.
- Keep production static serving path simple and deterministic.

## Validation

Use this required validation ladder for frontend-impacting changes:

1. Static checks: run lint and typecheck for each affected frontend package.
2. Component checks: run nearest unit/component tests for changed UI modules.
3. Browser checks: run at least one browser flow that exercises changed UX paths.
4. Integration checks: verify viewer interaction with context-api and ticket-api contracts, or filesystem-backed endpoints, for changed paths.
5. Regression checks: when fixing a bug, include a reproducer assertion before or with the fix.
6. Evidence summary: report commands run, pass/fail outcome, and which UX states were validated.

## UX Definition of Done

For changed UI flows, confirm all of the following:

- Loading, empty, success, and error states exist and are user-visible.
- Error messages are actionable and include recovery options when possible.
- Keyboard-only interaction works for affected controls and navigation.
- Focus placement is deterministic after route or data transitions.
- Rendering remains usable at desktop, tablet, and mobile widths.

If MCP-facing behavior or docs changed, run documentation validation workflows.
