# Plan: Viewer Tools Feature Implementation

**Date:** 2026-03-01
**Scope:** log-viewer, doc-viewer, viewer-api
**Interview:** `agents/interviews/20260301_VIEWER_TOOLS_FEATURE_PLAN.md`
**Status:** IN PROGRESS — Phases 1, 3 done; Phases 4-7 in parallel

---

## Objective

Implement 8 features across the viewer toolchain: extract shared infrastructure, integrate with doc-viewer, refactor event naming, add query path visualization, fix highlighting bugs, finish search path visualization, complete context-insert visualization, and document all events.

## Deferred (not in this plan)

- Fix context-insert bugs and finish context-read implementation (later)
- Context-read event emission implementation (design only in this plan)

---

## Phase 1: Extract Infrastructure to viewer-api ✅

**Goal:** Move duplicated code from log-viewer to viewer-api so both tools share it.

**Completed:** Phase 1A (JQ engine, source utils) and Phase 1B (theme system) extracted. MCP boilerplate (1.3), WebGPU effects (1.7), list components (1.8), HypergraphView (1.9) deferred — HypergraphView tightly coupled to log-viewer store signals.

### Phase 1A: Rust Backend Extraction

| Step | Description | Source | Target |
|------|-------------|--------|--------|
| 1.1 | Extract JQ query engine | `log-viewer/src/query.rs`, `doc-viewer/src/query.rs` | `viewer-api/src/query.rs` |
| 1.2 | Extract source file serving handler | `log-viewer/src/source.rs` | `viewer-api/src/source.rs` |
| 1.3 | Extract MCP server boilerplate | Common patterns in `log-viewer/src/mcp_server.rs`, `doc-viewer/src/mcp/` | `viewer-api/src/mcp.rs` |
| 1.4 | Update log-viewer to use shared modules | `log-viewer/src/main.rs`, `router.rs` | Import from viewer-api |
| 1.5 | Update doc-viewer to use shared modules | `doc-viewer/src/main.rs`, `http.rs` | Import from viewer-api |

**Files affected:**
- `tools/viewer-api/src/lib.rs` — add module declarations
- `tools/viewer-api/src/query.rs` — new shared JQ engine
- `tools/viewer-api/src/source.rs` — new shared source serving
- `tools/viewer-api/src/mcp.rs` — new shared MCP boilerplate
- `tools/viewer-api/Cargo.toml` — add `jaq-*` dependencies
- `tools/log-viewer/src/query.rs` — thin wrapper or re-export
- `tools/log-viewer/src/source.rs` — thin wrapper or re-export
- `tools/doc-viewer/src/query.rs` — thin wrapper or re-export

### Phase 1B: Frontend Extraction

| Step | Description | Source | Target |
|------|-------------|--------|--------|
| 1.6 | Extract theme system | `log-viewer/frontend/src/store/theme.ts`, CSS variables | `viewer-api/frontend/src/styles/`, `src/theme.ts` |
| 1.7 | Extract WebGPU effects system | `log-viewer/frontend/src/effects/` (palette, shaders) | `viewer-api/frontend/src/effects/` |
| 1.8 | Extract list components | Log entry list patterns from log-viewer | `viewer-api/frontend/src/components/` |
| 1.9 | Extract HypergraphView core | `log-viewer/frontend/src/components/HypergraphView/` | `viewer-api/frontend/src/components/HypergraphView/` |
| 1.10 | Update log-viewer to import from viewer-api | All extracted component imports | Use `@context-engine/viewer-api-frontend` |
| 1.11 | Update viewer-api `package.json` and `index.ts` | Barrel exports | Add new component exports |

**Files affected:**
- `tools/viewer-api/frontend/src/index.ts` — add exports
- `tools/viewer-api/frontend/src/theme.ts` — new
- `tools/viewer-api/frontend/src/effects/` — new directory
- `tools/viewer-api/frontend/src/components/HypergraphView/` — new directory
- `tools/log-viewer/frontend/src/store/theme.ts` — thin re-export
- `tools/log-viewer/frontend/src/effects/` — thin re-exports
- `tools/log-viewer/frontend/src/components/HypergraphView/` — imports from shared

**Validation:**
- `cargo build -p viewer-api` passes
- `cargo build -p log-viewer` passes
- `cargo build -p doc-viewer` passes
- `cd tools/log-viewer/frontend && npm run build` succeeds
- Log-viewer works identically after extraction (manual test)

---

## Phase 2: Integrate New Features into doc-viewer ⏸️

**Goal:** Add hypergraph visualization and shared components to doc-viewer.

**Blocked:** Depends on HypergraphView extraction (Phase 1B step 1.9).

