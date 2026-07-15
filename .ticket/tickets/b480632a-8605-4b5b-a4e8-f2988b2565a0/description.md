# Goal

Replace ad-hoc browser console calls in viewer-api Dioxus/WASM code with structured tracing that supports levels, fields, spans, runtime filtering, console diagnostics, and the separate persisted sink.

# Current implementation evidence

The repository contains a tracing subscriber setup, runtime filter controls, tracing-wasm console/timeline integration, and shared Playwright checks that look for the startup subscriber record. Remaining work is to audit migration completeness, validate structured field/span behavior across viewers, and record release-browser evidence.

# Scope

- Audit viewer-api, ticket-viewer, and spec-viewer WASM code for direct diagnostic console calls.
- Use tracing macros and stable targets/fields for overlay bootstrap, Graph3D initialization, API operations, and errors.
- Keep per-frame spans behind the profiling feature so normal release logging remains bounded.
- Validate default INFO, runtime DEBUG override, and `log=off` behavior.
- Preserve browser console output while enabling the file sink owned by `8f349d96`.

# Acceptance criteria

- [ ] Diagnostic direct console calls are removed or explicitly justified as non-tracing browser integration.
- [ ] DevTools records expose level, target, structured fields, and operation spans.
- [ ] Default level is INFO; DEBUG and OFF controls work through documented query/localStorage settings.
- [ ] Per-frame trace volume is bounded in ordinary release builds.
- [ ] Ticket-viewer and spec-viewer pass the same shared tracing-console suite.
- [ ] WgpuOverlay and Graph3D startup/navigation remain functional in release-browser checks.
- [ ] Validation executions and correlated logs are recorded.

# Implementation steps

1. Audit remaining direct console usage.
2. Normalize stable tracing targets and fields.
3. Verify runtime filters and profiling feature boundaries.
4. Run shared release-browser tracing checks.
5. Record evidence and hand off persistence-specific behavior to `8f349d96`/`9202bc21`.