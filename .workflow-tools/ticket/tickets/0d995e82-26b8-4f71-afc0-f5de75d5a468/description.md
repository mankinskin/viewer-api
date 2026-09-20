W5a. We need a minimal viewer-template that serves as the bootstrap for new viewers (rule-viewer, audit-viewer).

The template must implement:
- a basic entity tree explorer (left),
- a center main view with tabs,
- a right panel and a bottom panel,
- floating panels within the main view.

Deliverables: a new viewer-api/viewer-template spec (to be authored), the template crate/scaffold, and docs positioning it as the canonical new-viewer starting point. Builds on layout components spec (b3362691) and shared-shell work (763f8c13, bb1c32f5, d1e4ab96, 92964ada). Precedes W5b.