| Step | Description |
|------|-------------|
| 2.1 | Add `@context-engine/viewer-api-frontend` HypergraphView dependency to doc-viewer frontend |
| 2.2 | Create a `HypergraphPage` component in doc-viewer that wraps the shared HypergraphView |
| 2.3 | Add hypergraph tab/route to doc-viewer navigation |
| 2.4 | Add shared theme support to doc-viewer (consume extracted theme system) |
| 2.5 | Add shared effects to doc-viewer (consume extracted WebGPU effects) |
| 2.6 | Identify and implement doc-specific features (TBD — needs further scoping) |

**Files affected:**
- `tools/doc-viewer/frontend/package.json` — dependency update
- `tools/doc-viewer/frontend/src/components/HypergraphPage.tsx` — new
- `tools/doc-viewer/frontend/src/App.tsx` — add tab/route
- `tools/doc-viewer/frontend/src/store.ts` — add hypergraph state signals

**Validation:**
- Doc-viewer frontend builds: `cd tools/doc-viewer/frontend && npm run build`
- Hypergraph view renders in doc-viewer (manual test)

---

## Phase 3: Refactor Event Names & Modular ID Paths ✅

**Goal:** Make event naming semantic, consistent, namespaced by operation type and module.

**Completed:** path_id format changed to `<op_type>/<module>/<semantic_id>`. Added `parse_path_id()` + `OperationType::from_path_id()`. Legacy `search_state` parsing removed. Frontend `SearchStatePanel` updated with `parsePathId`/`formatPathIdShort`. Transition names audited — all 18 already consistent, no renames needed.

### Phase 3A: path_id Refactoring (Rust)

| Step | Description |
|------|-------------|
| 3.1 | Design new path_id format: `<op_type>/<module>/<semantic_id>` (e.g. `search/context-search/token-42-start-0`) |
| 3.2 | Update `context-search` path_id generation in `crates/context-search/src/state/start/core.rs` |
| 3.3 | Update `context-insert` path_id generation in `crates/context-insert/src/visualization.rs` |
| 3.4 | Design path_id format for future `context-read` events |

### Phase 3B: Transition Naming Cleanup (Rust)

| Step | Description |
|------|-------------|
| 3.5 | Audit all 18 transition kinds for naming consistency |
| 3.6 | Rename inconsistent transition variants in `crates/context-trace/src/graph/visualization.rs` |
| 3.7 | Update all call sites in `context-search` and `context-insert` |
| 3.8 | Re-export TypeScript types: `cargo test -p context-trace -p log-viewer export_bindings` |

### Phase 3C: Frontend Cleanup

| Step | Description |
|------|-------------|
| 3.9 | Remove legacy `search_state` parsing from `log-viewer/frontend/src/store/index.ts` |
| 3.10 | Update `SearchStatePanel` to use new path_id display format |
| 3.11 | Update `PathChainPanel` labels if transition names changed |

**Files affected:**
- `crates/context-trace/src/graph/visualization.rs` — Transition enum, path_id helpers
- `crates/context-search/src/search/mod.rs` — ~10 emission call sites
- `crates/context-search/src/state/start/core.rs` — path_id generation
- `crates/context-insert/src/visualization.rs` — path_id generation
- `crates/context-insert/src/insert/context.rs` — emission call sites
- `crates/context-insert/src/join/context/frontier.rs` — emission call sites
- `tools/log-viewer/frontend/src/types/generated/` — regenerated
- `tools/log-viewer/frontend/src/store/index.ts` — remove legacy parsing
- `tools/log-viewer/frontend/src/components/HypergraphView/components/SearchStatePanel.tsx`

**Validation:**
- All Rust tests pass: `cargo test -p context-trace -p context-search -p context-insert`
- TS types regenerated: `cargo test -p context-trace -p log-viewer export_bindings`
- Frontend builds: `cd tools/log-viewer/frontend && npm run build`
- Existing log files still parse correctly (new format, old logs still work via path_id prefix detection)

---

## Phase 4: Add Query Path Visualization ✅

**Goal:** Visualize the input pattern (query) as a separate path with its own panel, events, and styling.

**Implementation notes:**
- No new Transition variants needed — cursor position and matched state are metadata on existing events via `QueryInfo` fields
- Added `matched_positions: Vec<usize>` and `active_token: Option<usize>` to `QueryInfo`
- `emit_graph_op` infers cursor position from transition variant fields (ChildMatch, ChildMismatch, MatchAdvance)
- QueryPathPanel renders as horizontal token strip with progress bar and animated states

### Phase 4A: Rust Event Emission ✅

