# Shared Page Header Shell

The layout surface includes a reusable page-header shell for Dioxus viewers.

## Goals

- Compose the existing `Header` structure and `HeaderActions` behavior into one reusable contract.
- Cover the common viewer actions: `Home`, `Theme settings`, optional `Refresh`, optional filter toggle, and optional clear-filters affordance.
- Allow route-specific extras without forcing each viewer to rebuild the header row manually.

## Contract

The shared page-header shell:

- renders an optional leading control area for route-level affordances such as sidebar toggles or back buttons,
- renders optional title metadata (`icon`, `title`, `subtitle`) when the route uses a standard viewer title block,
- renders optional left-side extra content for route-specific UI such as breadcrumbs,
- renders optional right-side prefix/suffix content around the shared action row,
- wires shared actions through the same labels and semantics as `HeaderActions` so browser tests can target one contract across viewers.

## Intended Consumers

- `spec-viewer` list, detail, and graph routes,
- `ticket-viewer` list route,
- `doc-viewer` Dioxus header.

## Acceptance Behavior

- A viewer can adopt the page-header shell without reimplementing a custom settings button row.
- Home and Theme settings affordances use shared labels and remain available wherever the route opts in.
- Route-level extras such as links, breadcrumbs, or back buttons remain possible without bypassing the shared action shell.

## Implementation Status

- Implemented in `viewer-api-dioxus` as `PageHeader`, composed from `Header` and `HeaderActions`.
- Adopted by `spec-viewer` list, detail, and graph routes.
- Adopted by `ticket-viewer` list route, including shared `Home` and `Theme settings` header actions.
- Adopted by the Dioxus `doc-viewer` header, using shared `Refresh` and `Theme settings` actions.

## Validation

- `cargo check --target wasm32-unknown-unknown -p viewer-api-dioxus`
- `cargo check --target wasm32-unknown-unknown -p spec-viewer-dioxus`
- `cargo check --target wasm32-unknown-unknown -p ticket-viewer-dioxus`
- `cargo check --manifest-path tools/viewer/doc-viewer/frontend/dioxus/Cargo.toml --target wasm32-unknown-unknown`
- `npm run test:e2e:release -- e2e-release/ticket-viewer.release.spec.ts -g "shared header actions render Home and Theme settings affordances|theme settings button opens and closes the theme settings panel"` in `memory-viewers/ticket-viewer/frontend/dioxus`
- Focused Chromium Playwright probe against managed `doc-viewer` at `1440x900`, validating `Refresh`, `Theme settings`, and the open/close theme panel flow after `viewer-ctl prepare doc-viewer`
