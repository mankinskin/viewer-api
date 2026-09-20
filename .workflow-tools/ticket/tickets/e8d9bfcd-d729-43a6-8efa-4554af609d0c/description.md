Update Graph3D component documentation and examples to reflect the four graph improvements and provide clear integration guidance:

**Documentation Updates:**
1. **Graph3D component spec** (`viewer-api/components/graph3d`):
   - Document new `on_deselect` event handler
   - Document property-based rendering tiers (5-level LOD system)
   - Document panel-aware framing and viewport insets
   - Document Fixed2D camera mode and keyframing

2. **Integration guide**:
   - Create step-by-step guide for integrating Graph3D into new viewers
   - Provide viewer-specific examples for ticket-viewer, spec-viewer, log-viewer
   - Document common integration patterns and best practices

3. **API reference**:
   - Update Graph3DProps documentation with new properties
   - Document NodeDetailTier enum and usage
   - Document CameraMode enum (including Fixed2D variant)

4. **Examples**:
   - Create minimal working examples for each viewer type
   - Provide code snippets for common use cases
   - Include troubleshooting guide for common integration issues

**Files to update:**
- `viewer-api/viewer-api/frontend/dioxus/README.md`
- `viewer-api/viewer-api/README.md`
- Graph3D component spec sections
- Any existing integration documentation

**Acceptance Criteria:**
1. Graph3D component spec fully documents all four improvements
2. Integration guide provides clear, actionable steps for viewer developers
3. API reference is complete and accurate
4. Examples work correctly and demonstrate best practices
5. Documentation is consistent across all viewer-api packages