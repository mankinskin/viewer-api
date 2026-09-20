<!-- aligned-structure:v1 -->

# Summary

The four graph improvements implemented in ticket-viewer need to be generalized to spec-viewer and log-viewer:

## Behavior Story

The four graph improvements implemented in ticket-viewer need to be generalized to spec-viewer and log-viewer:

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# Problem

The four graph improvements implemented in ticket-viewer need to be generalized to spec-viewer and log-viewer:

1. **Selection focus/deselection**: Outside-click clearing, focus neighborhood transparency
2. **Property-based rendering tiers**: 5-level LOD (Point/Sphere → Icon → Label → Compact → Rich) with hover promotion
3. **Panel-aware framing**: Z-index behind UI panels, viewport insets for panel overlap safety
4. **2D mode/keyframing**: Fixed2D camera mode, presentation keyframing

These improvements are currently only available in ticket-viewer, creating inconsistency across the memory-viewer ecosystem.

# Goals

- Provide consistent graph interaction patterns across all memory-viewers
- Enable spec-viewer and log-viewer to benefit from the same graph improvements
- Maintain viewer-specific customization while sharing core Graph3D functionality
- Update documentation and examples for Graph3D component adoption

# Requirements

## Core Graph3D Component Updates

The shared `viewer-api/frontend/dioxus/src/graph3d/` component already contains all four improvements. The work involves:

1. **Spec-viewer integration**: Update `spec_graph/page.rs` to use the updated Graph3D props
2. **Log-viewer integration**: Update `log-viewer/app.rs` to use the updated Graph3D props
3. **Documentation**: Update Graph3D component spec with generalized feature descriptions
4. **Examples**: Provide viewer-specific examples for each integration pattern

## Integration Patterns

### Spec-viewer Integration
- Update `spec_graph/page.rs` to pass `on_deselect` handler
- Configure viewport insets for spec preview panel overlap
- Enable Fixed2D camera mode for spec graph presentations
- Ensure property-based rendering tiers work with spec node data

### Log-viewer Integration  
- Update `app.rs` to use Graph3D for hypergraph visualization
- Pass `on_deselect` handler for log graph interactions
- Configure viewport insets for log detail panel overlap
- Enable Fixed2D camera mode for algorithm visualization
- Integrate GraphOpEvent replay with rendering tiers

## Validation Requirements

1. **Browser verification** for both viewers:
   - Outside-click deselection works
   - Property-based rendering tiers respond to distance and hover
   - Graph nodes stay behind UI panels
   - 2D mode switching works

2. **Playwright E2E coverage**:
   - Critical graph interactions in spec-viewer
   - Critical graph interactions in log-viewer
   - Cross-viewer consistency checks

3. **No regression** in existing functionality:
   - Spec-viewer graph navigation
   - Log-viewer hypergraph visualization
   - GraphOpEvent replay

# Implementation Plan

1. **Phase 1**: Update spec-viewer graph integration
2. **Phase 2**: Update log-viewer graph integration  
3. **Phase 3**: Update Graph3D component documentation
4. **Phase 4**: Add viewer-specific examples and test coverage
5. **Phase 5**: Browser verification and E2E testing

# Related Tickets

- [254ac30d Generalize graph improvements to spec-viewer and log-viewer](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/254ac30d-26c0-4bfe-8a66-b10ab9e4843a/ticket.toml)
- [88f87410 Integrate graph improvements into spec-viewer](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/88f87410-e0fa-4196-a461-805050670d08/ticket.toml)
- [bf295665 Integrate graph improvements into log-viewer](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/bf295665-a075-4cfb-9a86-f54e96918695/ticket.toml)

# Code References

- `viewer-api/viewer-api/frontend/dioxus/src/graph3d/` (shared component)
- `memory-viewers/spec-viewer/frontend/dioxus/src/components/spec_graph/page.rs`
- `tools/viewer/log-viewer/frontend/dioxus/src/app.rs`
- `memory-viewers/ticket-viewer/frontend/dioxus/src/graph3d.rs` (reference implementation)
