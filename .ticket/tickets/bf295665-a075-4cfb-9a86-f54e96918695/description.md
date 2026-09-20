Integrate the four graph improvements into log-viewer:

1. **Selection focus/deselection**: Update `app.rs` to use Graph3D with `on_deselect` handler, implement outside-click clearing for hypergraph visualization
2. **Property-based rendering tiers**: Ensure log graph nodes use the 5-level LOD system (Point/Sphere → Icon → Label → Compact → Rich) with hover promotion
3. **Panel-aware framing**: Configure viewport insets to keep log graph nodes behind UI panels in log-viewer layout
4. **2D mode/keyframing**: Add Fixed2D camera mode support to log-viewer graph, enable presentation keyframing for algorithm visualization

**Files to update:**
- `tools/viewer/log-viewer/frontend/dioxus/src/app.rs`
- Any log-viewer specific graph integration code

**Acceptance Criteria:**
1. log-viewer hypergraph visualization uses updated Graph3D component with all four improvements
2. Browser verification confirms:
   - Clicking outside log graph nodes clears selection
   - Node rendering detail changes with distance and hover state
   - Graph nodes stay behind log detail panels
   - 2D mode switching works in log-viewer context
3. Playwright E2E coverage for log-viewer graph interactions
4. No regression in existing log-viewer graph functionality
5. GraphOpEvent replay visualization works with new rendering tiers