# Interview: Viewer Tools Feature Plan

**Date:** 2026-03-01
**Scope:** log-viewer, doc-viewer, viewer-api
**Status:** ANSWERED — ready for plan creation

---

## Answers Summary

### Q1: Search Path Visualization — Unfinished Work
1. **Reset visualization for rejected parent candidates + end paths** — when a parent candidate is rejected and a different one explored, the visualization doesn't reset properly
2. **Query cursor path** — showing input tokens and cursor with additional edge/node styling for query path
3. **Fix missing or invalid search events** — review ordering, naming, and fields of emitted events

### Q2: Query Path Visualization — Definition
A **query path** is a special path where the root is a **Pattern** (list of nodes) rather than a single graph node. This pattern is the input to the search operation and is advanced during search. Requires:
1. A **query token overlay** — visual bar/panel showing query_tokens synced with cursor_position
2. A **text panel** with information about the input pattern to the search operation
3. **New events** emitted as the pattern is advanced during search
4. **Edge and node styling** specific to the query path

### Q3: Event Names & Modular ID Paths — Full Scope
All selected:
- **Semantic path_ids** — human-readable (e.g. `search/token-42/attempt-1`)
- **Fix transition naming inconsistencies**
- **Drop legacy `search_state` parsing** from frontend
- **Namespace by operation type** (`search/<id>`, `insert/<id>`, `read/<id>`)
- **Module-level namespacing** (by crate/module)

### Q4: Context-Insert Visualization — Full Overhaul
- **Dedicated node styling** — CSS classes for split fragments, join participants, new patterns
- **Graph mutation tracking** — show before/after states when nodes split or join
- **Separate UI panel** — distinct panel treatment for insert vs search operations
- **Graph update events** — support adding/removing nodes, edges, node data in events

### Q5: Context-Read Visualization — Design Only
Design the event schema and transitions for context-read, but don't implement yet. Implementation deferred.

### Q6: Edge/Node Highlighting Bugs
- **Intermediate edge gaps** — BFS `findDescendantPath` misses edges in certain topologies
- **Missing path edge update after parent candidate rejection** — when a parent is rejected and a different one explored, the old path edges remain highlighted

### Q7: Doc-Viewer New Features
- **Hypergraph visualization** — add to doc-viewer
- **Doc-specific new features** (beyond log-viewer features)
- **Better shared infrastructure consumption** from viewer-api

### Q8: Infrastructure Extraction to viewer-api
From Rust: **JQ query engine**, **MCP server setup boilerplate**, **source file serving**
From Frontend: **Hypergraph components**, **themes**, **effects**, **list components**

### Q9: Search Event Documentation — Comprehensive
- **Full transition reference** — when each fires, what it means, accompanying LocationInfo
- **Agent guide** in `agents/guides/`
- **Inline Rust doc comments** on each variant
- **Cover both search and insert events**
- **Include CSS mapping table** (which viz classes each transition triggers)

### Q10: Ordering — Revised
User-preferred order:
1. **Extract infrastructure to viewer-api** (do first)
2. **Integrate with doc-viewer** (right after extraction)
3. **Refactor event names & modular ID paths**
4. **Add query path visualization**
5. **Fix search path edge/node highlighting bugs**
6. **Finish search path visualization**
7. **Complete context-insert visualization**
8. **Document search events** (alongside or after implementation)

---

## Context Summary

After thoroughly researching the three tool codebases, here is the current state:

### log-viewer
- **Frontend:** Preact + WebGPU hypergraph visualization with search path display
- **Backend:** Rust HTTP + MCP server with JQ query support
- **Search visualization:** Mostly complete — `VizPathGraph` tracks start_path → root → end_path, edges highlighted via `edgePairKey` matching, nodes styled with CSS classes (`viz-sp-start`, `viz-sp-root`, etc.)
- **Insert visualization:** Basic `getPrimaryNode()` support for insert transitions (`split_start`, `split_complete`, `join_*`, `create_*`, `update_pattern`), but no dedicated styling or layout logic
- **Context-read:** `OperationType::Read` exists but no events are emitted from `context-read` crate

### doc-viewer
- **Frontend:** Preact app with TreeView, Sidebar, DocViewer, FilterPanel, CodeViewer
- **Backend:** Rust HTTP + MCP server for managing agent docs and crate API docs
- **Uses:** `@context-engine/viewer-api-frontend` for shared components (TreeView, Spinner, TabBar, CodeViewer, etc.)

### viewer-api
- **Rust lib:** `ServerConfig`, `ServerArgs`, `init_tracing`, `default_cors`, `with_static_files`, `SessionStore`
- **Frontend package:** Shared components (TreeView, Spinner, TabBar, Icons, Header, Sidebar, Layout, CodeViewer) + styles + Preact/signals re-exports

