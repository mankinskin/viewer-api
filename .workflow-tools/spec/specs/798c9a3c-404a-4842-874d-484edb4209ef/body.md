# viewer-api recurring principles

This spec captures the cross-cutting design principles that recur across `viewer-api` and the managed viewers (`doc-viewer`, `log-viewer`, `ticket-viewer`, `spec-viewer`). They are the canonical authority for how the shared viewer surface is expected to behave.

Each principle is its own section so a `rule scan` materialises one canonical entry per principle and downstream agent guidance can reference them individually.

## Sections

- `public-surface-contracts` — Every viewer's HTTP, GraphQL, and frontend surface is a versioned public contract owned by `viewer-api`.
- `viewer-ctl-lifecycle-boundary` — `viewer-ctl` is the only supported way to start, stop, restart, or prepare a viewer.

## Related tickets

Viewer-specific recurring-principles follow-up should be recorded only when the shared migration history at the context-engine root is not enough for a managed-viewer release or viewer-api contract change.

## Related specs

Viewer-api uses the context-engine root recurring-principles spec as the cross-workspace contract anchor, and should mention neighboring viewer-specific specs here only when that extra traceability is needed.
