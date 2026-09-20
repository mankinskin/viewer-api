Generalize the four graph improvements implemented in ticket-viewer to spec-viewer and log-viewer:

1. **Selection focus/deselection**: Outside-click clearing, focus neighborhood transparency
2. **Property-based rendering tiers**: 5-level LOD (Point/Sphere → Icon → Label → Compact → Rich) with hover promotion
3. **Panel-aware framing**: Z-index behind UI panels, viewport insets for panel overlap safety
4. **2D mode/keyframing**: Fixed2D camera mode, presentation keyframing

These improvements have been successfully implemented in ticket-viewer and need to be integrated into:
- spec-viewer (`memory-viewers/spec-viewer/frontend/dioxus/src/components/spec_graph/page.rs`)
- log-viewer (`tools/viewer/log-viewer/frontend/dioxus/src/app.rs`)

**Acceptance Criteria:**
1. spec-viewer graph page uses updated Graph3D component with all four improvements
2. log-viewer hypergraph visualization uses updated Graph3D component with all four improvements
3. Browser verification for both viewers confirms:
   - Outside-click deselection works
   - Property-based rendering tiers respond to distance and hover
   - Graph nodes stay behind UI panels
   - 2D mode switching works
4. Playwright E2E coverage for critical interactions in both viewers
5. Documentation updates for Graph3D component spec to reflect generalized features