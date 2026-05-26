<!-- rule-api:file generated=true -->

<!-- rule-api:entry id=b2c53031-caef-4f2b-b911-a242a564b40b slug=viewer-api/recurring-principles/viewer-api-recurring-principles/l1 -->
# viewer-api recurring principles

This spec captures the cross-cutting design principles that recur across `viewer-api` and the managed viewers (`doc-viewer`, `log-viewer`, `ticket-viewer`, `spec-viewer`). They are the canonical authority for how the shared viewer surface is expected to behave.

<!-- rule-api:entry id=35ebb9ec-1962-46ca-a25d-988e42806632 slug=viewer-api/recurring-principles/viewer-api-recurring-principles/l5 -->
Each principle is its own section so a `rule scan` materialises one canonical entry per principle and downstream agent guidance can reference them individually.

<!-- rule-api:entry id=5d2accaf-c570-4235-a219-c61e1116e071 slug=viewer-api/recurring-principles/viewer-api-recurring-principles/sections/l7 -->
## Sections

- `public-surface-contracts` — Every viewer's HTTP, GraphQL, and frontend surface is a versioned public contract owned by `viewer-api`.
- `viewer-ctl-lifecycle-boundary` — `viewer-ctl` is the only supported way to start, stop, restart, or prepare a viewer.

<!-- rule-api:entry id=925ea219-ff23-45cf-9ced-de2a23c3bac5 slug=viewer-api/recurring-principles/viewer-api-recurring-principles/related-tickets/l12 -->
## Related tickets

- [f147eb0e Migrate recurring spec principles to canonical rule entries via spec sync-generated](.ticket/tickets/f147eb0e-c758-459b-a956-a1162c3e1af6/ticket.toml)
- [a5fe4c58 Adopt rule targets for generated spec artifacts](memory-viewers/memory-api/.ticket/tickets/a5fe4c58-f59c-4d97-8ee6-3447724b5fac/ticket.toml)

<!-- rule-api:entry id=6980beef-8d68-4a5b-901e-db262aaf8b2f slug=viewer-api/recurring-principles/viewer-api-recurring-principles/related-specs/l17 -->
## Related specs

- `spec-api/generated-documents` (`1cf68c36-7f64-4d81-b553-1947b978fbe3` in memory-viewers/memory-api)
- `memory-api/recurring-principles` (`f9c32554-9884-41c4-8b5b-d1d32b37e341` in memory-viewers/memory-api)
- `context-engine/recurring-principles` (`954d9807-f357-41e5-9fd4-b1da39e0933d` at the context-engine root)
