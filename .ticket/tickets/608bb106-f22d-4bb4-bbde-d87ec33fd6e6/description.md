---
tags: `#plan` `#visualization` `#search` `#logging` `#frontend`
summary: Emit structured search state events from context-search and visualize them as colored node annotations in the hypergraph view with replay capability.
status: 📋
---

# Search Visualization in Hypergraph View

## Objective

Emit structured search-state events from `context-search` during algorithm execution, pipe them through the existing tracing→log-viewer pipeline, and visualize them in the hypergraph view with distinct colors per node role. Support both log replay and live algorithm execution with optional delay.

---

## Context

### Current Pipeline

```
Rust: graph.emit_graph_snapshot()
  → tracing::info!(graph_data = %json, "graph_snapshot")
  → log file (JSON lines)

Backend: LogParser::parse() → Vec<LogEntry>
  → /api/logs/:file → JSON response

Frontend: entries signal → computed hypergraphSnapshot
  → finds first entry with message=="graph_snapshot"
  → JSON.parse(fields.graph_data) → HypergraphSnapshot
  → buildLayout() → 3D force-directed layout
  → HypergraphView.tsx → DOM nodes + WebGPU edges
```

### Search Algorithm Concepts

The search algorithm maintains these node roles at each step:

| Role | Description | Color |
|------|-------------|-------|
| **Query cursor** | Current position in the search query pattern | Bright cyan / electric blue |
| **Start node** | The initial token where search began | White / bright marker |
| **Previously matched** | Nodes confirmed as matching the query so far | Green |
| **Partially matched** | Single node currently being compared (candidate match) | Yellow / amber |
| **Candidate parents** | Parent nodes in the BFS queue (`ParentCandidate`) | Orange |
| **Candidate children** | Child nodes in the BFS queue (`ChildCandidate`) | Purple / magenta |
| **Current root** | The root node of the current match being explored | Gold ring |

### Key Types (Rust side)

- `SearchNode::ParentCandidate(ParentCompareState)` — parent in BFS queue
- `SearchNode::ChildCandidate(CompareState<Candidate, Candidate>)` — child in BFS queue
- `CompareState<Matched, Matched>` — a confirmed match state
- `PatternCursor` with `atom_position: AtomPosition` — cursor in query
- `SearchIterator` with `queue: SearchQueue` — BFS priority queue
- `MatchResult` with `PathCoverage` — final/intermediate match result
- `RootCursor` — explores matches within a single root

### Files Affected

**Rust (new/modified):**
- `crates/context-trace/src/graph/snapshot.rs` — extend `GraphSnapshot` with optional search state **OR** add new `SearchSnapshot` struct
- `crates/context-search/src/search/mod.rs` — emit search state events at key transitions
- `crates/context-search/src/match/iterator.rs` — emit events when queue changes
- `crates/context-search/src/match/mod.rs` — emit events on node comparison results
- `crates/context-search/src/search/searchable.rs` — emit initial search setup event

**Frontend (new/modified):**
- `tools/log-viewer/frontend/src/types/index.ts` — add `SearchStateEvent` types
- `tools/log-viewer/frontend/src/store/index.ts` — parse search events, add `searchStates` computed signal
- `tools/log-viewer/frontend/src/components/HypergraphView/HypergraphView.tsx` — render node colors based on active search state
- `tools/log-viewer/frontend/src/components/HypergraphView/layout.ts` — extend `LayoutNode` with optional role/color override
- `tools/log-viewer/frontend/src/components/HypergraphView/hypergraph.css` — role-based CSS classes

---

## Design

### 1. Search State Event Format (Rust → JSON)

