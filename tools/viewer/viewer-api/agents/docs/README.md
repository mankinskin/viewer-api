# Viewer API

Shared server infrastructure for viewer tools in the context-engine project.

## Overview

Viewer API provides common infrastructure used by both `log-viewer` and `doc-viewer`:

- HTTP server with CORS and static file serving
- MCP server support via rmcp
- Tracing/logging initialization
- Command-line argument parsing
- Common utilities

## Usage

```rust
use viewer_api::{ServerConfig, run_server, McpServerFactory};
use axum::Router;
use std::path::PathBuf;

#[derive(Clone)]
struct MyState;

fn create_router(state: MyState, _static_dir: Option<PathBuf>) -> Router {
    Router::new().with_state(state)
}

#[tokio::main]
async fn main() {
    let config = ServerConfig::new("my-viewer", 3000);
    let state = MyState;
    
    run_server(config, state, create_router, None::<McpServerFactory<MyState>>).await.unwrap();
}
```

## Key Types

### ServerConfig

Configuration for the HTTP server:
- `name`: Server name (for logs)
- `default_port`: Default HTTP port
- `host`: Bind address (default: 127.0.0.1)
- `static_dir`: Optional static files directory
- `workspace_root`: Optional workspace root

### ServerArgs

Parsed command-line arguments:
- `--http`: Run HTTP server
- `--mcp`: Run MCP server
- Default: HTTP only if no flags

### TracingConfig

Logging configuration:
- `level`: Log level (trace/debug/info/warn/error)
- `file_logging`: Enable file output
- `log_dir`: Directory for log files
- `log_file_prefix`: Prefix for log filenames

## Utilities

| Function | Description |
|----------|-------------|
| `to_unix_path()` | Convert path to Unix-style |
| `display_host()` | Convert 0.0.0.0 to localhost |
| `init_tracing_full()` | Initialize tracing subsystem |
