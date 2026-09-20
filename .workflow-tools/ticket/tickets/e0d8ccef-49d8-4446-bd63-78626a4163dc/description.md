W6. The viewer-api 3D graph view needs optimization to efficiently provide the implemented features and to allow future extensibility in different contexts.

Requirements:
- Optimize the renderer for the implemented feature set (focus, property-based LOD, multi-level node detail).
- Efficiently render bounded neighbourhoods sourced from a cached graph (pairs with W2).
- Expose documented extension points so the renderer is reusable by rule/audit/demo viewers.

Relates to f9e9aaae (property-based node tiers), 322ba030 (multi-level node detail), b3d250d5 (WgpuOverlay moved to viewer-api). Spec 4f14356f.