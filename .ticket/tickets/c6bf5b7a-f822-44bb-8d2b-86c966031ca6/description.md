The shared Graph3D edge overlay in `viewer-api/viewer-api/frontend/dioxus/src/graph3d/mod.rs` renders directed-edge arrow markers that are too small to read comfortably in ticket-viewer and other viewer-api graph consumers.

Acceptance criteria:
1. Shared Graph3D directed edge arrow tips are visibly larger in the viewer-api graph view and ticket-viewer graph view.
2. The change is made in the shared viewer-api Graph3D renderer rather than only in a tool-local consumer.
3. Marker sizing remains aligned with the existing edge stroke and does not clip or detach from the line endpoint.
4. Relevant Graph3D spec/docs mention the larger directed-edge marker treatment.
5. Validation covers at least a focused compile/build check for the shared frontend package.
