# Summary

Implement a concrete doc-viewer migration path that replaces the current Preact-first shell with a Dioxus frontend built on `viewer-api-dioxus` and backed by `doc-http` for its server-facing document queries.

## Motivation

There is already shared work to bring doc-viewer UX primitives into `viewer-api-dioxus`, and there is an earlier deferred umbrella about eventually migrating doc-viewer. What is still missing is a concrete implementation ticket for the chosen end state: doc-viewer as a Dioxus viewer using shared viewer-api components and a dedicated `doc-http` backend surface.

## Scope

- Build the main doc-viewer frontend path on `viewer-api-dioxus` instead of the existing Preact shell.
- Consume `doc-http` endpoints for repository docs, generated-doc navigation, and content loading.
- Reuse shared viewer-api layout, tree, tabs, breadcrumbs, filter, and file/content primitives where possible.
- Preserve doc-viewer-specific browsing behavior while moving backend-specific logic behind `doc-http`.

## Acceptance Criteria

- The primary doc-viewer frontend is Dioxus-based and uses shared `viewer-api-dioxus` primitives for navigation and document display.
- The frontend loads its document tree and content from `doc-http` rather than bespoke doc-viewer-only server routes.
- Existing core doc-viewer flows remain covered by browser validation and Playwright coverage.
- The migration plan clearly identifies any remaining Preact-only fallback or cleanup work instead of leaving an implicit dual-runtime steady state.
- `viewer-ctl` and managed-viewer flows continue to start doc-viewer successfully after the migration.