### Event naming
- Events use `message == "graph_op"` with JSON payload in `graph_op` field
- Legacy `search_state` format still parsed by frontend but no longer emitted
- `path_id` format: `search-<node_index>-<nanos>` or `insert-<nanos>`
- `OperationType`: `search | insert | read`

---

## Questions to Resolve

### Q1: Search Path Visualization — What's "unfinished"?

The search path visualization has:
- ✅ `VizPathGraph` data model (start_node → start_path → root → end_path)
- ✅ Path reconstruction from transitions (`reconstruction.ts`)
- ✅ Edge highlighting with start/root/end color coding
- ✅ Node classes for path roles (sp-start, sp-root, sp-start-path, sp-end-path)
- ✅ `PathChainPanel` breadcrumb display
- ✅ `SearchStatePanel` with path groups and step navigation
- ✅ `computeSearchPathLayout` for root-anchored layout

**What specifically is unfinished?** Possible gaps:
- (a) The cursor position / query progress bar visualization?
- (b) Animation between steps (currently instant)?
- (c) Something about the path chain panel display?
- (d) Integration with specific edge cases in the search algorithm?
- (e) Something else entirely?

### Q2: "Query Path Visualization" — What is this?

Is this:
- (a) Visualizing the **query tokens** alongside the search path (e.g. a separate bar or overlay showing which atoms in the query have been matched)?
- (b) A different kind of path through the graph related to **query execution plan**?
- (c) The `QueryInfo.query_tokens` + `cursor_position` displayed as a timeline/progress bar synced with search steps?
- (d) Something related to `context-read` queries?
- (e) Something else?

### Q3: "Refactor event names and modular ID paths" — Scope clarification

Current state:
- Event message: `"graph_op"` (unified), legacy `"search_state"` still parsed
- `path_id`: `search-<idx>-<nanos>` or `insert-<nanos>` (timestamp-based, not human-readable)
- `OperationType`: `search | insert | read`
- Transition kinds: `start_node`, `visit_parent`, `visit_child`, `child_match`, `child_mismatch`, `done`, `dequeue`, `root_explore`, `match_advance`, `parent_explore`, `split_start`, `split_complete`, `join_start`, `join_step`, `join_complete`, `create_pattern`, `create_root`, `update_pattern`

Questions:
- (a) Should `path_id` become more semantic (e.g. `search/token-42/attempt-1` instead of `search-42-1740000000`)?
- (b) Are there naming inconsistencies in transition kinds to fix?
- (c) Should the frontend drop legacy `search_state` parsing entirely?
- (d) Should `OperationType` affect how `path_id` is structured (e.g. `search/<id>`, `insert/<id>`, `read/<id>`)?
- (e) What's the "modular" part — is this about namespacing by crate/module (e.g. `context-search/search/token-42`)?

### Q4: Context-Insert Visualization — Current State vs. Goal

Current state:
- Insert events are emitted: `SplitStart`, `SplitComplete`, `JoinStart`, `JoinStep`, `JoinComplete`, `CreatePattern`, `CreateRoot`, `UpdatePattern`
- `getPrimaryNode()` handles all insert transitions
- No dedicated CSS classes or visual styles for insert-specific node roles
- No dedicated layout logic (uses same focused layout as search)
- The `SearchStatePanel` shows insert events in path groups but doesn't distinguish them visually from search events

What's needed:
- (a) Dedicated node styling for insert operations (split fragments, join participants, new patterns)?
- (b) A different layout algorithm for insert operations?
- (c) Visual graph mutation tracking (showing before/after states when nodes split or join)?
- (d) Separate panel or UI treatment for insert operations vs. search operations?
- (e) All of the above?

### Q5: Context-Read Visualization — Scope

The `context-read` crate does not yet emit `graph_op` events. The `OperationType::Read` exists but is unused.

- (a) Is the plan to add event emission in `context-read` as part of this work?
- (b) Or is `context-read` visualization specifically listed under "things we will do later"?
- (c) If (a), what transitions/events should `context-read` emit? (Reading patterns, expanding context, etc.)

### Q6: "Fix search path edge and node highlighting" — Known Bugs

The current edge highlighting uses `edgePairKey` (pair-based, ignoring `pattern_idx`) for search path edges. Potential issues:

- (a) Are edges being highlighted for the **wrong pattern** when a node has multiple patterns? (e.g. edge from→to exists in pattern 0 and pattern 1, but only pattern 0 is in the search path)
- (b) Are intermediate edges between VizPathGraph nodes not being found correctly? (The `findDescendantPath` BFS fills in gaps but might miss edges in certain graph topologies)
- (c) Is the issue specifically about **node** highlighting (e.g. nodes not getting the right viz-sp-* classes)?
- (d) Are the start_path edges being swapped incorrectly? (Start edges are child→parent but layout edges are parent→child)
- (e) Can you describe a specific scenario where the highlighting is wrong?

