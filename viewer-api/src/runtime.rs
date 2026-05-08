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
    services::ServeDir,
};
use tracing::{
    error,
    info,
};
use tracing_subscriber::{
    EnvFilter,
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

use crate::to_unix_path;

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

    pub fn with_level(
        mut self,
        level: impl Into<String>,
    ) -> Self {
        self.level = level.into();
        self
    }

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

pub fn init_tracing_full(config: &TracingConfig) {
    let filter = EnvFilter::try_new(&config.level)
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    if config.file_logging {
        let log_dir = config
            .log_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("logs"));
        std::fs::create_dir_all(&log_dir).ok();

        let log_file_name = format!("{}.log", config.log_file_prefix);
        let file_appender =
            tracing_appender::rolling::daily(&log_dir, &log_file_name);
        let (non_blocking, guard) =
            tracing_appender::non_blocking(file_appender);
        std::mem::forget(guard);

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

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub name: String,
    pub default_port: u16,
    pub static_dir: Option<PathBuf>,
    pub host: String,
    pub workspace_root: Option<PathBuf>,
}

impl ServerConfig {
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

    pub fn with_static_dir(
        mut self,
        dir: PathBuf,
    ) -> Self {
        self.static_dir = Some(dir);
        self
    }

    pub fn with_host(
        mut self,
        host: impl Into<String>,
    ) -> Self {
        self.host = host.into();
        self
    }

    pub fn with_workspace_root(
        mut self,
        root: PathBuf,
    ) -> Self {
        self.workspace_root = Some(root);
        self
    }

    pub fn get_port(&self) -> u16 {
        std::env::var("PORT")
            .ok()
            .and_then(|port| port.parse().ok())
            .unwrap_or(self.default_port)
    }

    pub fn get_addr(&self) -> String {
        format!("{}:{}", self.host, self.get_port())
    }

    pub fn get_display_addr(&self) -> String {
        format!("{}:{}", display_host(&self.host), self.get_port())
    }
}

pub fn display_host(host: &str) -> &str {
    if host == "0.0.0.0" {
        "localhost"
    } else {
        host
    }
}

mod cli;

pub use cli::{ServerArgs, init_tracing};

pub fn default_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

pub async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    info!("Received shutdown signal");
}

pub fn with_static_files(
    router: Router,
    static_dir: Option<PathBuf>,
) -> Router {
    use axum::{
        body::Body,
        http::{
            HeaderValue,
            Response,
            StatusCode,
            header,
        },
        response::IntoResponse,
    };
    use tower::service_fn;

    if let Some(dir) = static_dir {
        if dir.exists() {
            let index = dir.join("index.html");
            let index_html = std::fs::read(&index).unwrap_or_default();
            let spa_fallback = service_fn(move |_req| {
                let body = index_html.clone();
                async move {
                    let mut res: Response<Body> =
                        (StatusCode::OK, body).into_response();
                    res.headers_mut().insert(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static("text/html; charset=utf-8"),
                    );
                    Ok::<_, std::convert::Infallible>(res)
                }
            });
            router
                .fallback_service(ServeDir::new(&dir).fallback(spa_fallback))
        } else {
            router
        }
    } else {
        router
    }
}

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
        if let Some(factory) = mcp_factory {
            factory(state).await.map_err(
                |error| -> Box<dyn std::error::Error> {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        error.to_string(),
                    ))
                },
            )?;
        } else {
            eprintln!("MCP mode requested but no MCP handler provided");
            return Err("MCP mode not supported".into());
        }
    } else if args.http && !args.mcp {
        run_http_server(config, state, create_router).await?;
    } else if args.http && args.mcp {
        if let Some(factory) = mcp_factory {
            let state_clone = state.clone();
            tokio::spawn(async move {
                if let Err(error) = factory(state_clone).await {
                    error!("MCP server error: {:?}", error);
                }
            });
        }

        run_http_server(config, state, create_router).await?;
    }

    Ok(())
}

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