# Interview: Mobile Responsive Layout

**Date:** 2026-04-08
**Applies to:** `b3f9878d` (Layout components), `2405a83e` (CSS), `44d22e8f` (ticket-viewer scaffold)

## Question

What's the minimum viable width? Should the Dioxus port include a mobile-responsive layout (collapsible sidebar, stacked panels), or desktop-only?

## Answer

**Yes, mobile responsive layout is minimally implemented in viewer-api and log-viewer, doc-viewer, etc. We require it and want to improve it.**

## Implications

- Responsive layout is a requirement, not optional
- Study existing responsive implementations in viewer-api, log-viewer, doc-viewer for baseline patterns
- Tri-pane layout must collapse: sidebar → collapsible drawer, panels → stacked vertically on narrow screens
- CSS media queries or container queries for breakpoints
- Touch-friendly: larger tap targets, swipe gestures for sidebar toggle
- ResizeHandle behavior on mobile: likely hidden, replaced by full-width stacking
- Tree navigation on mobile: full-screen overlay or slide-in drawer
- Improve upon existing implementations — this is an upgrade opportunity, not just a port
- Test at common mobile widths (320px, 375px, 414px) and tablet (768px, 1024px)
