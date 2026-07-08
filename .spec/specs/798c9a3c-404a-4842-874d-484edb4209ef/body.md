<!-- aligned-structure:v1 -->

# Summary

Capture the recurring principles that define how `viewer-api` and the managed viewers expose and operate their shared public surface.

## Behavior Story

`viewer-api` keeps one canonical recurring-principles spec so the managed viewers share the same expectations for public surface ownership and lifecycle management rather than drifting into per-viewer conventions.

## Provided Surface Contracts

- The `viewer-api` recurring-principles spec is the canonical authority for shared viewer-surface behavior.
- Each principle is maintained as its own section so generated guidance can reference it independently.
- The current principles cover viewer public surface contracts and the `viewer-ctl` lifecycle boundary.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- [f147eb0e Migrate recurring spec principles to canonical rule entries via spec sync-generated](.ticket/tickets/f147eb0e-c758-459b-a956-a1162c3e1af6/ticket.toml)
- [a5fe4c58 Adopt rule targets for generated spec artifacts](memory-api/.ticket/tickets/a5fe4c58-f59c-4d97-8ee6-3447724b5fac/ticket.toml)

## Background Knowledge References

- `spec-api/generated-documents` (`1cf68c36-7f64-4d81-b553-1947b978fbe3` in memory-viewers/memory-api)
- `memory-api/recurring-principles` (`f9c32554-9884-41c4-8b5b-d1d32b37e341` in memory-viewers/memory-api)
- `context-engine/recurring-principles` (`954d9807-f357-41e5-9fd4-b1da39e0933d` at the context-engine root)

## Legacy Content (Preserved)

# viewer-api recurring principles

This spec captures the cross-cutting design principles that recur across `viewer-api` and the managed viewers (`doc-viewer`, `log-viewer`, `ticket-viewer`, `spec-viewer`). They are the canonical authority for how the shared viewer surface is expected to behave.

Each principle is its own section so a `rule scan` materialises one canonical entry per principle and downstream agent guidance can reference them individually.

## Sections

- `public-surface-contracts` — Every viewer's HTTP, GraphQL, and frontend surface is a versioned public contract owned by `viewer-api`.
- `viewer-ctl-lifecycle-boundary` — `viewer-ctl` is the only supported way to start, stop, restart, or prepare a viewer.

## Related tickets

- [f147eb0e Migrate recurring spec principles to canonical rule entries via spec sync-generated](.ticket/tickets/f147eb0e-c758-459b-a956-a1162c3e1af6/ticket.toml)
- [a5fe4c58 Adopt rule targets for generated spec artifacts](memory-api/.ticket/tickets/a5fe4c58-f59c-4d97-8ee6-3447724b5fac/ticket.toml)

## Related specs

- `spec-api/generated-documents` (`1cf68c36-7f64-4d81-b553-1947b978fbe3` in memory-viewers/memory-api)
- `memory-api/recurring-principles` (`f9c32554-9884-41c4-8b5b-d1d32b37e341` in memory-viewers/memory-api)
- `context-engine/recurring-principles` (`954d9807-f357-41e5-9fd4-b1da39e0933d` at the context-engine root)
