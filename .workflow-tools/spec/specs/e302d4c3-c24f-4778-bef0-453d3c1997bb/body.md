<!-- aligned-structure:v2 -->

# Motivation

The four managed viewers share browser, WASM, WebGPU, and tracing surfaces, but their release checks, browser launch options, and evidence are fragmented. This contract replaces unsupported claims of zero-copy, GC-free execution, nearly-native behavior, fixed FPS, or universal no-code-change scalability with repeatable browser evidence that identifies the workload, warm-up, metric, budget, environment class, and attached artifacts.

# Dependent expectation

If this spec is implemented, dependents can rely on one release-browser validation contract for ticket-viewer, spec-viewer, log-viewer, and doc-viewer: functional behavior is tested independently from software-WebGPU and hardware-WebGPU performance evidence; every performance result is tied to an environment manifest; and browser failures can be correlated to frontend logs, backend logs, traces, screenshots, and diagnostic artifacts. Native Tauri behavior is a separate, later contract and is not evidence for this browser/WASM batch.

## Measurable workloads and evidence

- **Cross-viewer release smoke:** cold start each viewer, wait for shared readiness, navigate its deterministic fixture, and capture console/browser diagnostics plus a screenshot. Functional success requires the readiness and viewer-specific assertions; it makes no GPU performance claim.
- **Graph3D interaction:** use the deterministic 12-node/18-edge demo, warm for 60 animation frames, then collect 300 consecutive frame durations while orbiting, panning, zooming, selecting, and exercising the SVG fallback. Report frame-time median, p95, p99, and the count above 50 ms; smoothed FPS is informational only.
- **WebGPU overlay:** warm the mounted overlay for 60 animation frames, collect 300 frames with the fixed stress fixture, and record the same percentiles and long-frame count. Missing WebGPU must run the documented fallback assertion, not silently pass a GPU profile.
- **Leak/soak:** repeat the deterministic navigation and mount/unmount workload for a declared duration and iteration count, attach browser diagnostics and any available memory/handle measurements, and report the measurement limitation when the browser cannot expose a metric.

A performance budget is an environment-qualified baseline artifact. For a workload to pass, its candidate median, p95, p99, and long-frame count must each be no worse than the accepted baseline by the declared tolerance in that artifact; a missing baseline, changed workload, missing manifest, or changed environment class blocks a performance claim rather than borrowing a result from another machine.

## Environment classes

- **functional:** release browser with GPU behavior not asserted; this is the fast compatibility lane.
- **software-webgpu:** named software adapter/backend (for example SwiftShader) with its launch flags and adapter evidence recorded. Results prove deterministic functional rendering only and must never be described as native-GPU evidence.
- **hardware-webgpu:** named physical adapter with browser version, driver/backend, operating system, viewport, device scale factor, and display metadata recorded. This is the only class eligible for hardware performance budgets.

Each environment manifest contains the command/profile name, browser and browser revision, OS, viewport, scale factor, adapter identity, graphics backend/driver where exposed, enabled and disabled features, and the full launch arguments. A required WebGPU capability that is absent produces an explicit blocked result with the probe output and fallback evidence.

## Experimental assumptions and non-goals

HTML-to-texture or browser-element capture is experimental. It must be feature-detected, retain a tested fallback, and is not a condition for a browser release pass. This spec does not promise zero-copy transfer, GC-free execution, nearly-native speed, fixed FPS, performance portability across adapters, or automatic scalability without code changes. Native Tauri/Appium validation, long-running hardware qualification, and CI retention policy are outside this first batch.

# Guards

- Required aggregate guard: [`val-viewer-first-batch`](.test/default/specs/val-viewer-first-batch.json). A passing execution is permitted only after all six linked tickets have completed their focused evidence.
- Required evidence supplied by the shared-harness ticket: release-browser conformance, readiness diagnostics, and all-viewer artifacts.
- Required evidence supplied by the GPU-profiles ticket: named profile probes, manifests, explicit blocked results, and environment-qualified baseline comparisons.
- Required evidence supplied by the log-sink ticket: correlated `x-session-id` frontend/backend log and trace evidence.
- Required evidence supplied by the UX-gates ticket: Axe, keyboard/focus, responsive viewport, screenshot baseline, and canvas-pixel evidence.
- Required evidence supplied by the result-integration design ticket: structured Playwright, wasm-pack, benchmark, retry, blocked, provenance, and artifact mappings into test-api.

