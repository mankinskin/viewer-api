Extract a reusable Dioxus page-header shell in viewer-api-dioxus so viewer routes stop composing ad-hoc header behavior inline.

## Motivation

The current code already has Header and HeaderActions, but the route-level composition is inconsistent:

- spec-viewer root wires Home, filters, and Theme through HeaderActions, while detail and graph pages use manual Links/back buttons and only partial shared actions.
- ticket-viewer keeps a one-off gear button and bypasses HeaderActions entirely.
- doc-viewer Dioxus lacks the theme settings affordance that already exists in the TS doc-viewer header.

The missing shared piece is not another icon button. It is a higher-level page-header shell or helper contract that standardizes how viewers compose shared actions and route affordances on top of the existing primitives.

## Scope

- Build on top of Header and HeaderActions instead of replacing them.
- Define a reusable composition for common viewer actions: Home, Theme settings, Filter toggle, Refresh, and optional route links/back affordances.
- Keep extension points for viewer-specific extras without requiring inline style-heavy button rows in every consumer.
- Make the contract suitable for spec-viewer routes, ticket-viewer list view, and doc-viewer Dioxus.

## Acceptance criteria

- viewer-api-dioxus exposes a reusable page-header shell or equivalent helper contract for common viewer header composition.
- The contract covers Home and Theme settings affordances, plus optional filter/refresh hooks and route-level extras.
- The demo-viewer layout and theme-settings showcase work can consume or demonstrate the shared header contract.
- Browser and Playwright coverage can validate the shared actions instead of each viewer inventing incompatible selectors and labels.