| Step | Description |
|------|-------------|
| 4.1 | Design query path events: pattern root display, cursor advance, token match/mismatch |
| 4.2 | Add query path transitions to `Transition` enum (e.g. `QueryAdvance`, `QueryMatch`, `QueryMismatch`) |
| 4.3 | Emit query path events from `context-search` during search |
| 4.4 | Include pattern info in `QueryInfo` (or new dedicated field on `GraphOpEvent`) |

### Phase 4B: Frontend Visualization ✅

| Step | Description |
|------|-------------|
| 4.5 | Create `QueryPathPanel` component — text panel showing input pattern tokens with cursor |
| 4.6 | Add query path node/edge styling to `useVisualizationState.ts` |
| 4.7 | Add query path edge colors to `useOverlayRenderer.ts` |
| 4.8 | Integrate `QueryPathPanel` into HypergraphView layout |
| 4.9 | Sync query cursor with search step navigation |

**Files affected:**
- `crates/context-trace/src/graph/visualization.rs` — new Transition variants, QueryInfo extensions
- `crates/context-search/src/search/mod.rs` — new event emission points
- `tools/log-viewer/frontend/src/types/generated/` — regenerated
- `tools/log-viewer/frontend/src/components/HypergraphView/components/QueryPathPanel.tsx` — new
- `tools/log-viewer/frontend/src/components/HypergraphView/hooks/useVisualizationState.ts`
- `tools/log-viewer/frontend/src/components/HypergraphView/hooks/useOverlayRenderer.ts`
- `tools/log-viewer/frontend/src/components/HypergraphView/HypergraphView.tsx`

**Validation:**
- Query path panel displays input pattern tokens with cursor position
- Cursor advances as search steps progress
- Token match/mismatch events trigger visual feedback in the query panel
- Edge/node styling distinguishes query path from search path

---

## Phase 5: Fix Search Path Edge & Node Highlighting

**Goal:** Fix intermediate edge gaps and stale highlighting after parent candidate rejection.

| Step | Description |
|------|-------------|
| 5.1 | Analyze `findDescendantPath` BFS failures — identify topologies where it misses edges |
| 5.2 | Fix BFS to handle multi-pattern edges, disconnected sub-paths, and long-distance shortcuts |
| 5.3 | Add path edge reset logic when parent candidate is rejected |
| 5.4 | Ensure `searchStartEdgeKeys` is recomputed (not accumulated) when start_path changes |
| 5.5 | Add test cases for edge highlighting in `reconstruction.test.ts` |

**Files affected:**
- `tools/log-viewer/frontend/src/components/HypergraphView/hooks/useVisualizationState.ts` — BFS fix, reset logic
- `tools/log-viewer/frontend/src/search-path/reconstruction.test.ts` — new test cases
- `tools/log-viewer/frontend/src/components/HypergraphView/hooks/useOverlayRenderer.ts` — if edge data plumbing changes

**Validation:**
- Edge highlighting updates correctly when stepping through parent rejection → new parent exploration
- No stale edges remain highlighted from previous rejected paths
- All intermediate edges between VizPathGraph nodes are found and highlighted
- Existing tests pass: `cd tools/log-viewer/frontend && npm test`

---

## Phase 6: Finish Search Path Visualization

**Goal:** Complete the search path visualization with proper resets and cursor display.

| Step | Description |
|------|-------------|
| 6.1 | Implement visualization reset for rejected parent candidates — clear start_path highlighting when parent is rejected |
| 6.2 | Implement end_path reset when backtracking from a failed child comparison |
| 6.3 | Add cursor position overlay to PathChainPanel or as separate widget |
| 6.4 | Ensure PathChainPanel breadcrumb updates correctly during step transitions |
| 6.5 | Review and fix any remaining search event ordering or field issues |

**Files affected:**
- `tools/log-viewer/frontend/src/components/HypergraphView/hooks/useVisualizationState.ts`
- `tools/log-viewer/frontend/src/components/HypergraphView/components/PathChainPanel.tsx`
- `tools/log-viewer/frontend/src/components/HypergraphView/components/SearchStatePanel.tsx`
- Possibly `crates/context-search/src/search/mod.rs` — if events need fixing

**Validation:**
- Stepping through a full search with rejections shows correct path at each step
- No stale highlights from rejected parent/child paths
- Cursor position is visible and synced with current step

---

## Phase 7: Complete Context-Insert Visualization ✅ DONE

**Goal:** Full visualization overhaul for insert operations: dedicated styling, graph mutation tracking, separate panel.

### Phase 7A: Graph Mutation Events (Rust) ✅

| Step | Description | Status |
|------|-------------|--------|
| 7.1 | Design graph update event schema — `DeltaOp` enum + `GraphDelta` struct | ✅ |
| 7.2 | Extend `GraphOpEvent` with `graph_delta: Option<GraphDelta>` field | ✅ |
| 7.3 | Emit graph deltas from `context-insert` split/join operations | ✅ |
| 7.4 | Frontend: regenerate TS bindings, update barrel exports | ✅ |

