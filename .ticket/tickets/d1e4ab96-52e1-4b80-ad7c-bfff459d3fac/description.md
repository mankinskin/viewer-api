Converge the duplicated Dioxus viewer shell patterns across the current frontend implementations so each viewer stays thin and generic behavior lives in viewer-api.

## Motivation

The current viewers already share foundational primitives in viewer-api-dioxus, but the consumer apps keep re-implementing the same shell behavior in incompatible ways:

- spec-viewer root uses HeaderActions and FileTree, but detail and graph routes fall back to route-specific header composition.
- ticket-viewer ships a bespoke explorer header and a manual settings button instead of reusing shared viewer-api shell pieces.
- doc-viewer Dioxus uses raw TreeView and a custom header without the theme/settings behavior already present in the parallel TS frontend.

This tracker keeps the convergence work narrow: extract the generic shells in viewer-api, then tie the demo-viewer showcase pages to those shared surfaces.

## Deliverables

1. A shared page-header shell ticket that standardizes Home, Theme, Filter, Refresh, and route-level header composition on top of existing viewer-api primitives.
2. A shared explorer shell ticket that standardizes search, loading/empty/error chrome, and FileTree sort/filter controls without forcing viewer-specific row rendering.
3. The existing demo-viewer layout, tree-view, and theme-settings feature-page tickets are linked as showcase consumers of the shared shells.
4. The existing doc-viewer Dioxus port ticket is linked as a consumer path for the shared shells.

## Out of scope

- Rewriting ticket-viewer row rendering or batch actions into a generic tree node model.
- Rewriting the doc-viewer document tab/content model.
- Cross-store ticket rewiring in sibling ticket stores.

## Acceptance criteria

- Shared header and explorer shell tickets are created with concrete scope and linked into this tracker.
- Demo-viewer layout, tree-view, and theme-settings tickets are linked so the showcase work stays attached to the shared-shell effort.
- The doc-viewer port ticket is linked to consume the shared shells rather than drifting further.
