## Owned surfaces

- The HTTP routes registered by `viewer-api/viewer-api` and re-exposed by each viewer's binary.
- The GraphQL schema served from the same routes, including request/response shapes for queries and subscriptions.
- The shared frontend Dioxus package and its TypeScript bindings consumed by each viewer's frontend crate.
- The Playwright end-to-end suites that exercise the surface (`memory-viewers/viewer-api/viewer-api/frontend/dioxus/e2e/shared/` plus per-viewer wrappers).