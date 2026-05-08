<!-- rule-api:file generated=true -->

<!-- rule-api:entry id=e613b3d4-78bd-4786-b1f7-8481055ea8b4 slug=shared/instructions/frontend/frontend-instructions/l1 -->
---
description: "Use when editing frontend packages or generated TypeScript types. Covers Preact/Vite conventions, shared viewer-api frontend package usage, type generation, and browser-test expectations."
applyTo: "**/frontend/**,packages/context-types/**"
---

<!-- rule-api:entry id=f28243fd-4f4c-410d-a55d-77873b0d1c60 slug=shared/instructions/frontend/frontend-guidance/stack-and-shared-dependencies/l8 -->
## Stack and Shared Dependencies

- Frontends in this repository use Preact + Vite + TypeScript.
- Prefer `@preact/signals` patterns for shared reactive state where already used.
- Reuse shared package primitives from `@context-engine/viewer-api-frontend` before adding tool-local duplicates.

<!-- rule-api:entry id=8f4e4254-c36d-4146-b2a6-f4cae61ad592 slug=shared/instructions/frontend/frontend-guidance/shared-frontend-package-usage/l14 -->
## Shared Frontend Package Usage

- Shared UI and style primitives live under `tools/viewer/viewer-api/frontend/ts/`.
- Place cross-viewer reusable components in the shared package, not copied per tool.
- Keep tool-specific behavior in each tool frontend and shared behavior in viewer-api frontend.

<!-- rule-api:entry id=618b7a5e-65e5-4781-b095-4231b85b573f slug=shared/instructions/frontend/frontend-guidance/typescript-type-generation/l20 -->
## TypeScript Type Generation

- Do not hand-edit generated files under `packages/context-types/src/generated/`.
- Generate types from Rust `ts-rs` exports using `viewer-ctl gen-types` (or `cargo make gen-types`).
- PowerShell variant also available at `scripts/generate-types.ps1`.
- For context-api type exports, maintain feature-gated generation patterns (`ts-gen`) where required.

<!-- rule-api:entry id=d9b47158-1f03-49f2-a0dc-239d5b2dfffd slug=shared/instructions/frontend/frontend-guidance/component-and-code-organization/l27 -->
## Component and Code Organization

- Keep component logic modular and colocated by feature.
- Keep API client interactions separate from presentation components.
- Keep state/store logic explicit and testable.

<!-- rule-api:entry id=3697b8d7-2208-4742-85cb-1f61eeb0536e slug=shared/instructions/frontend/frontend-guidance/frontend-validation/l33 -->
## Frontend Validation

For frontend-impacting changes run, at minimum:

<!-- rule-api:entry id=61d90f3e-3126-4250-9604-de69eeabf87f slug=shared/instructions/frontend/frontend-guidance/frontend-validation/l37 -->
1. Lint and typecheck for each affected frontend package.
2. Nearest unit/component tests where available (for example Vitest in log-viewer frontend).
3. Browser end-to-end checks where available (for example Playwright flows in ticket-viewer frontend).
4. Contract checks for changed API integration paths.

<!-- rule-api:entry id=bfe9240c-4b43-4724-bef0-9882e6d851af slug=shared/instructions/frontend/frontend-guidance/ux-validation-expectations/l42 -->
## UX Validation Expectations

- Verify loading, empty, success, and error states for changed flows.
- Verify keyboard navigation and focus behavior for changed interactions.
- Verify responsive rendering on desktop, tablet, and mobile-width layouts.
- Keep user-facing error text actionable and recovery-oriented.
