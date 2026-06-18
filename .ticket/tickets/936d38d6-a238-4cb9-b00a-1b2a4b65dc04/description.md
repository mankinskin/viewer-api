Track the remaining Dioxus adoption work needed after the shared doc-viewer-inspired primitives landed in `viewer-api-dioxus`, so current viewers reuse the shared shells and stores instead of carrying bespoke implementations.

## Motivation

The original phase plan under this tracker succeeded in landing most of the shared foundations, but the tracker description is now stale: several items it still marks as missing already exist in `viewer-api-dioxus`, while the remaining work is mostly consumer adoption. Refreshing the tracker keeps follow-up tickets aligned with the real gaps instead of reopening completed primitives.

## Scope

Shared crate: `viewer-api/viewer-api/frontend/dioxus/`
Current Dioxus consumers in scope: `memory-viewers/spec-viewer/frontend/dioxus/`, `memory-viewers/ticket-viewer/frontend/dioxus/`, and `tools/viewer/doc-viewer/frontend/dioxus/`
Out of scope: the legacy Preact doc-viewer frontend, unrelated viewer layout rewrites, and speculative phases that are not backed by a confirmed current consumer gap.

## Current status

| Area | Current status | Evidence |
|---|---|---|
| Shared page header shell | Implemented; adoption tracked | `bb1c32f5-5275-4e4f-85ae-a0fba09c522a` |
| Shared explorer shell | Implemented; adoption tracked | `763f8c13-a4bd-47af-8894-3e95a63fde8d` |
| Rich tree tooltips | Implemented; adoption tracked | `21a2e8f4-4bd8-4436-be52-c2c4a07bb692` |
| TabsStore / PathCodec / Prefetcher | Implemented in shared crate | `da16dada-e245-4fdd-868a-c3691e6c351a` |
| Breadcrumbs / Overlay / MetaHeader / CardGrid | Implemented in shared crate | earlier phase tickets already landed |
| FilterPanel shell | Implemented in shared crate | `b4127011-4e08-47bc-ac73-3d3761f29587` |
| HeaderActions / mobile-sidebar audit / `tooltip_render` | Implemented in shared crate | `8bf5edd2-4fe6-4580-ac87-73843f0206f0` |

## Remaining confirmed gaps

1. `tools/viewer/doc-viewer/frontend/dioxus/src/app.rs` still hand-rolls tab state with `Signal<Vec<OpenArtifactTab>>` plus `Signal<Option<String>>` instead of consuming the shared `TabsStore<OpenArtifactTab>`.
2. Demo-viewer showcase tickets for shared tab/store primitives should stay linked to the next adoption slice so the generic contract remains demonstrable as consumers converge.

## Linked work

- Current child/adoption tickets:
	- `bb1c32f5-5275-4e4f-85ae-a0fba09c522a` — shared page header shell
	- `763f8c13-a4bd-47af-8894-3e95a63fde8d` — shared explorer shell
	- `21a2e8f4-4bd8-4436-be52-c2c4a07bb692` — rich tree tooltip adoption
	- `4d9293ab-b7a8-4113-b80a-bfe39297bad2` — shared TabsStore adoption in Dioxus doc-viewer
- Demo-viewer showcase links for the next tab-state slice:
	- `0eef1873-0626-4a87-93bc-51d182808e16` — feature page: tab bar
	- `1efec195-f8b4-4571-b073-806cac0b66ce` — feature page: store primitives

## Acceptance criteria

- The tracker only stays open for confirmed remaining adoption work, not for primitives that already landed.
- Each linked child ticket completes its scope with the relevant focused validation for the affected consumer and shared crate.
- Shared demo-viewer tickets remain linked to adoption slices when they are the intended generic showcase for the same shared contract.

## Risks

- Doc-viewer tab-state migration must preserve existing auto-open, close-neighbour, and JSON fetch flows while switching to the shared store contract.
- URL/path-sync and prefetch follow-ups should not be opened until the tab-state adoption clarifies the real consumer surface; otherwise the tracker will drift back into speculative work.
