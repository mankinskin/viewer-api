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
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
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
        let log_dir = env::var_os("LOG_DIR")
            .map(PathBuf::from)
            .or_else(|| env::var_os("LOG_FILE").map(|_| default_log_dir));

        Self {
            level,
            file_logging: log_dir.is_some(),
            log_dir,
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

fn file_appender(
    config: &TracingConfig
) -> Option<(
    tracing_appender::non_blocking::NonBlocking,
    tracing_appender::non_blocking::WorkerGuard,
)> {
    if !config.file_logging {
        return None;
    }

    let log_dir = config
        .log_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from("logs"));
    std::fs::create_dir_all(&log_dir).ok();

    let log_file_name = format!("{}.log", config.log_file_prefix);
    let file_appender =
        tracing_appender::rolling::daily(&log_dir, log_file_name);
    Some(tracing_appender::non_blocking(file_appender))
}

pub fn init_tracing_full(config: &TracingConfig) {
    let filter = EnvFilter::try_new(&config.level)
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    if let Some((non_blocking, guard)) = file_appender(config) {
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

        let log_dir = config
            .log_dir
            .as_deref()
            .unwrap_or_else(|| std::path::Path::new("logs"));
        info!(
            "File logging enabled to {}/{}",
            to_unix_path(&log_dir),
            format!("{}.log", config.log_file_prefix)
        );
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .init();
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::Write,
        time::{
            SystemTime,
            UNIX_EPOCH,
        },
    };

    use super::{
        file_appender,
        init_tracing_full,
        TracingConfig,
    };

    #[test]
    fn explicit_file_logging_creates_log_directory_and_file() {
        let log_dir = std::env::temp_dir().join(format!(
            "viewer-api-log-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let config = TracingConfig::default()
            .with_file_logging(log_dir.clone(), "viewer-api");
        let (mut writer, guard) = file_appender(&config).unwrap();

        writer.write_all(b"configured file logging\n").unwrap();
        drop(writer);
        drop(guard);

        assert!(log_dir.is_dir());
        assert!(std::fs::read_dir(&log_dir).unwrap().next().is_some());
        std::fs::remove_dir_all(log_dir).unwrap();
    }

    #[test]
    fn default_tracing_initialization_does_not_create_log_artifacts() {
        let temp_root = std::env::temp_dir().join(format!(
            "viewer-api-default-log-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let log_dir = temp_root.join("logs");
        let log_file = log_dir.join("viewer-api.log");
        let previous_log_dir = std::env::var_os("LOG_DIR");
        let previous_log_file = std::env::var_os("LOG_FILE");

        std::env::remove_var("LOG_DIR");
        std::env::remove_var("LOG_FILE");

        let result = std::panic::catch_unwind(|| {
            let config = TracingConfig::from_env("viewer-api", log_dir.clone());
            assert!(!config.file_logging);
            init_tracing_full(&config);

            assert!(!log_dir.exists());
            assert!(!log_file.exists());
            assert!(!temp_root.exists());
        });

        match previous_log_dir {
            Some(value) => std::env::set_var("LOG_DIR", value),
            None => std::env::remove_var("LOG_DIR"),
        }
        match previous_log_file {
            Some(value) => std::env::set_var("LOG_FILE", value),
            None => std::env::remove_var("LOG_FILE"),
        }

        result.unwrap();
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

pub use cli::{
    init_tracing,
    ServerArgs,
};

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
            header,
            HeaderValue,
            Response,
            StatusCode,
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
            router.fallback_service(ServeDir::new(&dir).fallback(spa_fallback))
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
