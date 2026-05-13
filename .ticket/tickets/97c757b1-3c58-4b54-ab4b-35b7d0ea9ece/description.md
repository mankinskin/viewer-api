---
tags: `#plan` `#refactoring` `#tools` `#viewer-api`
summary: Extract shared server infrastructure from viewer-api into a reusable library for log-viewer and doc-viewer.
status: 📋
---

# Viewer API Refactoring Plan

## Objective
Extract shared server infrastructure from viewer-api into a reusable library that both log-viewer and doc-viewer can use.

## Current State
- **viewer-api**: Full log-viewer implementation (renamed from log-viewer)
- **log-viewer**: Copy of viewer-api for logs (to be refactored)
- **doc-viewer**: Full doc-viewer with separate server infrastructure

## Target Architecture

```
viewer-api/          # Shared library
├── src/
│   └── lib.rs       # Server infrastructure, utilities
│
log-viewer/          # Log viewing tool
├── src/
│   ├── main.rs      # Uses viewer-api, defines routes
│   ├── log_parser.rs
│   ├── query.rs
│   └── mcp_server.rs
│
doc-viewer/          # Doc viewing tool
├── src/
│   ├── main.rs      # Uses viewer-api, defines routes
│   ├── http.rs      # HTTP handlers
│   ├── tools/       # Doc management logic
│   └── ...          # Domain-specific modules
```

## Shared Components (viewer-api/lib.rs)

1. **Server Runner**
   - Parse --http and --mcp flags
   - Start HTTP server, MCP server, or both
   - Handle tokio runtime

2. **HTTP Infrastructure**
   - CORS layer setup
   - Static file serving
   - Common error types

3. **Utilities**
   - `to_unix_path()` - Path normalization
   - Logging/tracing initialization

## API Design

```rust
// viewer-api/src/lib.rs

/// Configuration for the server
pub struct ServerConfig {
    pub name: String,
    pub default_port: u16,
    pub static_dir: Option<PathBuf>,
}

/// Run the server with given HTTP routes and MCP handler
pub async fn run_server<S, M>(
    config: ServerConfig,
    http_state: S,
    http_routes: fn(S) -> Router,
    mcp_handler: Option<M>,
) -> Result<(), Box<dyn Error>>
where
    S: Clone + Send + Sync + 'static,
    M: McpHandler + Send + 'static,
{
    // Parse args, start appropriate servers
}

/// Default CORS layer for development
pub fn default_cors() -> CorsLayer { ... }

/// Serve static files with fallback
pub fn static_service(dir: PathBuf) -> impl Service { ... }
```

## Execution Steps

### Step 1: Create viewer-api library
- [x] Read current viewer-api structure
- [x] Create lib.rs with shared infrastructure
- [x] Remove log-specific code from viewer-api
- [x] Update Cargo.toml to be a library

### Step 2: Update log-viewer
- [x] Add viewer-api as dependency
- [x] Simplify main.rs to use viewer-api (kept existing code, uses viewer-api as dependency)
- [x] Keep log_parser.rs, query.rs, mcp_server.rs

### Step 3: Update doc-viewer
- [x] Add viewer-api as dependency
- [x] Simplify main.rs to use viewer-api (kept existing code, uses viewer-api as dependency)
- [x] Keep http.rs, tools/, schema.rs

## Validation
- [x] `cargo build -p viewer-api`
- [x] `cargo run` in log-viewer starts on port 3000
- [x] `cargo run` in doc-viewer starts on port 3001
- [ ] `cargo run -- --mcp` works for both

## Notes
- Keep each tool self-contained with its own Cargo.toml
- viewer-api should have minimal dependencies
- Tools can extend the shared infrastructure as needed
