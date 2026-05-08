//! Viewer API - Shared server infrastructure for viewer tools.
//!
//! This library provides common HTTP and MCP server infrastructure for
//! building viewer applications like log-viewer and doc-viewer.
//!
//! # Features
//!
//! - HTTP server with CORS and static file serving
//! - MCP server support via rmcp
//! - Command-line flag parsing (--http, --mcp)
//! - Tracing/logging initialization (console and file)
//! - Dev proxy to Vite dev server (--dev mode)
//! - Common utilities
//!
//! # Example
//!
//! ```rust,no_run
//! use viewer_api::{ServerConfig, run_server, McpServerFactory};
//! use axum::Router;
//! use std::path::PathBuf;
//!
//! #[derive(Clone)]
//! struct MyState;
//!
//! fn create_routes(state: MyState, _static_dir: Option<PathBuf>) -> Router {
//!     Router::new().with_state(state)
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = ServerConfig::new("my-viewer", 3000);
//!     let state = MyState;
//!
//!     run_server(config, state, create_routes, None::<McpServerFactory<MyState>>).await.unwrap();
//! }
//! ```

pub mod dev_proxy;
pub mod client_log;

// New shared primitives for ticket-viewer and future viewer tools
pub mod auth;
pub mod error;
pub mod middleware;
pub mod pagination;
mod runtime;
pub mod sse;

// Session management module
pub mod query;
pub mod session;
pub mod source;

// Re-export commonly used types
pub use axum;
pub use rmcp;
pub use tokio;
pub use tower_http;
pub use tracing;
pub use tracing_appender;
pub use runtime::{
    McpServerFactory,
    ServerArgs,
    ServerConfig,
    TracingConfig,
    default_cors,
    display_host,
    init_tracing,
    init_tracing_full,
    run_server,
    shutdown_signal,
    with_static_files,
};

// Re-export domain types from context-api for downstream consumers
// (log-viewer, doc-viewer, etc.)
pub use context_api::{
    jq,
    log_parser,
    types::{
        LogAnalysis,
        LogDeleteResult,
        LogEntryInfo,
        LogFileInfo,
        LogFileSearchResult,
        SpanSummary,
        TraceSummary,
    },
};

/// Convert a path to Unix-style string (forward slashes)
pub fn to_unix_path(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_unix_path() {
        let path = std::path::Path::new("C:\\Users\\test\\file.txt");
        assert_eq!(to_unix_path(path), "C:/Users/test/file.txt");
    }

    #[test]
    fn test_server_config() {
        let config = ServerConfig::new("test", 3000)
            .with_host("0.0.0.0")
            .with_static_dir(std::path::PathBuf::from("/static"));

        assert_eq!(config.name, "test");
        assert_eq!(config.default_port, 3000);
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(
            config.static_dir,
            Some(std::path::PathBuf::from("/static"))
        );
    }
}
