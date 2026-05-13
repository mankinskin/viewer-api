# Objective

Design a viewer-wide keyboard interaction model for the Dioxus viewer stack without bundling it into the immediate explorer fixes.

The user explicitly wants this tracked as a separate follow-up/epic. Initial examples called out in discussion:

- WASD movement in the camera view
- Enter to open tickets
- Tab-based switching between ticket detail tabs
- consistent keyboard behavior across tree, search, and graph surfaces

## Current State

Research shows the keyboard story is fragmented today:

- quick-search has a local open/close shortcut but no result-list navigation model
- the sidebar explorer lacks a normal arrow/enter flow
- the graph interaction path appears mouse-centric
- the detail tab bar is button-click driven, with no dedicated tab-switch shortcut model

## Deliverables

1. A documented shortcut map that defines scope, precedence, and focus ownership for global versus local shortcuts.
2. A conflict policy for text inputs, editors, modal overlays, tree views, and graph canvases.
3. A phase breakdown that separates immediate low-risk flows from higher-risk graph/camera controls.
4. Follow-up implementation tickets for the approved phases, including browser verification requirements.

## Key Design Questions

- When should a shortcut be global versus only active inside a focused surface?
- How should the graph canvas acquire and release keyboard focus before enabling camera movement?
- Should `Tab`/`Shift+Tab` cycle detail tabs only, or participate in a broader page focus order?
- Which shortcut-help surface will make the model discoverable?

## Suggested First Breakdown

1. Phase 1: local ticket-list navigation (sidebar + quick-search)
2. Phase 2: detail-panel tab switching and focused action shortcuts
3. Phase 3: graph/camera keyboard controls, including WASD gating and escape hatches

## Out of Scope

This ticket does not implement the concrete explorer fixes directly. Those are tracked separately so they can ship without waiting for the full keyboard model review.
