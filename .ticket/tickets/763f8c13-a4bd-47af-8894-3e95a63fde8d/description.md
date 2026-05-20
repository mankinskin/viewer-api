Extract a reusable Dioxus explorer shell around FileTree so viewers stop duplicating sidebar search and tree-control chrome.

## Motivation

viewer-api-dioxus already provides FileTree with generic sort and filter controls, but the consumer apps still drift:

- spec-viewer adds its own inline search input above FileTree.
- ticket-viewer reimplements explorer search, state chips, and auxiliary actions with bespoke markup and inline styles, while keeping sorting internal to the ticket tree component.
- doc-viewer Dioxus drops to raw TreeView and loses the shared sort/filter/search affordances entirely.

The goal is to share the explorer chrome, not to force every viewer into the same row-rendering model.

## Scope

- Build on FileTree and shared layout/sidebar styling instead of replacing them.
- Standardize search field rendering, empty/loading/error chrome, and the placement of sort/filter controls around FileTree.
- Provide extension slots for viewer-specific controls so ticket-viewer can keep specialized actions without forking the entire shell.
- Keep ticket-specific row expansion and detail/file rendering out of scope.

## Acceptance criteria

- viewer-api-dioxus exposes a reusable explorer shell around FileTree or TreeView-based consumers.
- spec-viewer and doc-viewer Dioxus can adopt the shared search/tree chrome directly.
- ticket-viewer can reuse the shared explorer chrome while keeping ticket-specific row rendering behind its own component boundary.
- The demo-viewer tree-view showcase ticket can demonstrate the shared explorer contract rather than a one-off page implementation.
