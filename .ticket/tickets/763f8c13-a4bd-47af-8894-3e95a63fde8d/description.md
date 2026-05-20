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

Implementation update:
- Added shared `ExplorerShell` and `SidebarSearch` primitives in `viewer-api-dioxus` so viewers can compose shared search, controls, status, and body chrome around either `FileTree` or raw `TreeView` content.
- Adopted the shared shell in `spec-viewer` `SpecTree` so the spec sidebar now shares the search and status layout around `FileTree` instead of keeping bespoke wrapper markup.
- Adopted the shared shell in `ticket-viewer` `TicketTree` while preserving ticket-specific row rendering, state chips, selection controls, and keyboard navigation.
- Kept the shell body generic so existing TreeView-based consumers, including the current Dioxus `doc-viewer` structure, can adopt the same shared chrome directly without changing row rendering.

Validation status:
- Passed: `cargo check --target wasm32-unknown-unknown -p viewer-api-dioxus`
- Passed: `cargo check --target wasm32-unknown-unknown -p spec-viewer-dioxus`
- Passed: `cargo check --target wasm32-unknown-unknown -p ticket-viewer-dioxus`
- Passed: focused Chromium probe against managed `spec-viewer` at `1440x900`, validating the browse route and shared sidebar search shell after `viewer-ctl prepare spec-viewer`.
- Passed: `npm run test:e2e:release -- e2e-release/sidebar-query-state-filter.spec.ts e2e-release/keyboard-navigation.spec.ts -g "sidebar explorer keeps active state filter when filter text is non-empty|sidebar filter keeps focus while arrows move the active ticket and Enter selects it"` in `memory-viewers/ticket-viewer/frontend/dioxus`.