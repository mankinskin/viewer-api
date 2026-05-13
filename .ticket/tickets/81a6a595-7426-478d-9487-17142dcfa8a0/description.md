---
tags: `#context-api` `#phase4.1` `#refactor` `#viewer-api` `#log-viewer` `#context-http` `#frontend`
summary: Phase 4.1 — Refactor viewer-api and log-viewer into thin frontend-server layers that depend on context-http for all context-engine and log backend APIs
status: 🔧
---

# Plan: context-api Phase 4.1 — Viewer-Layer Refactoring

## Objective

Refactor `tools/viewer-api` and `tools/log-viewer` so that **all context-engine domain logic** (workspace management, graph operations, log parsing, log querying, tracing capture) is served exclusively by `tools/context-http` and `crates/context-api`.  The viewer tools become **thin frontend-server shells** that:

1. Depend on `context-http` (as a library) for the canonical REST/RPC API surface.
2. Retain **only** viewer-specific concerns: frontend serving (Vite dev proxy, static files), session management, source-file resolution, and any UI-specific presentation endpoints.
3. Compose a single `axum::Router` by **nesting** the `context-http` router under a shared prefix alongside their own viewer-specific routes.

This sets the stage for a frontend redesign where the Preact SPA talks to the unified `context-http` API for all graph and log operations, while the viewer shell provides the dev experience (HMR proxy, static serving, sessions).

## Context

### Prerequisites

- **Phase 4 complete** — `tools/context-http` exists with `POST /api/execute`, REST convenience endpoints, error mapping, and `AppState` wrapping `Arc<Mutex<WorkspaceManager>>`.
- **Phase 3.1 complete** — Log commands (`ListLogs`, `GetLog`, `QueryLog`, `AnalyzeLog`, `SearchLogs`, `DeleteLog`, `DeleteLogs`) are implemented in `crates/context-api` and accessible via `context-http`'s RPC endpoint.
- **`viewer-api`** is a shared library used by `log-viewer`, `doc-viewer`, and `context-http`.
- **`log-viewer`** is a full-stack tool (Rust backend + Preact frontend) that currently implements its own log-reading, parsing, searching, and querying handlers by directly calling `viewer_api::log_parser` and `viewer_api::query`.

### Interview / Plan References

- Phase 4 plan: `agents/plans/20260310_PLAN_CONTEXT_API_PHASE4.md`
- Phase 3.1 plan: `agents/plans/20260314_PLAN_CONTEXT_API_PHASE3_1.md`
- Overview: `agents/plans/20260310_PLAN_CONTEXT_API_OVERVIEW.md`

### Problem Statement

Today the architecture looks like this:

```
context-api  ←  viewer-api  ←  log-viewer   (log-viewer reimplements log API handlers)
                            ←  doc-viewer
                            ←  context-http  (canonical API, but not consumed by viewers)
```

**Duplication:**

| Capability | Where it lives today | Where it should live |
|---|---|---|
| Log file listing | `log-viewer/handlers.rs` `list_logs` | `context-api` `commands::logs::list_logs` via `context-http` |
| Log file reading + parsing | `log-viewer/handlers.rs` `get_log` | `context-api` `commands::logs::get_log` via `context-http` |
| Log searching (regex) | `log-viewer/handlers.rs` `search_log` | `context-api` (needs new command) via `context-http` |
| Log querying (JQ) | `log-viewer/handlers.rs` `query_log` | `context-api` `commands::logs::query_log` via `context-http` |
| Log analysis | `log-viewer/mcp_server.rs` `analyze_log` | `context-api` `commands::logs::analyze_log` via `context-http` |
| Cross-file log search | `log-viewer/mcp_server.rs` `search_all_logs` | `context-api` `commands::logs::search_logs` via `context-http` |
| Graph snapshot | not in log-viewer | `context-http` `GET /api/workspaces/:name/snapshot` |
| Graph statistics | not in log-viewer | `context-http` `GET /api/workspaces/:name/statistics` |
| Workspace management | not in log-viewer | `context-http` `POST /api/execute` |

**Viewer-only concerns** (should stay in the viewer layer):

| Capability | Location | Reason it's viewer-specific |
|---|---|---|
| Source file serving (local + GitHub) | `log-viewer/source.rs`, `viewer-api/source.rs` | Presentation concern — source code rendering in the frontend |
| Session management (verbose toggle, counters) | `log-viewer/state.rs`, `viewer-api/session.rs` | Per-browser-tab session state for UI preferences |
| Signature file serving | `log-viewer/handlers.rs` `get_signatures` | Debug tool artifact — not a context-api concept |
| Frontend serving (Vite proxy, static) | `log-viewer/router.rs`, `viewer-api/dev_proxy.rs` | Dev experience concern |
| Feature-flag scanning (has_graph_snapshot, etc.) | `log-viewer/handlers.rs` `list_logs` | UI hint metadata — could be added to `LogFileInfo` in context-api later |
| TypeScript type generation (`ts-rs`) | `log-viewer/types.rs` | Frontend build concern |
| MCP server (log-specific tools) | `log-viewer/mcp_server.rs` | To be migrated to context-mcp or kept as a viewer-level MCP |

