# Public surface contracts

Every viewer exposes a versioned public contract through `viewer-api` (HTTP routes, GraphQL schema, the shared frontend package). The contract is the integration surface for tests, MCP tools, and downstream automation, and it must be evolved deliberately.

## Owned surfaces

- The HTTP routes registered by `viewer-api/viewer-api` and re-exposed by each viewer's binary.
- The GraphQL schema served from the same routes, including request/response shapes for queries and subscriptions.
- The shared frontend Dioxus package and its TypeScript bindings consumed by each viewer's frontend crate.
- The Playwright end-to-end suites that exercise the surface (`viewer-api/viewer-api/frontend/dioxus/e2e/shared/` plus per-viewer wrappers).

## Versioning rules

- Additive changes (new routes, new GraphQL fields, new frontend props with defaults) are allowed without a version bump.
- Removing or renaming routes, fields, props, or events requires a version bump and a spec entry that documents the migration path.
- The shared E2E suites must keep passing across all consumers before a contract change is merged; failures in one viewer block the change for all viewers.

## Single source

Per-viewer crates must not duplicate the contract types or re-implement the shared frontend logic. If a viewer needs a behaviour that is not covered, the behaviour is added to `viewer-api` first and then consumed by every viewer in lockstep.
