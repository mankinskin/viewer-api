Bring the Preact doc-viewer's UX patterns (breadcrumbs, modal overlay, document tabs store, category landing pages, filter panel, rich tooltips, prefetch cache, human-readable URL routing) into the shared `viewer-api-dioxus` crate so spec-viewer (and future Dioxus viewers) can reuse them.

## Motivation

doc-viewer (Preact/TS) has matured a number of cross-cutting UX widgets and stores that the Dioxus viewers (currently only spec-viewer) lack. Each Dioxus viewer reinvents these inline with ad-hoc styles. Porting them to the shared crate eliminates drift and unblocks richer Dioxus viewers.

## Scope

Shared crate: `tools/viewer/viewer-api/frontend/dioxus/`
Initial consumer: `tools/viewer/spec-viewer/frontend/dioxus/`
Out of scope: rewriting doc-viewer itself. Doc-viewer may later opt into the shared crate via TS bindings, tracked separately.

## Gap matrix (summary)

| # | Feature | Dioxus status |
|---|---|---|
| 1 | Multi-document tab store (TabsStore<T>) | Missing |
| 2 | Breadcrumbs widget | Missing |
| 3 | CategoryPage / Card / CardGrid | Missing |
| 4 | Filter panel + JQ query shell | Missing |
| 5 | Modal/overlay shell | Missing |
| 6 | Document meta header + Chip | Missing |
| 7 | Rich tree-node tooltip (Element-based) | TreeNode.tooltip is String only |
| 8 | Prefetch / LRU cache | Missing |
| 9 | Split content area | Use TabsStore |
| 10 | HeaderActions helper | Missing |
| 11 | PathCodec + tree expansion sync | UrlStateManager exists, no codec helper |
| 12 | Mobile sidebar plumbing | Audit needed |

## Phases

Subtickets implement the plan in dependency order:

- **Phase 1 — primitives** (Breadcrumbs, Overlay/Modal, MetaHeader+Chip, Card/CardGrid)
- **Phase 2 — state containers** (TabsStore, PathCodec/url_path, Prefetcher)
- **Phase 3 — extend existing widgets** (TreeNode.tooltip_render, layout audit, HeaderActions)
- **Phase 4 — filter panel** (UI shell + per-viewer query backend)
- **Phase 5 — adopt in spec-viewer**
- **Phase 6 — adopt in doc-viewer (optional, deferred)**

## Acceptance criteria

- Each phase ticket completes its scope with passing `cargo check -p viewer-api-dioxus --target wasm32-unknown-unknown` and `cargo check -p spec-viewer-dioxus --target wasm32-unknown-unknown`.
- spec-viewer visually exercises every new widget (browser verification per AGENTS.md).
- No regressions in existing Dioxus viewers.

## Risks

- TabsStore generic over `T` — Dioxus `Signal` quirks; may need `Rc<dyn Any>` or split inner/outer.
- Overlay + WGPU canvas pointer-event interaction.
- Filter-panel JQ backend: spec-viewer has no jq endpoint; needs server-side support or deferred scope.
- `tooltip_render: Element` lifetime — likely `Rc<dyn Fn() -> Element>`.
