<!-- spec-api:file generated=true -->

<!-- spec-api:entry id=b2c53031-caef-4f2b-b911-a242a564b40b slug=viewer-api/recurring-principles/viewer-api-recurring-principles/l1 -->
# viewer-api recurring principles

This spec captures the cross-cutting design principles that recur across `viewer-api` and the managed viewers (`doc-viewer`, `log-viewer`, `ticket-viewer`, `spec-viewer`). They are the canonical authority for how the shared viewer surface is expected to behave.

<!-- spec-api:entry id=35ebb9ec-1962-46ca-a25d-988e42806632 slug=viewer-api/recurring-principles/viewer-api-recurring-principles/l5 -->
Each principle is its own section so a `rule scan` materialises one canonical entry per principle and downstream agent guidance can reference them individually.

<!-- spec-api:entry id=5d2accaf-c570-4235-a219-c61e1116e071 slug=viewer-api/recurring-principles/viewer-api-recurring-principles/sections/l7 -->
## Sections

- `public-surface-contracts` — Every viewer's HTTP, GraphQL, and frontend surface is a versioned public contract owned by `viewer-api`.
- `viewer-ctl-lifecycle-boundary` — `viewer-ctl` is the only supported way to start, stop, restart, or prepare a viewer.

<!-- spec-api:entry id=925ea219-ff23-45cf-9ced-de2a23c3bac5 slug=viewer-api/recurring-principles/viewer-api-recurring-principles/related-tickets/l12 -->
## Related tickets

Viewer-specific recurring-principles follow-up should be recorded only when the shared migration history at the context-engine root is not enough for a managed-viewer release or viewer-api contract change.

<!-- spec-api:entry id=6980beef-8d68-4a5b-901e-db262aaf8b2f slug=viewer-api/recurring-principles/viewer-api-recurring-principles/related-specs/l17 -->
## Related specs

Viewer-api uses the context-engine root recurring-principles spec as the cross-workspace contract anchor, and should mention neighboring viewer-specific specs here only when that extra traceability is needed.
