---
description: "Use when editing frontend packages or generated TypeScript types. Covers Preact/Vite conventions, shared viewer-api frontend package usage, type generation, and browser-test expectations."
applyTo: "**/frontend/**,packages/context-types/**"
---

# Frontend Guidance

## Stack and Shared Dependencies

- Frontends in this repository use Preact + Vite + TypeScript.
- Prefer `@preact/signals` patterns for shared reactive state where already used.
- Reuse shared package primitives from `@context-engine/viewer-api-frontend` before adding tool-local duplicates.

## Shared Frontend Package Usage

- Shared UI and style primitives live under `tools/viewer/viewer-api/frontend/ts/`.
- Place cross-viewer reusable components in the shared package, not copied per tool.
- Keep tool-specific behavior in each tool frontend and shared behavior in viewer-api frontend.

## TypeScript Type Generation

- Do not hand-edit generated files under `packages/context-types/src/generated/`.
- Generate types from Rust `ts-rs` exports using repository scripts.
- Preferred regeneration command: `bash scripts/generate-types.sh` (PowerShell variant exists at `scripts/generate-types.ps1`).
- For context-api type exports, maintain feature-gated generation patterns (`ts-gen`) where required.

## Component and Code Organization

- Keep component logic modular and colocated by feature.
- Keep API client interactions separate from presentation components.
- Keep state/store logic explicit and testable.

## Frontend Validation

For frontend-impacting changes run, at minimum:

1. Lint and typecheck for each affected frontend package.
2. Nearest unit/component tests where available (for example Vitest in log-viewer frontend).
3. Browser end-to-end checks where available (for example Playwright flows in ticket-viewer frontend).
4. Contract checks for changed API integration paths.

## UX Validation Expectations

- Verify loading, empty, success, and error states for changed flows.
- Verify keyboard navigation and focus behavior for changed interactions.
- Verify responsive rendering on desktop, tablet, and mobile-width layouts.
- Keep user-facing error text actionable and recovery-oriented.
