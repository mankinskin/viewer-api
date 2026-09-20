# Goal

Introduce multiple graph node detail levels so zoomed-out views stay legible and zoomed-in views can show rich ticket content.

# Scope

- define several node detail tiers, from minimal dot or particle forms up through compact and full ticket-card rendering
- choose detail levels from zoom, focus state, or visibility budget
- trim text length and auxiliary metadata aggressively at lower detail levels
- allow focused nodes to stay richer than distant nodes when the graph is dense

# Acceptance

- the graph can render nodes at several distinct detail levels instead of a single fixed card presentation
- zoomed-out graphs avoid unreadable text walls by falling back to minimal representations
- zooming in restores progressively richer ticket content
- focus or hover state can keep the active node more informative than distant nodes at the same zoom level

# Status

## Implementation

- shared node detail tiers now live in `viewer-api/viewer-api/frontend/dioxus/src/graph3d/data.rs` and are selected from projected pixel scale plus focus state
- the shared DOM renderer in `viewer-api/viewer-api/frontend/dioxus/src/graph3d/render.rs` now applies tier-specific card dimensions and toggles rich, compact, and minimal node subviews per frame
- ticket-viewer graph cards in `ticket-viewer/frontend/dioxus/src/graph3d.rs` now expose stable `data-node-id` selectors and tier-specific rich, compact, and minimal content blocks for browser assertions and renderer-driven LOD changes
- host-side viewer-api Dioxus tests now compile through `viewer-api/viewer-api/frontend/dioxus/Cargo.toml` by enabling the minimal non-wasm `web-sys` features needed by `dioxus-web` during `cargo test`

## Validation

- passed `cargo test --manifest-path viewer-api/viewer-api/frontend/dioxus/Cargo.toml node_detail_`
- passed `viewer-ctl stop ticket-viewer && cd memory-viewers/ticket-viewer/frontend/dioxus && npm run test:e2e:release -- graph-detail-sidebar.spec.ts`

## Documentation

- updated [322ba030 ticket](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/.ticket/tickets/322ba030-160c-41d3-8a12-42936ae92858/ticket.toml) status summary and the shared tracker spec at [8c4d51ef body](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/context-engine/memory-viewers/.spec/specs/8c4d51ef-c0b6-437d-bc14-672b0802cef2/body.md)