### Key Decisions

1. **`context-http` becomes a library + binary crate** (already done in Phase 4) — viewer tools import it as a library to embed its router.

2. **`viewer-api` stays as shared infrastructure** but loses all domain re-exports — it keeps: `axum` re-export, `default_cors`, `with_static_files`, `init_tracing*`, `TracingConfig`, `ServerConfig`, `run_server`, `dev_proxy`, `session`, `source`, and `query` (the generic JQ engine). It no longer re-exports `log_parser`, `LogFileInfo`, `LogEntryInfo`, etc.

3. **`log-viewer` becomes a thin shell** — its handlers are replaced by either:
   - Forwarding to the embedded `context-http` router (for log/graph operations), or
   - Keeping a small set of viewer-specific handlers (source, signatures, sessions).

4. **The frontend is planned for redesign** — this plan does NOT rewrite the frontend, but it restructures the backend so the frontend CAN start talking to `context-http` endpoints. A compatibility shim layer maps old `/api/logs` routes to context-http commands during the transition.

5. **`doc-viewer` is not touched in this phase** — it uses `viewer-api` for sessions and query but has no log/context-graph concerns. It will be evaluated separately.

### Dependencies (External Crates)

No new external crates. This is purely a refactoring of internal dependency wiring.

### Files Affected

**Modified:**

| File | Change |
|---|---|
| `tools/viewer-api/Cargo.toml` | Remove `context-api` dependency |
| `tools/viewer-api/src/lib.rs` | Remove `log_parser`, `jq`, and context-api type re-exports |
| `tools/log-viewer/Cargo.toml` | Add `context-http` dependency, keep `viewer-api` for frontend infra |
| `tools/log-viewer/src/main.rs` | Construct `context_http::AppState` + compose routers |
| `tools/log-viewer/src/router.rs` | Nest context-http router, keep viewer-only routes |
| `tools/log-viewer/src/handlers.rs` | Remove log handlers (list, get, search, query) — replaced by context-http |
| `tools/log-viewer/src/state.rs` | Remove `parser: Arc<LogParser>`, add `context_state: context_http::AppState` |
| `tools/log-viewer/src/log_parser.rs` | Remove (was a re-export shim) |
| `tools/log-viewer/src/query.rs` | Remove (was a re-export shim) |
| `tools/log-viewer/src/types.rs` | Reduce to viewer-only types; log types come from context-api |
| `tools/log-viewer/src/mcp_server.rs` | Migrate log tools to call context-api directly |
| `tools/context-http/src/rest.rs` | Add log convenience REST endpoints |
| `tools/context-http/src/router.rs` | Add log REST routes |

**New:**

| File | Purpose |
|---|---|
| `tools/log-viewer/src/compat.rs` | Compatibility shim: maps legacy `/api/logs*` routes to context-http commands |
| `tools/context-http/src/log_rest.rs` | Log-specific REST convenience endpoints (list, get, search, query, analyze) |

**Deleted (contents moved upstream):**

| File | Replacement |
|---|---|
| `tools/log-viewer/src/log_parser.rs` | Direct import from `context_api::log_parser` where needed |
| `tools/log-viewer/src/query.rs` | Direct import from `context_api::jq` or `viewer_api::query` |

---

## Analysis

### Current Dependency Graph

```
context-trace  ←  context-api  ←  viewer-api  ←  log-viewer
                                              ←  doc-viewer
                                              ←  context-http
                               ←  context-mcp
                               ←  context-cli
```

### Target Dependency Graph

```
context-trace  ←  context-api  ←  context-http  ←  log-viewer (thin shell)
                                                ←  (future: doc-viewer)
                               ←  viewer-api    ←  log-viewer (frontend infra)
                                                ←  doc-viewer
                               ←  context-mcp
                               ←  context-cli
```

Key change: `viewer-api` no longer depends on `context-api`. It becomes a pure HTTP/frontend infrastructure library. Domain types flow through `context-http` instead.

### What log-viewer Currently Does vs. What It Should Do

#### Before (log-viewer owns everything)

```
Browser ──GET /api/logs──────► log-viewer::handlers::list_logs
                               ├── scan log_dir for .log files
                               ├── stat each file (size, modified)
                               ├── scan first 64KB for feature markers
                               └── return Vec<LogFileInfo>

Browser ──GET /api/logs/:name─► log-viewer::handlers::get_log
                               ├── read file from log_dir
                               ├── LogParser::parse()
                               └── return LogContentResponse

Browser ──GET /api/search/:name► log-viewer::handlers::search_log
                               ├── read + parse log file
                               ├── regex match across all fields
                               └── return SearchResponse

Browser ──GET /api/query/:name─► log-viewer::handlers::query_log
                               ├── read + parse log file
                               ├── JqFilter::compile + filter
                               └── return JqQueryResponse
```

#### After (log-viewer delegates to context-http)