# Positions

- `partial` — `Makefile.toml`: existing aggregate viewer commands omit ticket-viewer and retain obsolete Windows paths; it is not yet a canonical all-viewer runner.
- `partial` — `viewer-api/viewer-api/frontend/dioxus/e2e/shared/suites/graph3d-profiling-suite.ts`: the suite creates tracing evidence but does not enforce frame-time percentile or long-frame budgets.
- `partial` — `viewer-api/viewer-api/frontend/dioxus/tests/graph3d_bench.rs`: it only proves elapsed time is finite, not a regression budget.
- `partial` — `viewer-api/viewer-api/frontend/dioxus/src/tracing_setup/network_layer.rs`: browser tracing transport exists, but session correlation and bounded operational behavior remain unproven.
- `not-implemented` — canonical shared all-viewer release runner, named browser profiles, and environment-manifest writer.
- `not-implemented` — shared Axe, focus, responsive, visual-baseline, and canvas-pixel gates.
- `not-implemented` — native Tauri validation is intentionally deferred and has no status in this browser/WASM contract.

# Governing-rule requirement

This spec is introduced under PolicyRule `84fa9769-cff9-4d89-9068-88474584b4b3` (ticket/spec routing) and PolicyRule `397b0447-135e-4d35-ad05-bcc69047d2c0` (mandatory browser quality gates), rendered from [AGENTS.md](AGENTS.md). The pinned routing rule currently fails to resolve in the durable session; that stale reference is a session-maintenance blocker, not an exception to this contract.

# Traceability

## Tickets

- [956485ad Robust browser, observability, and performance validation strategy](memory-viewers/.ticket/tickets/956485ad-2e80-4a4c-b5ec-42bac2c7c295/ticket.toml)
- [7d951620 Define measurable browser and GPU validation contract](memory-viewers/.ticket/tickets/7d951620-76c5-4b24-90ce-e7d08d2dd188/ticket.toml)
- [93b8a331 Browser and TypeScript automated test integration strategy](memory-api/.ticket/tickets/93b8a331-da80-4fef-b13d-7f277cadb15f/ticket.toml)
- [6002e996 Canonical shared Playwright harness and all-viewer runner](memory-viewers/.ticket/tickets/6002e996-dcb8-4dad-a830-20346ce9d8cc/ticket.toml)
- [26a73130 Reproducible software and hardware browser lanes](memory-viewers/.ticket/tickets/26a73130-c631-4168-8030-fade31c5cf55/ticket.toml)
- [8f349d96 Ship WASM tracing logs to a server file sink](viewer-api/.ticket/tickets/8f349d96-a307-400b-a90e-3aceb2250166/ticket.toml)
- [40110a1a Accessibility, responsive, focus, and visual regression gates](memory-viewers/.ticket/tickets/40110a1a-40e8-4345-8407-577bd5f4d602/ticket.toml)

## Related specs

- [347b6f97 Prefer MCP Playwright/browser tools](.spec/specs/347b6f97-5ebf-46c6-a0e1-cc8afc600319/spec.toml)
- [b06c9df8 Structured tracing for WASM frontend](viewer-api/.spec/specs/b06c9df8-2866-433a-af73-ae9b1f4a0f0a/spec.toml)
- [479e226a WASM tracing file sink](viewer-api/.spec/specs/479e226a-b4ef-4e30-ade0-ebdabbf956ed/spec.toml)
- [4f14356f Graph3D](viewer-api/.spec/specs/4f14356f-c4bd-4554-be1e-35361de241da/spec.toml)
- [f153483c WebGPU overlay](viewer-api/.spec/specs/f153483c-f984-4564-94ac-36234b5cbe3f/spec.toml)

The acceptance evidence for this contract is a passing aggregate ValidationExecution for `val-viewer-first-batch`, plus the focused executable evidence linked by each child ticket.