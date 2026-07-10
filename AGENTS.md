<!-- rule-api:file generated=true -->

<!-- rule-api:entry id=176359c8-3364-4bf6-ba39-bee981d3abc5 slug=shared/agent-rules/l1 -->
# Agent Rules

Global working rules for this repository. Keep this file small and stable.

<!-- rule-api:entry id=a417ac53-f26b-467c-8461-a9db76fcd474 slug=shared/agent-rules/operating-principles/l5 -->
## Operating Principles

Viewer-api follows the root operating principles as its default contract. Use this local section only for principles about shared-viewer surfaces, viewer-ctl boundaries, or frontend/public-API ownership that are too specific for the workspace-wide rules.

<!-- rule-api:entry id=7f34807e-0342-4ed9-904d-0336d42d4f69 slug=shared/agent-rules/discovery-protocol-before-editing/l17 -->
## Discovery Protocol (Before Editing)

Use live sources first:

<!-- rule-api:entry id=115ff3b8-0253-428b-ba11-c23f79422d3a slug=shared/agent-rules/discovery-protocol-before-editing/l21 -->
The canonical discovery protocol is owned at the context-engine root and mirrored through memory-viewers. Add viewer-api-specific discovery steps here only when the shared protocol is insufficient.

<!-- rule-api:entry id=affc5c5a-5594-43b1-938d-61e4e8f23059 slug=shared/agent-rules/discovery-protocol-before-editing/l31 -->
Use static references as support:

<!-- rule-api:entry id=abdc5a30-a69d-41fe-8069-8049107bc0d7 slug=shared/agent-rules/discovery-protocol-before-editing/l33 -->
1. `CHEAT_SHEET.md` for type-level gotchas and common patterns.
2. crate `README.md` and `HIGH_LEVEL_GUIDE.md` for design context.
3. existing tests for usage examples and assertions.

<!-- rule-api:entry id=88bb04cb-fe6d-4935-8654-bef9406c5d82 slug=shared/agent-rules/task-routing/l37 -->
## Task Routing

Viewer-api follows the root task-routing rules by default. Add local routing notes here only when shared-viewer frontend, viewer-ctl lifecycle, or browser verification work needs a narrower execution path than the shared workflow already provides.

<!-- rule-api:entry id=cec6d88f-4d8f-4e65-8d1d-77be678b06c5 slug=shared/agent-rules/quality-gates/l46 -->
## Quality Gates

Viewer-api inherits the shared quality gates from the context-engine root. Reserve this local section for browser-facing checks such as screenshots, Playwright coverage, and shared-viewer contract validation that the generic root guidance does not spell out.

<!-- rule-api:entry id=2bdbe25d-faa0-4ff3-ae99-684929d9d5c4 slug=shared/agent-rules/quality-gates/l61 -->
```rust
let _tracing = init_test_tracing!(&graph);
```

<!-- rule-api:entry id=0183c0d9-0992-47af-9763-872c5037dbe3 slug=shared/agent-rules/quality-gates/l65 -->
Add viewer-api-specific trailing quality-gate reminders here only when the shared quality-gate owner at the context-engine root is insufficient.

<!-- rule-api:entry id=e8a0002c-d749-4cba-8f9d-cb451f7a5108 slug=shared/agent-rules/feedback-workflow/l70 -->
## Feedback Workflow

The canonical feedback workflow is owned at the context-engine root and mirrored through memory-viewers. Add viewer-api-specific feedback handling here only when the shared owner is insufficient.

<!-- rule-api:entry id=eaf1888b-8daf-4606-b036-aae4df30f164 slug=shared/agent-rules/escalation-rules/l80 -->
## Escalation Rules

Escalate in viewer-api when a change crosses shared-viewer contracts, generated frontend artifacts, or multi-repo ownership boundaries. Otherwise, rely on the root escalation policy without repeating it here.

<!-- rule-api:entry id=4aca810e-2378-4f35-b1fb-7e61f38fbeff slug=shared/agent-rules/fallback-mode-when-mcp-is-unavailable/l86 -->
## Fallback Mode (When MCP Is Unavailable)

- Docs fallback: search/read local docs directly.
- Ticket fallback: use `ticket` CLI.
- Logs fallback: inspect files under `target/test-logs/` directly.
- Context fallback: use `tools/context-cli/` commands.

<!-- rule-api:entry id=e87186d5-1b9c-4f5b-a4a8-41f471842057 slug=shared/agent-rules/canonical-sources/l93 -->
## Canonical Sources

- API patterns and gotchas: `CHEAT_SHEET.md`
- Ticket workflow details: `.agents/prompts/tickets.prompt.md`
- Swarm workflow details: `.agents/prompts/swarm-worker.prompt.md`
- Path-specific rules: `.agents/instructions/*.instructions.md`