Emit a `tracing::info!` event with message `"search_state"` at each meaningful transition:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct SearchStateSnapshot {
    /// Monotonically increasing step counter
    pub step: usize,
    /// Human-readable description of what happened
    pub description: String,
    /// What phase of the algorithm we're in
    pub phase: SearchPhase,
    /// The query tokens being searched for
    pub query_tokens: Vec<usize>,
    /// Current cursor position in the query (atom index)
    pub cursor_position: usize,
    /// Node where search started
    pub start_node: usize,
    /// Nodes confirmed as matched so far
    pub matched_nodes: Vec<usize>,
    /// Single node currently being compared (partially matched)
    pub partial_node: Option<usize>,
    /// Parent candidate nodes in the BFS queue
    pub candidate_parents: Vec<usize>,
    /// Child candidate nodes in the BFS queue
    pub candidate_children: Vec<usize>,
    /// The root node currently being explored (RootCursor)
    pub current_root: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub enum SearchPhase {
    /// Search just started, initial queue populated
    Init,
    /// Popped a node from queue, about to compare
    Dequeue,
    /// Comparing leaf tokens
    Compare,
    /// Found a root match, exploring via RootCursor
    RootExplore,
    /// Advanced match within root
    MatchAdvance,
    /// Need parent exploration (root boundary reached)
    ParentExplore,
    /// Search concluded
    Done,
}
```

Emitted via:
```rust
tracing::info!(
    search_state = %serde_json::to_string(&snapshot).unwrap(),
    step = snapshot.step,
    "search_state"
);
```

### 2. Emission Points in the Algorithm

| Location | Phase | What changed |
|----------|-------|--------------|
| `searchable.rs` `start_search()` | `Init` | Initial queue, start node, query |
| `SearchIterator::next()` — found root match | `RootExplore` | current_root set, queue cleared |
| `NodeConsumer::consume()` — ParentCandidate advance | `Dequeue` | Popped parent from queue |
| `NodeConsumer::compare_next()` — FoundMatch | `Compare` | partial_node → matched |
| `NodeConsumer::compare_next()` — Prefixes | `Compare` | New children added to queue |
| `SearchState::finish_root_cursor()` — match advanced | `MatchAdvance` | matched_nodes grows, cursor advances |
| `SearchState::finish_root_cursor()` — inconclusive | `ParentExplore` | New parents in queue |
| `SearchState::finish_root_cursor()` — conclusive | `Done` | Final state |
| `SearchState::search()` — final result | `Done` | Final matched/unmatched state |

### 3. Frontend Data Model

```typescript
export interface SearchStateEvent {
  step: number;
  description: string;
  phase: string;  // 'Init' | 'Dequeue' | 'Compare' | 'RootExplore' | 'MatchAdvance' | 'ParentExplore' | 'Done'
  query_tokens: number[];
  cursor_position: number;
  start_node: number;
  matched_nodes: number[];
  partial_node: number | null;
  candidate_parents: number[];
  candidate_children: number[];
  current_root: number | null;
}
```

### 4. Store Signal

```typescript
// Collect all search_state events from the log
export const searchStates = computed((): SearchStateEvent[] => {
  return entries.value
    .filter(e => e.message === 'search_state' && e.fields?.search_state)
    .map(e => JSON.parse(e.fields.search_state as string))
    .sort((a, b) => a.step - b.step);
});

// Currently active search step (controlled by slider/playback)
export const activeSearchStep = signal<number>(-1);

// The active search state snapshot
export const activeSearchState = computed((): SearchStateEvent | null => {
  const step = activeSearchStep.value;
  const states = searchStates.value;
  if (step < 0 || step >= states.length) return null;
  return states[step];
});
```

### 5. Hypergraph View Integration

**Node coloring:** Each `LayoutNode` gets a `role` field derived from the active search state:

```typescript
type NodeRole = 'default' | 'start' | 'matched' | 'partial' | 'candidate-parent' | 'candidate-child' | 'current-root' | 'query';
```

Role is computed per-frame by checking the node's index against the active search state's sets. CSS classes (`.hg-node.role-matched`, `.hg-node.role-candidate-parent`, etc.) control the visual style.

**Playback controls:** A small panel below the hypergraph view with:
- Step counter: "Step 3/17"
- Prev/Next buttons
- Play/Pause with speed control
- The description text from the current step

**Query display:** Show the query tokens with the cursor position highlighted (small bar above or below the graph).

### 6. Replay vs. Live Mode

**Replay mode (default):** Log file already contains all `search_state` events. The slider/playback controls step through them.

**Live mode:** The Rust algorithm emits events through tracing. The log-viewer watches for new entries (existing polling mechanism). A configurable delay can be injected:

```rust
// In context-search, gated behind a feature flag or runtime config
pub fn emit_search_state(snapshot: &SearchStateSnapshot) {
    let json = serde_json::to_string(snapshot).unwrap_or_default();
    tracing::info!(
        search_state = %json,
        step = snapshot.step,
        "search_state"
    );
    
    // Optional delay for human viewing
    if let Ok(delay) = std::env::var("SEARCH_VIZ_DELAY_MS") {
        if let Ok(ms) = delay.parse::<u64>() {
            std::thread::sleep(std::time::Duration::from_millis(ms));
        }
    }
}
```

---

## Execution Plan

### Phase 1: Rust-side event emission (context-search + context-trace)

1. **Add `SearchStateSnapshot` struct** to `crates/context-trace/src/graph/snapshot.rs` (or a new `search_snapshot.rs` module)
   - Add `serde::Serialize` derive
   - Add `SearchPhase` enum
   - Add `emit_search_state()` helper function

2. **Add step counter** to `SearchState` struct in `crates/context-search/src/search/mod.rs`
   - `step_counter: usize` field
   - Increment and emit at each transition point

3. **Wire emission points** in `search/mod.rs`:
   - `SearchState::search()` — Init and Done
   - `SearchState` Iterator impl — Dequeue/RootExplore
   - `SearchState::finish_root_cursor()` — MatchAdvance, ParentExplore, Done

4. **Wire emission points** in `match/iterator.rs`:
   - `SearchIterator::next()` — root cursor found
   - `SearchIterator::find_next_root_match()` — node consumed, queue updated

5. **Extract current queue state** — helper to collect node indices from `SearchQueue`

6. **Test:** Run existing search tests with `LOG_STDOUT=1 LOG_FILTER=info` and verify `search_state` events appear in logs

### Phase 2: Frontend parsing and store

7. **Add TypeScript types** in `types/index.ts`
   - `SearchStateEvent` interface

8. **Add store signals** in `store/index.ts`
   - `searchStates` computed signal (parse from entries)
   - `activeSearchStep` signal
   - `activeSearchState` computed signal

### Phase 3: Hypergraph view visualization

9. **Extend `LayoutNode`** with optional `role` field in `layout.ts`

10. **Add role computation** in `HypergraphView.tsx`
    - Per-frame: map active search state → node roles
    - Apply CSS classes to DOM nodes based on role

11. **Add role-based CSS** in `hypergraph.css`
    - Color/glow for each role

12. **Add playback controls** component
    - Step slider, prev/next, play/pause
    - Description display
    - Query cursor display

### Phase 4: Polish

13. **Transitions:** Smooth color transitions when stepping between states
14. **Edge highlighting:** Highlight edges involved in current comparison
15. **Live mode:** Verify polling picks up new entries in real-time

---

## Risks

| Risk | Mitigation |
|------|------------|
| Too many events in large searches (performance) | Gate behind feature flag or log level; batch small steps |
| Serialization cost of queue state | Only serialize node indices (usize), not full state |
| Frontend slowdown with many search states | Virtualize playback; only compute active state |
| Node indices changing between snapshot and search events | Search events reference the same graph snapshot emitted earlier |

---

## Validation

- [ ] Search tests pass with events enabled
- [ ] Events appear in log files with correct format
- [ ] Frontend parses events from log file
- [ ] Hypergraph nodes change color when stepping through states
- [ ] Playback controls work (step, play, pause)
- [ ] Live mode updates as new events arrive