### Phase 7B: Frontend Insert Visualization ✅

| Step | Description | Status |
|------|-------------|--------|
| 7.5 | Add CSS classes for insert-specific node roles: `viz-split-source`, `viz-split-left`, `viz-split-right`, `viz-join-left`, `viz-join-right`, `viz-join-result`, `viz-new-pattern`, `viz-new-pattern-child`, `viz-new-root` | ✅ |
| 7.6 | Update `useVisualizationState.ts` to derive insert-specific node states from Transition | ✅ |
| 7.7 | Update `getNodeVizClasses` and `getNodeVizStates` for insert roles | ✅ |
| 7.8 | Create `InsertStatePanel` component (separate from SearchStatePanel) | ✅ |
| 7.9 | Show before/after graph states in InsertStatePanel (graph mutation tracking) | ✅ |
| 7.10 | Add insert-specific edge colors to `useOverlayRenderer.ts` | ✅ |
| 7.11 | Update SearchStatePanel to distinguish insert path groups visually | ✅ |

**Files affected:**
- `crates/context-trace/src/graph/visualization.rs` — GraphOpEvent extensions
- `crates/context-insert/src/visualization.rs` — delta emission
- `crates/context-insert/src/insert/context.rs` — graph delta emission
- `crates/context-insert/src/join/context/frontier.rs` — graph delta emission
- `tools/log-viewer/frontend/src/types/generated/` — regenerated
- `tools/log-viewer/frontend/src/components/HypergraphView/hooks/useVisualizationState.ts`
- `tools/log-viewer/frontend/src/components/HypergraphView/hooks/useOverlayRenderer.ts`
- `tools/log-viewer/frontend/src/components/HypergraphView/components/InsertStatePanel.tsx` — new
- `tools/log-viewer/frontend/src/components/HypergraphView/hypergraph.css` — new classes
- `tools/log-viewer/frontend/src/store/index.ts` — graph snapshot mutation handling

**Validation:**
- Insert operations show dedicated styling per node role
- Split: source node, left/right fragments clearly distinguished
- Join: left/right/result nodes clearly distinguished
- Graph mutations are tracked and visualizable (before/after)
- InsertStatePanel shows insert-specific step information

---

## Phase 8: Document Search & Insert Events

**Goal:** Comprehensive event documentation as agent guide + inline Rust docs + CSS mapping.

| Step | Description |
|------|-------------|
| 8.1 | Create `agents/guides/20260301_GRAPH_OP_EVENTS_GUIDE.md` with full transition reference |
| 8.2 | For each of the ~18+ transitions: when it fires, what state it represents, what LocationInfo accompanies it |
| 8.3 | Include CSS class mapping table: transition → viz classes triggered on frontend |
| 8.4 | Include insert-specific events and their visual mapping |
| 8.5 | Update inline Rust doc comments on each `Transition` variant in `visualization.rs` |
| 8.6 | Add to `agents/guides/INDEX.md` |
| 8.7 | Design (document-only) context-read event schema for future implementation |

**Files affected:**
- `agents/guides/20260301_GRAPH_OP_EVENTS_GUIDE.md` — new
- `agents/guides/INDEX.md` — updated
- `crates/context-trace/src/graph/visualization.rs` — doc comment improvements

**Validation:**
- Guide covers all Transition variants
- CSS mapping table matches actual `getNodeVizClasses` implementation
- Rust doc comments are accurate and match guide

---

## Risk Assessment

| Risk | Mitigation |
|------|-----------|
| Phase 1 extraction breaks existing functionality | Incremental extraction with tests after each step |
| Phase 3 renaming breaks old log files | Frontend detects path_id prefix to handle both old and new formats |
| Phase 4 query path events add too much overhead | Make events optional via feature flag or tracing level |
| Phase 7 graph mutations are complex to serialize | Start with simple delta format, iterate |
| HypergraphView extraction is tightly coupled to log-viewer state | Parameterize the component to accept generic data sources |

---

## Dependencies Between Phases

```
Phase 1 (Extract) ──→ Phase 2 (Doc-viewer integration)
                  ──→ Phase 3 (Refactor events) ──→ Phase 4 (Query path viz)
                                                 ──→ Phase 5 (Fix highlighting)
                                                 ──→ Phase 6 (Finish search viz)
                                                 ──→ Phase 7 (Insert viz)
Phase 3-7 (all implementation) ──→ Phase 8 (Documentation)
```

Phases 4, 5, 6, 7 can be parallelized after Phase 3 completes.
Phase 8 should be done last or incrementally alongside phases 4-7.
