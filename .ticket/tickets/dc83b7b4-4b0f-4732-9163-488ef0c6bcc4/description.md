## Problem

The action buttons in the ticket-viewer header (`🎨 Theme settings`, `☑ Batch`, `+ New Ticket`) are styled inline in `routes.rs` with three different background tokens (`var(--bg-secondary)`, `var(--accent-blue)`, conditional accent for batch toggle). They look mismatched in the screenshot — one pink-tinted (theme), one white (batch), one solid blue (new ticket).

The state-filter chips (`All` / `new` / `ready` / …) in the sidebar also drift visually because the `All` chip uses an accent fill while inactive chips use `var(--bg-secondary)` which renders almost-transparent on the WebGPU smoke shader.

## Acceptance criteria

1. Add a new `IconButton` component in `viewer-api-dioxus::components::buttons` with three variants:
   - `Ghost` — fully transparent background, hover reveals `var(--bg-hover)`, border on focus only.
   - `Subtle` — `var(--bg-secondary)` with subtle border, used for toggles in their off-state.
   - `Primary` — `var(--accent-blue)` solid, white text, used sparingly for the single primary action.
2. Replace the three header buttons in `routes.rs` to use `IconButton` (`Ghost` for theme, `Subtle` toggleable for Batch, `Primary` for `+ New Ticket`).
3. Replace the inline chip buttons in `TicketTree` filter row with `Chip` (`viewer-api-dioxus::components::chip` already exists in CSS — see `public/css/chip.css`).
4. All buttons inherit theme tokens — no inline hex anywhere.
5. Active/hover/focus states meet WCAG AA contrast in all four built-in themes.
6. Visual regression: Playwright snapshot under PAPER + DARK themes for the header.

## Implementation notes

- `buttons.css` already defines `.btn`, `.btn-primary`, `.btn-secondary` — reuse these classes; add `.btn-ghost` and `.btn-icon` if missing.
- Pre-existing `chip.css` defines `.chip` and `.chip--active`; the Rust `Chip` wrapper component may need to be created.

## Parent

Epic `4e2b2b0b-9f56-4786-991c-8f10e653f4c3`. Depends on `f00204fc-f33f-4cd6-9b5f-395071f4e118`.
