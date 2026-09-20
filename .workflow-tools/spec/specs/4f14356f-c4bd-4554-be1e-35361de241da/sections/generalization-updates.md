# Generalization Updates (June 2026)

The Graph3D component has been enhanced with four major improvements that need to be generalized across all memory-viewers:

## New Features

### 1. Selection Focus/Deselection
- **`on_deselect` event handler**: Added to `Graph3DProps` for outside-click deselection
- **Focus neighborhood transparency**: Nodes outside active focus context render with transparency
- **Outside-click clearing**: Clicking empty graph space clears active selection

### 2. Property-Based Rendering Tiers
- **5-level LOD system**: `NodeDetailTier` enum with variants:
  - `PointOrSphere`: Minimal glyph (camera-mode specific)
  - `Icon`: Small icon representation
  - `Label`: Icon + short text label
  - `Compact`: Reduced detail card
  - `Rich`: Full detail card
- **Hover promotion**: Hovered nodes advance one tier in detail
- **Camera-mode specific minimal glyphs**: Fixed2D uses flat points, Orbit3D uses spheres

### 3. Panel-Aware Framing
- **Z-index management**: Graph nodes rendered behind UI panels
- **Viewport insets**: Configurable insets to keep nodes visible behind panels
- **Edge overlay styling**: Special styling for edges that pass behind panels

### 4. 2D Mode/Keyframing
- **Fixed2D camera mode**: Added to `CameraMode` enum for top-down planar projection
- **Presentation keyframing**: Temporary node positioning for presentations
- **Camera projection switching**: Seamless transition between 3D and 2D modes

## Integration Requirements

### For Spec-viewer
- Update `spec_graph/page.rs` to pass `on_deselect` handler
- Configure viewport insets for spec preview panel overlap
- Enable Fixed2D camera mode for spec graph presentations
- Ensure property-based rendering tiers work with spec node data

### For Log-viewer
- Update `app.rs` to use Graph3D for hypergraph visualization
- Pass `on_deselect` handler for log graph interactions
- Configure viewport insets for log detail panel overlap
- Enable Fixed2D camera mode for algorithm visualization
- Integrate GraphOpEvent replay with rendering tiers

## Related Specifications
- [Generalize graph improvements across all memory-viewers](spec:bca2c4a5-b39e-4896-91f2-8453a1f4ff60)
- [ticket-viewer: graph focus, property-based rendering, and 2D presentation mode](spec:98b4f75d-3628-470d-a5cc-c91b6cc9811a)