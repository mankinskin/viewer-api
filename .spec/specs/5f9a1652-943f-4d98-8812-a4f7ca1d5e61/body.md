<!-- aligned-structure:v1 -->

# Summary

Migrate `viewer-api` to the shared README schema and extend its generated child README surfaces with parent links back to the repo root.

## Behavior Story

Migrate `viewer-api` to the shared README schema and extend its generated child README surfaces with parent links back to the repo root.

## Provided Surface Contracts

- README surfaces in scope follow an explicit rule-backed contract instead of one-off rollout prose.
- Parent and child README navigation stays repo-internal and mechanically derivable.
- README completeness and rollout status are verified by mechanical validation rather than manual review.

## Required Validation

- Contract clause validation: The migrated spec names the intended README structure and navigation behavior as explicit contract properties.
- Contract clause validation: The migrated spec names the validation path that checks the README contract mechanically.
- Contract clause validation: The migrated spec records enough evidence to tell whether the README contract is satisfied or blocked.
- The authored spec body documents the README contract, scope boundaries, and navigation expectations for this migration slice.
- The authored spec body documents the mechanical validation path required to prove this migration slice.
- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# Summary

Migrate `viewer-api` to the shared README schema and extend its generated child README surfaces with parent links back to the repo root.

## Problem

`viewer-api` already generates its repo root and child READMEs, but it still uses a bespoke root target layout and its first-level child surfaces do not yet form a consistent parent-linked tree.

## Scope

This spec covers:

- adoption of the shared schema for `viewer-api/README.md`
- parent-linked generated README targets for `viewer-ctl`, `viewer-api`, and `viewer-api/frontend/dioxus`
- preservation of the existing frontend and lifecycle command-doc references

## Intended Behavior

- The root `viewer-api` README inherits the shared README schema.
- The generated child READMEs expose parent links to `viewer-api/README.md`.
- Optional viewer-specific sections like screenshots remain compatible with the shared schema.

## Assumptions To Prove

- `viewer-api` can adopt the shared schema without losing its screenshots or frontend-specific sections.
- Parent links can be added cleanly to the generated child README surfaces.
- The repo-local README tree remains independently generatable after the migration.

## Test Strategy

1. Migrate the root `viewer-api` README target to the shared schema.
2. Add one representative parent-linked child README target.
3. Extend the same pattern to the remaining in-scope child surfaces and re-run sync checks.

## Acceptance Criteria

- `viewer-api/README.md` uses the shared README schema.
- The in-scope generated child READMEs expose parent links to `viewer-api/README.md`.
- `sync-targets --check` passes from the `viewer-api` repo root.

## Traceability

- [d7d582c2 viewer-api rollout ticket](C:/Users/linus_behrbohm/git/SECOND_CHECKOUT/graph_app/.workflow-tools/ticket/tickets/d7d582c2-5734-4818-acf1-382f67bfdb89/ticket.toml)
