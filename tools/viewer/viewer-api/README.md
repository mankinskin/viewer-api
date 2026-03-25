# Viewer API

A shared library providing server infrastructure for viewer applications like `log-viewer` and `doc-viewer`.

## Features

- **Server Configuration**: Common configuration with `ServerConfig`
- **Argument Parsing**: Parse `--http` and `--mcp` flags via `ServerArgs`
- **HTTP Infrastructure**: CORS setup, static file serving
- **Utilities**: Path normalization (`to_unix_path`)
- **Re-exports**: `axum`, `tower_http`, `tokio`, `tracing`, `rmcp`

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
viewer-api = { path = "../viewer-api" }
```

### Basic Example

```rust
use viewer_api::{ServerConfig, ServerArgs, init_tracing, default_cors, with_static_files};
use axum::{Router, routing::get};
use std::path::PathBuf;

#[derive(Clone)]
struct AppState {
    // your state
}

fn create_router(state: AppState, static_dir: Option<PathBuf>) -> Router {
    let router = Router::new()
        .route("/api/hello", get(|| async { "Hello!" }))
        .layer(default_cors())
        .with_state(state);
    
    with_static_files(router, static_dir)
}

#[tokio::main]
async fn main() {
    let args = ServerArgs::parse();
    init_tracing("info");
    
    eprintln!("Server starting in {} mode", args.mode_str());
    
    let config = ServerConfig::new("my-server", 3000)
        .with_static_dir(PathBuf::from("static"));
    
    // Run your server based on args.http / args.mcp
}
```

## API

### `ServerConfig`

Configuration for the server:

```rust
let config = ServerConfig::new("server-name", 3000)
    .with_host("0.0.0.0")
    .with_static_dir(PathBuf::from("static"))
    .with_workspace_root(PathBuf::from(".."));
```

### `ServerArgs`

Parse command-line arguments:

```rust
let args = ServerArgs::parse();
// args.http - true if --http flag present (default if no flags)
// args.mcp - true if --mcp flag present
```

### Utilities

- `to_unix_path(path)` - Convert path to Unix-style (forward slashes)
- `init_tracing(level)` - Initialize tracing with the given log level
- `default_cors()` - Create a permissive CORS layer for development
- `with_static_files(router, dir)` - Add static file serving to a router

## Dependencies

This library re-exports commonly used crates:
- `axum` - Web framework
- `tower_http` - HTTP middleware (CORS, static files)
- `tokio` - Async runtime
- `tracing` - Logging
- `rmcp` - MCP protocol

## Frontend Package

The `frontend/` directory contains shared frontend components and styles for viewer applications.

### Components

- **TreeView** - Expandable tree component with folder/file/doc icons
- **Spinner** - Loading spinner in sm/md/lg sizes

### Styles

Import shared styles in your frontend:

```typescript
import '@context-engine/viewer-api-frontend/styles';
```

Or individual CSS files:

```css
@import '@context-engine/viewer-api-frontend/styles/variables.css';
@import '@context-engine/viewer-api-frontend/styles/base.css';
```

### Usage

Add to your frontend's `package.json`:

```json
{
  "dependencies": {
    "@context-engine/viewer-api-frontend": "file:../../viewer-api/frontend"
  }
}
```

```typescript
import { TreeView, Spinner } from '@context-engine/viewer-api-frontend';
```