```
Browser ──GET /api/logs──────► compat::list_logs
                               └── context_http::rpc::execute_command
                                   └── Command::ListLogs { workspace }

Browser ──GET /api/logs/:name─► compat::get_log
                               └── context_http::rpc::execute_command
                                   └── Command::GetLog { workspace, filename }

Browser ──GET /api/search/:name► compat::search_log
                               └── context_http::rpc::execute_command
                                   └── Command::SearchLogs { workspace, query }

Browser ──GET /api/query/:name─► compat::query_log
                               └── context_http::rpc::execute_command
                                   └── Command::QueryLog { workspace, filename, query }

Browser ──GET /api/source/*───► source::get_source  (unchanged, viewer-specific)
Browser ──GET /api/session────► session handler      (unchanged, viewer-specific)
Browser ──POST /api/execute───► context_http router  (new, direct access)
Browser ──GET /api/health─────► context_http router  (new)
Browser ──GET /api/workspaces─► context_http router  (new)
```

### Gap Analysis: What context-api is Missing

The log-viewer's `list_logs` handler enriches each file with **feature flags** scanned from the first 64KB of content:
- `has_graph_snapshot`
- `has_search_ops`
- `has_insert_ops`
- `has_search_paths`

These are not in `context-api`'s `LogFileInfo`. We have two options:

**Option A (recommended):** Add these fields to `context-api::types::LogFileInfo` and populate them in `commands::logs::list_logs`. This moves the logic upstream where it belongs.

**Option B:** Keep the enrichment in the viewer's compatibility layer. The compat handler calls `ListLogs` via context-http, then does a second pass to scan for features. This is simpler but perpetuates the split.

Similarly, the log-viewer's **regex search** (`search_log` in handlers.rs) is different from context-api's `SearchLogs` (which uses JQ). Options:

**Option A (recommended):** Add a `SearchLogsRegex` command to context-api that does regex-based search across log entry fields, matching the log-viewer's current behavior.

**Option B:** Keep the regex search as a viewer-level endpoint that reads the log file itself. Less clean but avoids expanding context-api for a possibly niche use case.

