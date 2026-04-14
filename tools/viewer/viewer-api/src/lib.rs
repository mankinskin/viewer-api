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

// New shared primitives for ticket-viewer and future viewer tools
pub mod auth;
pub mod error;
pub mod middleware;
pub mod pagination;
pub mod sse;

use axum::Router;
use std::{
    env,
    future::Future,
    path::PathBuf,
    pin::Pin,
};
use tower_http::{
    cors::{
        Any,
        CorsLayer,
    },
    services::{ServeDir, ServeFile},
};
use tracing::{
    error,
    info,
};
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

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

/// Tracing configuration
#[derive(Clone, Debug)]
pub struct TracingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Enable file logging
    pub file_logging: bool,
    /// Directory for log files (if file_logging is true)
    pub log_dir: Option<PathBuf>,
    /// Log file name prefix
    pub log_file_prefix: String,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_logging: false,
            log_dir: None,
            log_file_prefix: "app".to_string(),
        }
    }
}

impl TracingConfig {
    /// Create config from environment variables.
    ///
    /// Reads LOG_LEVEL and LOG_FILE environment variables.
    pub fn from_env(
        log_file_prefix: impl Into<String>,
        default_log_dir: PathBuf,
    ) -> Self {
        let level =
            env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
        let file_logging = env::var("LOG_FILE").is_ok();

        Self {
            level,
            file_logging,
            log_dir: Some(default_log_dir),
            log_file_prefix: log_file_prefix.into(),
        }
    }

    /// Set log level
    pub fn with_level(
        mut self,
        level: impl Into<String>,
    ) -> Self {
        self.level = level.into();
        self
    }

    /// Enable file logging
    pub fn with_file_logging(
        mut self,
        log_dir: PathBuf,
        prefix: impl Into<String>,
    ) -> Self {
        self.file_logging = true;
        self.log_dir = Some(log_dir);
        self.log_file_prefix = prefix.into();
        self
    }
}

/// Initialize tracing with optional file output.
///
/// This is the recommended way to initialize tracing for viewer tools.
/// It supports both console and file logging based on the configuration.
pub fn init_tracing_full(config: &TracingConfig) {
    let filter = EnvFilter::try_new(&config.level)
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    // Check if file logging is enabled
    if config.file_logging {
        let log_dir = config
            .log_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("logs"));
        std::fs::create_dir_all(&log_dir).ok();

        let log_file_name = format!("{}.log", config.log_file_prefix);
        let file_appender =
            tracing_appender::rolling::daily(&log_dir, &log_file_name);
        let (non_blocking, _guard) =
            tracing_appender::non_blocking(file_appender);

        // Store the guard to keep the appender alive
        std::mem::forget(_guard);

        let file_layer = fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true);

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .with(file_layer)
            .init();

        info!(
            "File logging enabled to {}/{}",
            to_unix_path(&log_dir),
            log_file_name
        );
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .init();
    }
}

/// Server configuration
#[derive(Clone, Debug)]
pub struct ServerConfig {
    /// Server name (used in logs)
    pub name: String,
    /// Default HTTP port
    pub default_port: u16,
    /// Static files directory (optional)
    pub static_dir: Option<PathBuf>,
    /// Host to bind to (default: 127.0.0.1)
    pub host: String,
    /// Workspace root for resolving paths
    pub workspace_root: Option<PathBuf>,
}

impl ServerConfig {
    /// Create a new server configuration with defaults
    pub fn new(
        name: impl Into<String>,
        default_port: u16,
    ) -> Self {
        Self {
            name: name.into(),
            default_port,
            static_dir: None,
            host: "127.0.0.1".to_string(),
            workspace_root: None,
        }
    }

    /// Set the static files directory
    pub fn with_static_dir(
        mut self,
        dir: PathBuf,
    ) -> Self {
        self.static_dir = Some(dir);
        self
    }

    /// Set the host to bind to
    pub fn with_host(
        mut self,
        host: impl Into<String>,
    ) -> Self {
        self.host = host.into();
        self
    }

    /// Set the workspace root
    pub fn with_workspace_root(
        mut self,
        root: PathBuf,
    ) -> Self {
        self.workspace_root = Some(root);
        self
    }

    /// Get the port from environment or use default
    pub fn get_port(&self) -> u16 {
        std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(self.default_port)
    }

    /// Get the address to bind to
    pub fn get_addr(&self) -> String {
        format!("{}:{}", self.host, self.get_port())
    }

    /// Get the display address (converts 0.0.0.0 to localhost for user-friendly output)
    pub fn get_display_addr(&self) -> String {
        format!("{}:{}", display_host(&self.host), self.get_port())
    }
}

/// Convert a host address to a display-friendly version.
/// Converts `0.0.0.0` to `localhost` since browsers can't open `0.0.0.0`.
pub fn display_host(host: &str) -> &str {
    if host == "0.0.0.0" {
        "localhost"
    } else {
        host
    }
}

/// Parsed command-line arguments
#[derive(Debug, Clone)]
pub struct ServerArgs {
    /// Run HTTP server
    pub http: bool,
    /// Run MCP server
    pub mcp: bool,
}

