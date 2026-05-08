//! Dev proxy for forwarding requests to a Vite dev server.
//!
//! Provides a reverse proxy that forwards non-API requests (including
//! WebSocket upgrades for HMR) to a Vite dev server, enabling hot
//! module replacement during development.
//!
//! # Usage
//!
//! ```rust,no_run
//! use viewer_api::dev_proxy::{DevServer, dev_proxy_fallback};
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Spawn Vite dev server and wait for it to be ready
//! let dev_server = DevServer::start(Path::new("frontend"), 5173).await?;
//!
//! // Create fallback router that proxies to Vite
//! let fallback = dev_proxy_fallback(5173);
//! # Ok(())
//! # }
//! ```

use axum::{
    body::Body,
    extract::Request,
    response::{
        IntoResponse,
        Response,
    },
    Router,
};
use hyper::StatusCode;
use hyper_util::{
    client::legacy::Client,
    rt::TokioExecutor,
};
use std::{
    path::Path,
    process::Child,
    time::Duration,
};
use tokio::time::sleep;
use tracing::{
    debug,
    error,
    info,
    warn,
};

mod process;
mod websocket;

use self::process::{ensure_npm_installed, spawn_vite_process};
use self::websocket::proxy_websocket;

/// A running Vite dev server process.
///
/// Kills the child process when dropped.
pub struct DevServer {
    child: Child,
    port: u16,
}

impl DevServer {
    /// Spawn a Vite dev server and wait for it to become ready.
    ///
    /// # Arguments
    /// * `frontend_dir` - Path to the frontend directory containing package.json
    /// * `port` - Port for Vite to listen on
    ///
    /// # Returns
    /// A `DevServer` handle that kills the process on drop.
    pub async fn start(
        frontend_dir: &Path,
        port: u16,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        info!(dir = %frontend_dir.display(), port, "Starting Vite dev server");

        // Ensure npm dependencies are installed before starting Vite
        ensure_npm_installed(frontend_dir)?;

        let child = spawn_vite_process(frontend_dir, port)?;

        let mut server = Self { child, port };

        // Wait for Vite to be ready
        if let Err(e) = server.wait_until_ready().await {
            // Kill on failure
            let _ = server.child.kill();
            return Err(e);
        }

        info!(port, "Vite dev server is ready");
        Ok(server)
    }

    /// Poll until the Vite server responds to HTTP requests.
    async fn wait_until_ready(
        &mut self
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("http://localhost:{}", self.port);
        let max_attempts = 50; // 50 * 200ms = 10 seconds max
        let delay = Duration::from_millis(200);

        for attempt in 1..=max_attempts {
            // Check if the process has already exited (crashed)
            if let Some(status) = self.child.try_wait()? {
                // Capture stderr so the user can see why Vite failed
                let stderr_output = self
                    .child
                    .stderr
                    .take()
                    .and_then(|mut err| {
                        let mut buf = String::new();
                        std::io::Read::read_to_string(&mut err, &mut buf)
                            .ok()?;
                        Some(buf)
                    })
                    .unwrap_or_default();

                let stdout_output = self
                    .child
                    .stdout
                    .take()
                    .and_then(|mut out| {
                        let mut buf = String::new();
                        std::io::Read::read_to_string(&mut out, &mut buf)
                            .ok()?;
                        Some(buf)
                    })
                    .unwrap_or_default();

                let mut msg = format!(
                    "Vite process exited early with status: {}",
                    status
                );
                if !stdout_output.trim().is_empty() {
                    msg.push_str(&format!(
                        "\n\n--- stdout ---\n{}",
                        stdout_output.trim()
                    ));
                }
                if !stderr_output.trim().is_empty() {
                    msg.push_str(&format!(
                        "\n\n--- stderr ---\n{}",
                        stderr_output.trim()
                    ));
                }
                return Err(msg.into());
            }

            // Try connecting
            match tokio::net::TcpStream::connect(format!(
                "localhost:{}",
                self.port
            ))
            .await
            {
                Ok(_) => {
                    debug!(attempt, url, "Vite dev server responded");
                    return Ok(());
                },
                Err(_) => {
                    if attempt % 10 == 0 {
                        debug!(attempt, "Waiting for Vite dev server...");
                    }
                    sleep(delay).await;
                },
            }
        }

        Err(format!(
            "Vite dev server did not start within 10 seconds on {}",
            url
        )
        .into())
    }

    /// Get the port the dev server is running on.
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Drop for DevServer {
    fn drop(&mut self) {
        info!(port = self.port, "Shutting down Vite dev server");
        if let Err(e) = self.child.kill() {
            warn!(error = %e, "Failed to kill Vite dev server process");
        } else {
            // Reap the process to avoid zombies
            let _ = self.child.wait();
        }
    }
}

/// Create a fallback router that proxies all requests to a Vite dev server.
///
/// Handles both regular HTTP requests and WebSocket upgrades (for HMR).
pub fn dev_proxy_fallback(vite_port: u16) -> Router {
    Router::new().fallback(move |req: Request| async move {
        proxy_request(req, vite_port).await
    })
}

/// Proxy a single request to the Vite dev server.
///
/// Dispatches to either HTTP or WebSocket proxy based on the request headers.
async fn proxy_request(
    req: Request,
    vite_port: u16,
) -> Response {
    let is_upgrade = is_websocket_upgrade(&req);

    // Build the proxied URI
    let uri = req.uri();
    let path_and_query =
        uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
    let upstream_uri: hyper::Uri =
        format!("http://localhost:{}{}", vite_port, path_and_query)
            .parse()
            .unwrap();

    debug!(
        upstream = %upstream_uri,
        websocket = is_upgrade,
        method = %req.method(),
        "Proxying request to Vite"
    );

    if is_upgrade {
        proxy_websocket(req, upstream_uri).await
    } else {
        proxy_http(req, upstream_uri).await
    }
}

/// Check if this is a WebSocket upgrade request.
fn is_websocket_upgrade(req: &Request) -> bool {
    req.headers()
        .get(hyper::header::UPGRADE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false)
}

/// Proxy a regular HTTP request.
async fn proxy_http(
    req: Request,
    upstream_uri: hyper::Uri,
) -> Response {
    let client = Client::builder(TokioExecutor::new()).build_http::<Body>();

    // Rebuild the request with the upstream URI
    let (mut parts, body) = req.into_parts();
    parts.uri = upstream_uri;

    // Remove host header so hyper sets it correctly
    parts.headers.remove(hyper::header::HOST);

    let proxy_req = Request::from_parts(parts, body);

    match client.request(proxy_req).await {
        Ok(resp) => resp.into_response(),
        Err(e) => {
            error!(error = %e, "Failed to proxy request to Vite");
            (StatusCode::BAD_GATEWAY, format!("Dev proxy error: {}", e))
                .into_response()
        },
    }
}