**Option C:** Convert the frontend to use JQ-based search (context-api's `SearchLogs`). This requires frontend changes but eliminates the regex handler entirely.

For this plan, we use **Option A for feature flags** and **Option C for regex search** (the compatibility layer translates regex queries into JQ where possible, and we document the migration path for the frontend).

### What `viewer-api` Loses and Keeps

| Module/Export | Keeps? | Reason |
|---|---|---|
| `pub use axum` | ✅ | All consumers need axum types |
| `pub use tokio` | ✅ | Async runtime re-export |
| `pub use tower_http` | ✅ | CORS, static files |
| `pub use tracing` | ✅ | Logging |
| `pub use tracing_appender` | ✅ | File logging |
| `pub use rmcp` | ✅ | MCP protocol |
| `TracingConfig`, `init_tracing*` | ✅ | Logging setup |
| `ServerConfig`, `run_server` | ✅ | Server lifecycle |
| `default_cors`, `with_static_files` | ✅ | Middleware |
| `display_host`, `to_unix_path` | ✅ | Utility |
| `ServerArgs` | ✅ | CLI arg parsing |
| `McpServerFactory` | ✅ | MCP integration |
| `pub mod dev_proxy` | ✅ | Vite HMR proxy |
| `pub mod session` | ✅ | Per-client sessions |
| `pub mod source` | ✅ | Source file serving |
| `pub mod query` | ✅ | Generic JQ filter engine |
| `pub use context_api::log_parser` | ❌ Remove | Domain concern → context-http |
| `pub use context_api::jq` | ❌ Remove | Domain concern → context-api direct |
| `pub use context_api::types::{Log*}` | ❌ Remove | Domain types → context-api direct |

Note: `viewer-api`'s `query` module and `context-api`'s `jq` module are **different implementations** of the same JQ concept but with different APIs. `viewer-api::query` wraps `jaq` and provides `JqFilter`, `filter_values`, `transform_values`. `context-api::jq` also wraps `jaq` with the same API shape. This duplication was introduced in Phase 3.1 when log commands were added to context-api. **This plan consolidates them** — `context-api::jq` becomes the canonical implementation, and `viewer-api::query` re-exports from it (or is replaced).

---

## Execution Steps

### Step 1: Extend context-api LogFileInfo with Feature Flags

Add the log-viewer's feature-flag fields to `context-api::types::LogFileInfo`:

```pseudo
pub struct LogFileInfo {
    pub filename: String,
    pub size: u64,
    pub modified: String,
    pub command: String,
    // New fields:
    pub has_graph_snapshot: bool,
    pub has_search_ops: bool,
    pub has_insert_ops: bool,
    pub has_search_paths: bool,
}
```

Update `commands::logs::list_logs` to scan the first 64KB of each file for the marker strings (same logic as `log-viewer/handlers.rs::list_logs`).

- [x] Modify `crates/context-api/src/types.rs` — add 4 boolean fields to `LogFileInfo`
- [x] Modify `crates/context-api/src/commands/logs.rs` — add feature-flag scanning to `list_logs`
- [x] Add tests for the new fields (`test_list_logs_feature_flags`, `test_list_logs_feature_flags_escaped_format`)
- [x] Verification: `cargo test -p context-api` passes (24/24 log tests pass)

---

### Step 2: Add Log REST Convenience Endpoints to context-http

Create `src/log_rest.rs` with REST endpoints that map to log commands, parallel to the existing REST endpoints for workspaces/atoms/etc:

```pseudo
// GET /api/workspaces/:name/logs
//   → Command::ListLogs { workspace, pattern: query.pattern, limit: query.limit }
pub async fn list_logs(...) -> Result<Json<Vec<LogFileInfo>>, HttpError>

// GET /api/workspaces/:name/logs/:filename
//   → Command::GetLog { workspace, filename, filter: query.filter, limit, offset }
pub async fn get_log(...) -> Result<Json<LogEntriesResponse>, HttpError>

// GET /api/workspaces/:name/logs/:filename/query
//   → Command::QueryLog { workspace, filename, query: query.jq, limit }
pub async fn query_log(...) -> Result<Json<LogQueryResponse>, HttpError>

// GET /api/workspaces/:name/logs/:filename/analysis
//   → Command::AnalyzeLog { workspace, filename }
pub async fn analyze_log(...) -> Result<Json<LogAnalysis>, HttpError>

// GET /api/workspaces/:name/logs/search
//   → Command::SearchLogs { workspace, query: query.jq, limit_per_file }
pub async fn search_logs(...) -> Result<Json<LogSearchResponse>, HttpError>

// DELETE /api/workspaces/:name/logs/:filename
//   → Command::DeleteLog { workspace, filename }
pub async fn delete_log(...) -> Result<Json<LogDeleteResult>, HttpError>

// DELETE /api/workspaces/:name/logs
//   → Command::DeleteLogs { workspace, older_than_days: query.older_than_days }
pub async fn delete_logs(...) -> Result<Json<LogDeleteResult>, HttpError>
```

Query parameter structs:

```pseudo
#[derive(Deserialize)]
pub struct ListLogsQuery {
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default = "default_100")]
    pub limit: usize,
}

#[derive(Deserialize)]
pub struct GetLogQuery {
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default = "default_100")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

#[derive(Deserialize)]
pub struct QueryLogQuery {
    pub jq: String,
    #[serde(default = "default_100")]
    pub limit: usize,
}

#[derive(Deserialize)]
pub struct SearchLogsQuery {
    pub jq: String,
    #[serde(default = "default_10")]
    pub limit_per_file: usize,
}

#[derive(Deserialize)]
pub struct DeleteLogsQuery {
    #[serde(default)]
    pub older_than_days: Option<u32>,
}
```

Wire these into `router.rs`:

```pseudo
.route("/api/workspaces/:name/logs", get(log_rest::list_logs).delete(log_rest::delete_logs))
.route("/api/workspaces/:name/logs/search", get(log_rest::search_logs))
.route("/api/workspaces/:name/logs/:filename", get(log_rest::get_log).delete(log_rest::delete_log))
.route("/api/workspaces/:name/logs/:filename/query", get(log_rest::query_log))
.route("/api/workspaces/:name/logs/:filename/analysis", get(log_rest::analyze_log))
```

- [x] Create `tools/context-http/src/log_rest.rs` (559 lines: 5 query structs, 3 response structs, 7 handlers, 9 unit tests)
- [x] Modify `tools/context-http/src/router.rs` — add log routes (`/search` before `/:filename` to avoid capture)
- [x] Modify `tools/context-http/src/lib.rs` — add `pub mod log_rest`
- [x] Add unit tests for query param deserialization (9 tests)
- [x] Add integration tests (15 tests: list empty/populated/filtered/limited, get/filter/paginate, query JQ, analyze, search, delete single/all, not-found, feature flags, RPC list/get) in `tests/http_integration.rs`
- [x] Verification: `cargo test -p context-http` passes (39 unit + 33 integration = 72 total, all pass)

**Note:** axum-test 14 requires `add_query_param()` for query parameters — inline `?key=value` in the URL path causes 404s.

---

### Step 3: Remove Domain Re-exports from viewer-api

Strip `viewer-api` of all `context-api` domain type re-exports. It should be a **pure infrastructure** crate.

**Changes to `viewer-api/Cargo.toml`:**
- Remove `context-api` from `[dependencies]`
- The `jq` query module stays because it depends on `jaq-*` crates directly (its own implementation), NOT on `context-api::jq`

**Changes to `viewer-api/src/lib.rs`:**
- Remove the `pub use context_api::{ jq, log_parser, types::{ ... } }` block
- Remove any `use context_api::*` imports

This is a **breaking change** for downstream consumers. `log-viewer` and `context-http` will need to update their imports.

- [ ] Modify `tools/viewer-api/Cargo.toml` — remove `context-api` dependency
- [ ] Modify `tools/viewer-api/src/lib.rs` — remove all `context_api` re-exports
- [ ] Verification: `cargo check -p viewer-api` passes
- [ ] Note which downstream crates break (expected: `log-viewer`, `context-http`)

---

### Step 4: Update context-http Imports

After Step 3, `context-http` can no longer access `context_api` types via `viewer_api::*` re-exports. It already imports `context-api` directly in most places, but verify and fix any broken imports.

- [ ] Fix any broken imports in `tools/context-http/src/*.rs`
- [ ] Verification: `cargo check -p context-http` passes
- [ ] Verification: `cargo test -p context-http` passes

---

### Step 5: Restructure log-viewer State

Replace the log-viewer's `AppState` so it **embeds** a `context_http::AppState` instead of owning a `LogParser` and `log_dir` directly.

**Before:**

```pseudo
pub struct AppState {
    pub log_dir: PathBuf,
    pub signatures_dir: PathBuf,
    pub workspace_root: PathBuf,
    pub parser: Arc<LogParser>,
    pub source_backend: SourceBackend,
    pub sessions: SessionStore,
}
```

**After:**

```pseudo
pub struct AppState {
    /// Embedded context-http state for domain operations.
    pub context: context_http::state::AppState,

    /// Signatures directory (viewer-specific: debug function signatures).
    pub signatures_dir: PathBuf,

    /// Source backend for serving source files (viewer-specific).
    pub source_backend: SourceBackend,

    /// Per-client session store (viewer-specific).
    pub sessions: SessionStore,

    /// The workspace name to use for log operations.
    /// In the current log-viewer, all logs come from a single "default"
    /// workspace directory. This maps to the context-http workspace concept.
    pub default_workspace: String,
}
```

The `log_dir` and `parser` fields are removed — log operations now go through `context_http::AppState.manager` → `WorkspaceManager` → log commands.

**Migration for `default_workspace`:** The current log-viewer reads logs from a flat directory (`target/test-logs`). The context-api model expects logs under `<workspace_dir>/logs/`. During the transition, we either:
- Create a "default" workspace whose log dir is symlinked/configured to the old location, or
- Add a `CONTEXT_HTTP_LOG_DIR` env var override that `WorkspaceManager::log_dir()` respects.

For this plan, we use the simpler approach: the log-viewer creates a workspace named after the project (or "default") and configures the `WorkspaceManager`'s base dir so that the workspace's log directory matches the configured `log_dir`.

- [ ] Modify `tools/log-viewer/src/state.rs` — new `AppState` with embedded context state
- [ ] Modify `tools/log-viewer/Cargo.toml` — add `context-http` and `context-api` dependencies
- [ ] Update `create_app_state_from_config` to construct both states
- [ ] Verification: `cargo check -p log-viewer` (will still fail until handlers are updated)

---

### Step 6: Create Compatibility Shim

Create `tools/log-viewer/src/compat.rs` — a set of handlers that accept the **old** log-viewer request format (query params, path structure) and translate them into `context-http` commands.

```pseudo
/// GET /api/logs → ListLogs command
pub async fn list_logs(
    State(state): State<AppState>,
) -> Result<Json<Vec<CompatLogFileInfo>>, ...> {
    let manager = state.context.manager.clone();
    let result = spawn_blocking(move || {
        let mut mgr = manager.lock()?;
        execute(&mut mgr, Command::ListLogs {
            workspace: state.default_workspace.clone(),
            pattern: None,
            limit: 1000,
        })
    }).await??;
    // Map CommandResult::LogList → Vec<CompatLogFileInfo>
    // (CompatLogFileInfo matches the old log-viewer types.ts format
    //  with `name` instead of `filename`, etc.)
}

/// GET /api/logs/:name → GetLog command
pub async fn get_log(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<CompatLogContentResponse>, ...> {
    // Validate filename, then:
    let result = execute Command::GetLog { workspace, filename: name, ... }
    // Map CommandResult::LogEntries → CompatLogContentResponse
}

/// GET /api/search/:name → regex search
/// NOTE: context-api uses JQ, not regex. The compat layer translates
/// simple regex patterns to JQ `test()` expressions where possible.
pub async fn search_log(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<CompatSearchResponse>, ...> {
    // Convert regex to JQ: `.message | test("pattern"; "i")`
    // Or fall back to local regex search if JQ translation is not feasible
}

/// GET /api/query/:name → QueryLog command
pub async fn query_log(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(params): Query<JqQuery>,
) -> Result<Json<CompatJqQueryResponse>, ...> {
    let result = execute Command::QueryLog { workspace, filename: name, query: params.jq, ... }
    // Map CommandResult::LogQueryResult → CompatJqQueryResponse
}
```

The `Compat*` types mirror the existing `log-viewer/types.rs` response shapes to avoid breaking the frontend. Over time, the frontend migrates to the canonical context-http response shapes.

- [ ] Create `tools/log-viewer/src/compat.rs`
- [ ] Add `Compat*` response types that match the current frontend contract
- [ ] Implement `list_logs`, `get_log`, `query_log`, `search_log` compat handlers
- [ ] Add unit tests for the regex → JQ translation
- [ ] Verification: compat handlers compile

---

### Step 7: Rewire log-viewer Router

Replace the handler references in `router.rs` to use the compatibility shim for legacy routes, and **nest** the full context-http router for new endpoints.

**Before:**

```pseudo
Router::new()
    .route("/api/logs", get(handlers::list_logs))
    .route("/api/logs/:name", get(handlers::get_log))
    .route("/api/signatures/:name", get(handlers::get_signatures))
    .route("/api/search/:name", get(handlers::search_log))
    .route("/api/query/:name", get(handlers::query_log))
    .route("/api/source/*path", get(source::get_source))
    .route("/api/session", get(handlers::get_session).post(handlers::update_session))
```

**After:**

```pseudo
// Viewer-specific routes
let viewer_routes = Router::new()
    .route("/api/signatures/:name", get(handlers::get_signatures))
    .route("/api/source/*path", get(source::get_source))
    .route("/api/session", get(handlers::get_session).post(handlers::update_session));

// Legacy compatibility routes (translate old format → context-http commands)
let compat_routes = Router::new()
    .route("/api/logs", get(compat::list_logs))
    .route("/api/logs/:name", get(compat::get_log))
    .route("/api/search/:name", get(compat::search_log))
    .route("/api/query/:name", get(compat::query_log));

// Canonical context-http routes (new endpoints available to the frontend)
let context_router = context_http::router::create_router(
    state.context.clone(),
    None, // no static files — viewer handles that
);

Router::new()
    .merge(viewer_routes.with_state(state.clone()))
    .merge(compat_routes.with_state(state.clone()))
    .merge(context_router)
    .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any))
    // ... frontend mode
```

This means the log-viewer server serves **both** the old `/api/logs*` routes (for frontend compatibility) **and** the new `/api/execute`, `/api/workspaces/*` routes (for the redesigned frontend). The old routes are deprecated.

- [ ] Modify `tools/log-viewer/src/router.rs` — compose routers
- [ ] Modify `tools/log-viewer/src/main.rs` — construct context_http::AppState and pass to router
- [ ] Verification: `cargo check -p log-viewer` passes

---

### Step 8: Clean Up log-viewer Handlers

Remove the log-parsing and log-querying logic from `handlers.rs`. Keep only:
- `get_signatures` — viewer-specific
- `get_session` / `update_session` — viewer-specific (or move to a `session_handlers.rs`)

Delete the re-export shim modules:
- `log_parser.rs` — no longer needed; compat.rs uses context-api directly
- `query.rs` — no longer needed; compat.rs uses context-api directly

- [ ] Modify `tools/log-viewer/src/handlers.rs` — remove `list_logs`, `get_log`, `search_log`, `query_log`
- [ ] Delete `tools/log-viewer/src/log_parser.rs`
- [ ] Delete `tools/log-viewer/src/query.rs`
- [ ] Update `tools/log-viewer/src/main.rs` — remove `mod log_parser`, `mod query`
- [ ] Verification: `cargo check -p log-viewer` passes

---

### Step 9: Migrate log-viewer MCP Server

The log-viewer's MCP server (`mcp_server.rs`) currently has 6 tools that directly read log files. Migrate them to call context-api commands instead:

| MCP Tool | Current Implementation | New Implementation |
|---|---|---|
| `list_logs` | Direct file scan | `commands::execute(&mut mgr, Command::ListLogs { ... })` |
| `get_log` | Direct parse | `commands::execute(&mut mgr, Command::GetLog { ... })` |
| `query_logs` | Direct JQ filter | `commands::execute(&mut mgr, Command::QueryLog { ... })` |
| `analyze_log` | Direct analysis | `commands::execute(&mut mgr, Command::AnalyzeLog { ... })` |
| `search_all_logs` | Direct cross-file search | `commands::execute(&mut mgr, Command::SearchLogs { ... })` |
| `get_source` | Direct file read | **Keep as-is** (viewer-specific) |

The MCP server needs access to a `WorkspaceManager`. Options:
- **Option A:** Pass an `Arc<Mutex<WorkspaceManager>>` to the MCP server (shared with context-http's AppState).
- **Option B:** Have the MCP server call context-http's AppState directly.

Use **Option A** — the MCP server receives the same `Arc<Mutex<WorkspaceManager>>` from the log-viewer's `AppState.context.manager`.

- [ ] Modify `tools/log-viewer/src/mcp_server.rs` — replace direct file operations with `commands::execute` calls
- [ ] Update `LogServer` struct to hold `Arc<Mutex<WorkspaceManager>>`
- [ ] Update `run_mcp_server` to accept the workspace manager
- [ ] Verification: `cargo check -p log-viewer` passes

---

### Step 10: Update log-viewer Types

Reduce `types.rs` to viewer-specific types only. The canonical log types now live in `context-api::types` and are consumed from there.

**Keep:**
- `SourceQuery` — viewer-specific (source endpoint query params)
- `ErrorResponse` — viewer-level error format
- `SessionConfigUpdate` — viewer-specific

**Remove or replace with re-exports:**
- `LogFileInfo` → use `context_api::types::LogFileInfo` (with new feature-flag fields)
- `LogContentResponse` → replaced by `CommandResult::LogEntries` response format
- `SearchResponse` → replaced by `CommandResult::LogSearchResult`
- `JqQueryResponse` → replaced by `CommandResult::LogQueryResult`
- `SearchQuery` → move to `compat.rs` (only needed there)
- `JqQuery` → move to `compat.rs`

The `Compat*` types in `compat.rs` handle the translation between old frontend-expected shapes and the new canonical shapes.

- [ ] Modify `tools/log-viewer/src/types.rs` — remove log types, keep viewer-specific types
- [ ] Move query param types to `compat.rs`
- [ ] Verification: `cargo check -p log-viewer` passes

---

### Step 11: Integration Tests

Verify the full stack works end-to-end:

1. **context-http log REST endpoints** — new integration tests: ✅ (completed in Step 2)
   - `rest_list_logs_empty` — fresh workspace has no logs ✅
   - `rest_list_logs_after_write` — list logs after writing a file ✅
   - `rest_list_logs_with_pattern_and_limit` — pattern filter and limit ✅
   - `rest_get_log_entries` — retrieve log entries ✅
   - `rest_get_log_with_filter_and_pagination` — level filter + offset/limit ✅
   - `rest_get_log_not_found` — 404 for missing file ✅
   - `rest_query_log_with_jq` — JQ filter on log entries ✅
   - `rest_analyze_log` — analyze a log file ✅
   - `rest_search_logs` — cross-file JQ search ✅
   - `rest_delete_log` — delete single and verify gone ✅
   - `rest_delete_logs_all` — delete all and verify counts ✅
   - `rest_delete_log_not_found` — 404 for missing file ✅
   - `rest_list_logs_feature_flags` — feature flag fields populated ✅
   - `rpc_list_logs` — list_logs via POST /api/execute ✅
   - `rpc_get_log` — get_log via POST /api/execute ✅

2. **log-viewer compat endpoints** — verify old routes still work:
   - `test_compat_list_logs` — `GET /api/logs` returns expected format
   - `test_compat_get_log` — `GET /api/logs/:name` returns expected format
   - `test_compat_query_log` — `GET /api/query/:name?jq=...` works
   - `test_viewer_source_still_works` — source endpoint unchanged
   - `test_viewer_signatures_still_works` — signatures endpoint unchanged

3. **Router composition** — verify both old and new endpoints coexist:
   - `test_context_http_routes_accessible` — `POST /api/execute` works through log-viewer
   - `test_rest_workspaces_accessible` — `GET /api/workspaces` works through log-viewer
   - `test_health_accessible` — `GET /api/health` works through log-viewer

- [x] Add context-http log REST integration tests (15 tests, all pass)
- [ ] Add log-viewer compat integration tests
- [ ] Add router composition tests
- [x] Verification: `cargo test -p context-http` passes (72 total tests)
- [ ] Verification: `cargo test -p log-viewer` passes

---

### Step 12: Documentation

Update documentation to reflect the new architecture:

- [ ] Update `tools/context-http/README.md` — add log REST endpoints
- [ ] Update `tools/log-viewer/README.md` — document the layered architecture, compat routes, and migration path
- [ ] Add deprecation notices to log-viewer's compat routes (doc comments explaining they will be removed when frontend migrates)
- [ ] Update `tools/viewer-api/README.md` (if exists) — document the infrastructure-only scope

---

### Step 13: Final Verification

- [ ] `cargo check --workspace` — no errors
- [ ] `cargo test -p context-api` — all pass
- [ ] `cargo test -p context-http` — all pass (old + new)
- [ ] `cargo test -p viewer-api` — all pass
- [ ] `cargo test -p log-viewer` — all pass
- [ ] `cargo test -p doc-viewer` — all pass (should be unaffected)
- [ ] Manual test: start `log-viewer`, verify frontend loads, verify `/api/logs` returns data
- [ ] Manual test: verify `POST /api/execute` works through log-viewer server
- [ ] Manual test: verify `GET /api/workspaces` works through log-viewer server

---

## API Reference

### New context-http REST Endpoints (Step 2)

| Method | Path | Query Params | Response |
|--------|------|-------------|----------|
| GET | `/api/workspaces/:name/logs` | `pattern?`, `limit?` | `Vec<LogFileInfo>` |
| GET | `/api/workspaces/:name/logs/search` | `jq`, `limit_per_file?` | `LogSearchResponse` |
| GET | `/api/workspaces/:name/logs/:filename` | `filter?`, `limit?`, `offset?` | `LogEntriesResponse` |
| GET | `/api/workspaces/:name/logs/:filename/query` | `jq`, `limit?` | `LogQueryResponse` |
| GET | `/api/workspaces/:name/logs/:filename/analysis` | — | `LogAnalysis` |
| DELETE | `/api/workspaces/:name/logs/:filename` | — | `LogDeleteResult` |
| DELETE | `/api/workspaces/:name/logs` | `older_than_days?` | `LogDeleteResult` |

### Legacy Compat Endpoints (Step 6, log-viewer only)

| Method | Path | Maps To |
|--------|------|---------|
| GET | `/api/logs` | `Command::ListLogs` with default workspace |
| GET | `/api/logs/:name` | `Command::GetLog` with default workspace |
| GET | `/api/search/:name` | `Command::SearchLogs` (regex→JQ translation) |
| GET | `/api/query/:name` | `Command::QueryLog` with default workspace |

These are **deprecated** and will be removed once the frontend is migrated to the canonical context-http endpoints.

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Frontend breaks due to response shape changes | High | High | Compat layer (`compat.rs`) preserves exact response shapes. No frontend changes required in this phase. |
| `viewer-api` losing `context-api` dep breaks `doc-viewer` | Low | Medium | `doc-viewer` doesn't import any `context-api` types from `viewer-api` — it only uses sessions, query, and tracing. Verify before landing Step 3. |
| Workspace/log-dir path mismatch between old log-viewer config and context-api model | Medium | Medium | The compat layer configures WorkspaceManager's base dir so that the default workspace's log directory matches the old `log_dir` config path. |
| Performance regression from extra indirection (compat → command → execute) | Low | Low | The execute path adds one function call and a Mutex lock — negligible compared to file I/O and parsing. |
| JQ translation of regex search queries is lossy | Medium | Low | Document known limitations. Fall back to local regex search in compat layer when JQ translation fails. Plan frontend migration to JQ as Phase 4.2 work. |
| Circular dependency between context-http and viewer-api | Low | High | Verify: viewer-api does NOT depend on context-http. The dependency is one-way: log-viewer → {context-http, viewer-api}. |
| Test coverage gap during migration | Medium | Medium | Run full test suite after each step. Add new integration tests in Step 11 before removing old code. |

---

## Future Work (Not in This Phase)

### Phase 4.2 — Frontend Migration (Planned)

Once this refactoring is complete, the frontend can be incrementally migrated:

1. **New API client** — add a `contextHttpClient` alongside the existing `logViewerClient` in the frontend's `api/` module.
2. **Feature by feature** — migrate log listing to `GET /api/workspaces/:ws/logs`, log reading to `GET /api/workspaces/:ws/logs/:file`, etc.
3. **New panels** — add workspace management (create/open/save), atom/pattern editing, graph manipulation panels that talk to `POST /api/execute`.
4. **Graph visualization** — the existing `HypergraphView` component can be enhanced to use `GET /api/workspaces/:ws/snapshot` for live graph data.
5. **Remove compat routes** — once the frontend no longer calls `/api/logs*`, delete `compat.rs` and the legacy routes.

### Phase 4.3 — Unified Viewer (Speculative)

Merge `log-viewer` and `doc-viewer` into a single `context-viewer` application:
- One Axum server serving one SPA
- Tabs/panels for: logs, graph visualization, documentation, source code
- All backed by `context-http` for domain operations
- `viewer-api` becomes the shared dev-experience library

---

## Notes

### Relationship Between JQ Implementations

There are currently **two** JQ wrapper modules:

1. **`viewer-api/src/query.rs`** — uses `jaq-core`, `jaq-std`, `jaq-interpret`, `jaq-syn`. Provides `JqFilter`, `filter_values`, `transform_values`.
2. **`context-api/src/jq.rs`** — same dependencies, same API shape. Feature-gated behind `jq`.

These should be consolidated. The recommended approach:
- Keep `context-api::jq` as the canonical implementation (it's the lower-level crate).
- Change `viewer-api::query` to re-export from `context-api::jq` if `viewer-api` still has `context-api` as a dependency.
- OR after Step 3 (removing `context-api` from `viewer-api`), keep `viewer-api::query` as a standalone JQ wrapper for non-domain use (doc-viewer uses it for doc querying, which is not a context-api concern).

For this phase, we keep both implementations — consolidation can happen as a follow-up cleanup.

### Ordering Rationale

The steps are ordered to minimize breakage windows:

1. **Steps 1–2** (extend context-api/context-http) are purely additive — nothing breaks.
2. **Step 3** (strip viewer-api) is the breaking change — done in one step with Steps 4–8 fixing the fallout.
3. **Steps 5–8** (restructure log-viewer) form one logical unit — they should be landed together.
4. **Step 9** (MCP migration) is independent and can be done in parallel with Steps 6–8.
5. **Steps 10–13** are cleanup and verification.

### Deviations from Plan

- **Step 2 integration tests done early:** Rather than waiting for Step 11, integration tests for the log REST endpoints were added immediately in Step 2. This provides faster feedback and matches the additive nature of Steps 1–2.
- **axum-test query parameter handling:** Discovered that `axum-test` v14 does not support inline query strings in the URL (e.g., `/path?key=value` returns 404). Must use `.add_query_param("key", "value")` builder method instead.
- **`#[serde(default)]` on feature flags:** Added `#[serde(default)]` to all 4 boolean fields in `LogFileInfo` to maintain backward compatibility with existing serialized data that lacks these fields.
- **delete_log returns CommandResult::Ok:** The `delete_log` REST handler returns an empty JSON `{}` body (from `CommandResult::Ok`) rather than a `LogDeleteResult`, since the single-file delete command returns `()` at the domain level. The `delete_logs` (bulk) endpoint does return `LogDeleteResult` with count and freed bytes.

### Lessons Learned
*(To be filled after execution)*