impl ServerArgs {
    /// Parse command-line arguments
    pub fn parse() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let http = args.iter().any(|arg| arg == "--http");
        let mcp = args.iter().any(|arg| arg == "--mcp");

        // Default to HTTP if no flags specified
        let (http, mcp) = if !http && !mcp {
            (true, false)
        } else {
            (http, mcp)
        };

        Self { http, mcp }
    }

    /// Get mode description string
    pub fn mode_str(&self) -> &'static str {
        match (self.http, self.mcp) {
            (true, true) => "HTTP + MCP",
            (true, false) => "HTTP only",
            (false, true) => "MCP only",
            (false, false) => "none",
        }
    }
}

/// Initialize tracing with console output
pub fn init_tracing(level: &str) {
    let filter =
        EnvFilter::try_new(level).unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}

/// Create the default CORS layer for development
pub fn default_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

/// Wait for a Ctrl+C signal, then return.
///
/// Use with `axum::serve(...).with_graceful_shutdown(shutdown_signal())` so the
/// server shuts down cleanly and `Drop` impls (e.g. `DevServer`) run properly.
pub async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    info!("Received shutdown signal");
}

/// Create a router with static file serving.
///
/// Non-matching paths fall back to `index.html` so that client-side
/// SPA routing (Dioxus Router, etc.) works for deep-linked URLs.
pub fn with_static_files(
    router: Router,
    static_dir: Option<PathBuf>,
) -> Router {
    if let Some(dir) = static_dir {
        if dir.exists() {
            let index = dir.join("index.html");
            router.fallback_service(
                ServeDir::new(&dir)
                    .not_found_service(ServeFile::new(index)),
            )
        } else {
            router
        }
    } else {
        router
    }
}

/// Type alias for MCP server factory function
pub type McpServerFactory<S> = Box<
    dyn FnOnce(
            S,
        ) -> Pin<
            Box<
                dyn Future<
                        Output = Result<
                            (),
                            Box<dyn std::error::Error + Send + Sync>,
                        >,
                    > + Send,
            >,
        > + Send,
>;

/// Run the server based on command-line arguments.
///
/// # Arguments
///
/// * `config` - Server configuration
/// * `state` - Application state (cloned for MCP if both servers run)
/// * `create_router` - Function to create HTTP router from state
/// * `mcp_factory` - Optional factory to create MCP server
///
/// # Example
///
/// ```rust,no_run
/// use viewer_api::{ServerConfig, run_server, McpServerFactory};
/// use axum::Router;
/// use std::path::PathBuf;
///
/// #[derive(Clone)]
/// struct AppState;
///
/// fn routes(state: AppState, _static_dir: Option<PathBuf>) -> Router {
///     Router::new().with_state(state)
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let config = ServerConfig::new("example", 3000);
///     run_server(config, AppState, routes, None::<McpServerFactory<AppState>>).await.unwrap();
/// }
/// ```
pub async fn run_server<S, F>(
    config: ServerConfig,
    state: S,
    create_router: fn(S, Option<PathBuf>) -> Router,
    mcp_factory: Option<F>,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: Clone + Send + Sync + 'static,
    F: FnOnce(
            S,
        ) -> Pin<
            Box<
                dyn Future<
                        Output = Result<
                            (),
                            Box<dyn std::error::Error + Send + Sync>,
                        >,
                    > + Send,
            >,
        > + Send
        + 'static,
{
    let args = ServerArgs::parse();

    eprintln!("{} starting...", config.name);
    eprintln!("  Mode: {}", args.mode_str());

    if args.mcp && !args.http {
        // MCP-only mode
        if let Some(factory) = mcp_factory {
            factory(state).await.map_err(
                |e| -> Box<dyn std::error::Error> {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string(),
                    ))
                },
            )?;
        } else {
            eprintln!("MCP mode requested but no MCP handler provided");
            return Err("MCP mode not supported".into());
        }
    } else if args.http && !args.mcp {
        // HTTP-only mode
        run_http_server(config, state, create_router).await?;
    } else if args.http && args.mcp {
        // Both servers
        if let Some(factory) = mcp_factory {
            let state_clone = state.clone();

            // Spawn MCP server in background
            tokio::spawn(async move {
                if let Err(e) = factory(state_clone).await {
                    error!("MCP server error: {:?}", e);
                }
            });
        }

        // Run HTTP server in main task
        run_http_server(config, state, create_router).await?;
    }

    Ok(())
}

/// Run HTTP server (internal)
async fn run_http_server<S>(
    config: ServerConfig,
    state: S,
    create_router: fn(S, Option<PathBuf>) -> Router,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: Clone + Send + Sync + 'static,
{
    let addr = config.get_addr();
    let static_dir = config.static_dir.clone();

    if let Some(ref dir) = static_dir {
        eprintln!("  Static directory: {}", to_unix_path(dir));
    }
    eprintln!("  HTTP address: {}", addr);

    let app = create_router(state, static_dir);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    eprintln!(
        "HTTP server listening on http://{}",
        config.get_display_addr()
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
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
            .with_static_dir(PathBuf::from("/static"));

        assert_eq!(config.name, "test");
        assert_eq!(config.default_port, 3000);
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.static_dir, Some(PathBuf::from("/static")));
    }
}
