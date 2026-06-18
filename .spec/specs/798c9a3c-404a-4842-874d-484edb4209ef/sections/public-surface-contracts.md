<!-- spec-api:file generated=true -->

<!-- spec-api:entry id=62e1317d-a4a1-405b-8113-686594c927cb slug=viewer-api/recurring-principles/public-surface-contracts/public-surface-contracts/l1 -->
# Public surface contracts

Every viewer exposes a versioned public contract through `viewer-api` (HTTP routes, GraphQL schema, the shared frontend package). The contract is the integration surface for tests, MCP tools, and downstream automation, and it must be evolved deliberately.

<!-- spec-api:entry id=5fb89c68-fd49-4baa-8693-b3c04b603f30 slug=viewer-api/recurring-principles/public-surface-contracts/public-surface-contracts/owned-surfaces/l5 -->
## Owned surfaces

- The HTTP routes registered by `viewer-api/viewer-api` and re-exposed by each viewer's binary.
- The GraphQL schema served from the same routes, including request/response shapes for queries and subscriptions.
- The shared frontend Dioxus package and its TypeScript bindings consumed by each viewer's frontend crate.
- The Playwright end-to-end suites that exercise the surface (`viewer-api/viewer-api/frontend/dioxus/e2e/shared/` plus per-viewer wrappers).

<!-- spec-api:entry id=8219b071-c541-4dc1-a034-e083d9361ce3 slug=viewer-api/recurring-principles/public-surface-contracts/public-surface-contracts/versioning-rules/l12 -->
## Versioning rules

- Additive changes (new routes, new GraphQL fields, new frontend props with defaults) are allowed without a version bump.
- Removing or renaming routes, fields, props, or events requires a version bump and a spec entry that documents the migration path.
- The shared E2E suites must keep passing across all consumers before a contract change is merged; failures in one viewer block the change for all viewers.

<!-- spec-api:entry id=2ac29fd5-cc16-43f1-86c7-9a2e2b6f83c7 slug=viewer-api/recurring-principles/public-surface-contracts/public-surface-contracts/single-source/l18 -->
## Single source

Per-viewer crates must not duplicate the contract types or re-implement the shared frontend logic. If a viewer needs a behaviour that is not covered, the behaviour is added to `viewer-api` first and then consumed by every viewer in lockstep.
