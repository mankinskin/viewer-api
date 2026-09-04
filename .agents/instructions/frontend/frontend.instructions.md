---
description: "Use when editing frontend packages or generated TypeScript types. Covers Preact/Vite conventions, shared viewer-api frontend package usage, type generation, and browser-test expectations."
---


## Stack and Shared Dependencies

- Frontends in this repository use Preact + Vite + TypeScript.
- Prefer `@preact/signals` patterns for shared reactive state where already used.
- Reuse shared package primitives from `@context-engine/viewer-api-frontend` before adding tool-local duplicates.

## Shared Frontend Package Usage

- Shared TS UI/style primitives currently live under `workflow-tools/log/crates/log-viewer/frontend/viewer-api-frontend/`.
- Shared Dioxus viewer primitives and test helpers live under `viewer-api/viewer-api/frontend/dioxus/`.
- Place cross-viewer reusable components in the shared package, not copied per tool.
- Keep tool-specific behavior in each tool frontend and shared behavior in viewer-api frontend.

## TypeScript Type Generation

- Do not hand-edit generated files under `packages/context-types/src/generated/`.
- Generate types from Rust `ts-rs` exports using `viewer-ctl gen-types` (or `cargo make gen-types`).
- PowerShell variant also available at `scripts/generate-types.ps1`.
- For context-api type exports, maintain feature-gated generation patterns (`ts-gen`) where required.

## Component and Code Organization

- Keep component logic modular and colocated by feature.
- Keep API client interactions separate from presentation components.
- Keep state/store logic explicit and testable.

## Frontend Validation

For frontend-impacting changes run, at minimum:

1. Lint and typecheck for each affected frontend package.
2. Nearest unit/component tests where available (for example Vitest in log-viewer frontend).
3. Browser end-to-end checks where available, per [AGENTS.md](../../../AGENTS.md#quality-gates)'s MCP-Playwright-first rule (fall back to repo-local Playwright flows only when the MCP surface is unavailable or insufficient, for example ticket-viewer frontend).
4. Contract checks for changed API integration paths.

## UX Validation Expectations

- Verify loading, empty, success, and error states for changed flows.
- Verify keyboard navigation and focus behavior for changed interactions.
- Verify responsive rendering on desktop, tablet, and mobile-width layouts.
- Keep user-facing error text actionable and recovery-oriented.