### Q7: "Integrate new features to doc-viewer" — What features?

The doc-viewer currently handles:
- Agent docs management (CRUD via MCP + HTTP)
- Crate API docs viewing
- Source file viewing
- JQ query filtering

What new features should be integrated?
- (a) Hypergraph visualization (same as log-viewer)?
- (b) Log viewing capabilities?
- (c) Search/insert event replay?
- (d) Some doc-viewer-specific new features unrelated to log-viewer?
- (e) Is this about making the doc-viewer consume the **shared infrastructure** from viewer-api, or about adding entirely new functionality?

### Q8: "Extract infrastructure and boilerplate from log-viewer to viewer-api" — Specifics

Currently shared in `viewer-api` (Rust): `ServerConfig`, `ServerArgs`, `init_tracing`, `default_cors`, `with_static_files`, `SessionStore`
Currently shared in `viewer-api` frontend: TreeView, Spinner, TabBar, Icons, Header, Sidebar, Layout, CodeViewer, base styles

What additional code should be extracted?
- (a) The `JQ query` engine? (Both log-viewer and doc-viewer have their own `query.rs`)
- (b) Log file parsing? (Currently only in log-viewer)
- (c) MCP server boilerplate? (Both tools have their own MCP server setup)
- (d) The hypergraph visualization components? (Currently only in log-viewer)
- (e) Frontend state management patterns? (Store, effects, hooks)
- (f) Source file serving? (Both tools serve source files)
- (g) Are there specific patterns or components you've identified as duplicated?

### Q9: "Document search events and trigger conditions"

- (a) Is this about creating a reference document describing every `Transition` variant — when it fires, what it means, what `LocationInfo` accompanies it?
- (b) Should this be a markdown guide in `agents/guides/`, or inline documentation in the Rust source?
- (c) Should it also cover insert events, or just search events?
- (d) Should the document include the frontend CSS class mapping (which viz classes each transition triggers)?

### Q10: Overall Ordering Preferences

My initial ordering based on dependency analysis:

1. **Document search events** (foundational knowledge, helps with all other work)
2. **Fix search path edge and node highlighting** (fix bugs before building on top)
3. **Finish search path visualization** (complete what exists)
4. **Refactor event names and modular ID paths** (clean up before adding more events)
5. **Add query path visualization** (requires clean events + working search viz)
6. **Complete context-insert visualization** (builds on refactored events)
7. **Extract infrastructure to viewer-api** (refactor after features stabilize)
8. **Integrate new features to doc-viewer** (depends on extracted infrastructure)

Does this ordering make sense, or do you prefer a different sequence?

---

## Architecture Notes (for plan creation)

### Key files per feature area

**Search path visualization:**
- `tools/log-viewer/frontend/src/search-path/reconstruction.ts` — VizPathGraph reconstruction
- `tools/log-viewer/frontend/src/components/HypergraphView/hooks/useVisualizationState.ts` — node/edge state derivation
- `tools/log-viewer/frontend/src/components/HypergraphView/hooks/useOverlayRenderer.ts` — WebGPU edge rendering
- `tools/log-viewer/frontend/src/components/HypergraphView/components/PathChainPanel.tsx` — path breadcrumb
- `tools/log-viewer/frontend/src/components/HypergraphView/components/SearchStatePanel.tsx` — step navigation
- `tools/log-viewer/frontend/src/components/HypergraphView/layout.ts` — layout algorithms

**Event emission (Rust):**
- `crates/context-trace/src/graph/visualization.rs` — `GraphOpEvent`, `Transition`, `LocationInfo`, `OperationType`, `QueryInfo`
- `crates/context-search/src/search/mod.rs` — search event emission (~10 call sites)
- `crates/context-insert/src/visualization.rs` — insert event helpers
- `crates/context-insert/src/insert/context.rs` — split/join event emission
- `crates/context-insert/src/join/context/frontier.rs` — join step events

**Shared infrastructure:**
- `tools/viewer-api/src/lib.rs` — Rust server infrastructure
- `tools/viewer-api/frontend/src/` — shared frontend components
- `tools/log-viewer/src/router.rs` — HTTP routes (potential extraction target)
- `tools/log-viewer/src/query.rs` — JQ engine (potential extraction target)
- `tools/log-viewer/src/source.rs` — source file serving (potential extraction target)

**Doc-viewer integrations:**
- `tools/doc-viewer/src/main.rs` — server setup
- `tools/doc-viewer/frontend/src/App.tsx` — main app structure
- `tools/doc-viewer/frontend/src/store.ts` — state management
