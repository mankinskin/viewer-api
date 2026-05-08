use tracing_subscriber::{
    EnvFilter,
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

#[derive(Debug, Clone)]
pub struct ServerArgs {
    pub http: bool,
    pub mcp: bool,
}

impl ServerArgs {
    pub fn parse() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let http = args.iter().any(|arg| arg == "--http");
        let mcp = args.iter().any(|arg| arg == "--mcp");

        let (http, mcp) = if !http && !mcp {
            (true, false)
        } else {
            (http, mcp)
        };

        Self { http, mcp }
    }

    pub fn mode_str(&self) -> &'static str {
        match (self.http, self.mcp) {
            (true, true) => "HTTP + MCP",
            (true, false) => "HTTP only",
            (false, true) => "MCP only",
            (false, false) => "none",
        }
    }
}

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