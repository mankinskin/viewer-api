W5b. A complete demo-viewer that exercises all components from all viewers in a single application with arbitrary data, and a root-level domain selection to switch between domains.

Requirements:
- Single app showcasing every shared viewer component with arbitrary/seed data.
- Root-level domain selector switching between domains.
- Reuse the viewer-template layout from W5a.

Depends on W5a (viewer-template). Folds in existing b779c650 (demo-viewer scaffold, ready). Root-level domain selection benefits from W7 (workspace/domain enumeration). Spec 4c3b62b4.