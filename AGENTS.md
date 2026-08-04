# Agent Rules

Global working rules for this repository. Keep this file small and stable.

## Operating Principles

Viewer-api follows the root operating principles as its default contract. Use this local section only for principles about shared-viewer surfaces, install-ctl boundaries (the config-driven lifecycle manager now lives in the main repo at `tools/install/install-ctl/`, not in this submodule), or frontend/public-API ownership that are too specific for the workspace-wide rules.

## Discovery Protocol (Before Editing)

Use live sources first:

The canonical discovery protocol is owned at the context-engine root and mirrored through memory-viewers. Add viewer-api-specific discovery steps here only when the shared protocol is insufficient.

Use static references as support:

1. `CHEAT_SHEET.md` for type-level gotchas and common patterns.
2. crate `README.md` and `HIGH_LEVEL_GUIDE.md` for design context.
3. existing tests for usage examples and assertions.

## Task Routing

Viewer-api follows the root task-routing rules by default. Add local routing notes here only when shared-viewer frontend, install-ctl lifecycle (`tools/install/install-ctl/` in the main repo), or browser verification work needs a narrower execution path than the shared workflow already provides.

## Quality Gates

Viewer-api inherits the shared quality gates from the context-engine root. Reserve this local section for browser-facing checks such as screenshots, Playwright coverage, and shared-viewer contract validation that the generic root guidance does not spell out.

```rust
let _tracing = init_test_tracing!(&graph);
```

Add viewer-api-specific trailing quality-gate reminders here only when the shared quality-gate owner at the context-engine root is insufficient.

## Feedback Workflow

The canonical feedback workflow is owned at the context-engine root and mirrored through memory-viewers. Add viewer-api-specific feedback handling here only when the shared owner is insufficient.

## Escalation Rules

Escalate in viewer-api when a change crosses shared-viewer contracts, generated frontend artifacts, or multi-repo ownership boundaries. Otherwise, rely on the root escalation policy without repeating it here.

## Fallback Mode (When MCP Is Unavailable)

- Docs fallback: search/read local docs directly.
- Ticket fallback: use `ticket` CLI.
- Logs fallback: inspect files under `target/test-logs/` directly.
- Context fallback: use `tools/context-cli/` commands.

## Canonical Sources

- API patterns and gotchas: `CHEAT_SHEET.md`
- Ticket workflow details: `.agents/prompts/tickets.prompt.md`
- Swarm workflow details: `.agents/prompts/swarm-worker.prompt.md`
- Path-specific rules: `.agents/instructions/*.instructions.md`
