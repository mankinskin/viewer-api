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
    rt::{
        TokioExecutor,
        TokioIo,
    },
};
use std::{
    path::{
        Path,
        PathBuf,
    },
    process::{
        Child,
        Command,
        Stdio,
    },
    time::Duration,
};
use tokio::time::sleep;
use tracing::{
    debug,
    error,
    info,
    warn,
};

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

        // Try spawning vite. On Windows (or WSL bash.exe), npx is a .cmd
        // file that must be invoked through cmd.exe. We try the native
        // approach first and fall back to cmd.exe if it fails.
        let child = Command::new("npx")
            .args(["vite", "--port", &port.to_string(), "--strictPort"])
            .current_dir(frontend_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .or_else(|_| {
                debug!(
                    "npx not found directly, trying via cmd.exe (WSL/bash.exe)"
                );
                Command::new("cmd.exe")
                    .args([
                        "/c",
                        "npx",
                        "vite",
                        "--port",
                        &port.to_string(),
                        "--strictPort",
                    ])
                    .current_dir(frontend_dir)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
            })
            .map_err(|e| {
                format!(
                "Failed to spawn Vite dev server (is Node.js installed?): {}",
                e
            )
            })?;

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

/// Ensure npm dependencies are installed in the given directory.
///
/// Checks for the `vite` binary inside `node_modules/.bin/`; if missing,
/// runs `npm install`. This catches both a completely missing `node_modules`
/// and a partial/stale install where vite was never fetched.
/// Also resolves any `file:` dependencies in `package.json` and installs
/// those first (e.g. shared workspace packages like `viewer-api/frontend`).
fn ensure_npm_installed(
    frontend_dir: &Path
) -> Result<(), Box<dyn std::error::Error>> {
    // Check for the vite binary — look for both Unix and Windows variants
    // regardless of compile target, because we may be running under WSL
    // bash.exe where cfg!(windows) is false but npm installed .cmd shims.
    let bin_dir = frontend_dir.join("node_modules/.bin");
    let has_vite =
        bin_dir.join("vite").exists() || bin_dir.join("vite.cmd").exists();

    if has_vite {
        debug!(dir = %frontend_dir.display(), "vite binary found, skipping npm install");
        return Ok(());
    }

    info!(dir = %frontend_dir.display(), "vite not found — running npm install");

    // Resolve local file: dependencies from package.json so they are
    // installed first (they may have their own node_modules).
    if let Ok(pkg_contents) =
        std::fs::read_to_string(frontend_dir.join("package.json"))
    {
        for dep_dir in resolve_file_deps(&pkg_contents, frontend_dir) {
            if !dep_dir.join("node_modules").exists() {
                info!(dep = %dep_dir.display(), "Installing local file: dependency");
                run_npm_install(&dep_dir)?;
            }
        }
    }

    run_npm_install(frontend_dir)
}

/// Run `npm install` in the given directory, returning an error on failure.
fn run_npm_install(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let status = if cfg!(windows) {
        Command::new("cmd")
            .args(["/c", "npm", "install"])
            .current_dir(dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
    } else {
        // Try npm directly first, fall back to cmd.exe for WSL bash.exe
        Command::new("npm")
            .arg("install")
            .current_dir(dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .or_else(|_| {
                debug!(
                    "npm not found directly, trying via cmd.exe (WSL/bash.exe)"
                );
                Command::new("cmd.exe")
                    .args(["/c", "npm", "install"])
                    .current_dir(dir)
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status()
            })
    }
    .map_err(|e| {
        format!(
            "Failed to run npm install in {} (is Node.js installed?): {}",
            dir.display(),
            e
        )
    })?;

    if !status.success() {
        return Err(format!(
            "npm install failed in {} with status: {}",
            dir.display(),
            status
        )
        .into());
    }

    info!(dir = %dir.display(), "npm install completed successfully");
    Ok(())
}

/// Parse `package.json` content and return resolved paths for any `file:`
/// dependencies so they can be installed before the main project.
fn resolve_file_deps(
    pkg_json: &str,
    base_dir: &Path,
) -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // Simple JSON value parsing — look for "file:..." strings in
    // dependencies / devDependencies without pulling in a full JSON parser
    // (serde_json is not a dependency of this crate).
    for line in pkg_json.lines() {
        let trimmed = line.trim();
        // Match patterns like:  "some-pkg": "file:../../viewer-api/frontend"
        if let Some(pos) = trimmed.find("\"file:") {
            let after = &trimmed[pos + 6..]; // skip `"file:`
            if let Some(end) = after.find('"') {
                let rel_path = &after[..end];
                let resolved = base_dir.join(rel_path);
                if resolved.join("package.json").exists() {
                    dirs.push(resolved);
                }
            }
        }
    }

    dirs
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

/// Proxy a WebSocket upgrade request.
///
/// 1. Extracts the browser's `OnUpgrade` handle from the request
/// 2. Forwards the upgrade request to Vite via hyper client
/// 3. Copies Vite's 101 response headers and returns 101 to browser
/// 4. Spawns a task that waits for both upgrades, then pipes the raw
///    IO streams bidirectionally
async fn proxy_websocket(
    mut req: Request,
    upstream_uri: hyper::Uri,
) -> Response {
    // Extract the browser's upgrade handle BEFORE consuming the request.
    // Axum/hyper stores this in request extensions.
    let browser_upgrade =
        match req.extensions_mut().remove::<hyper::upgrade::OnUpgrade>() {
            Some(u) => u,
            None => {
                error!("WebSocket request missing OnUpgrade extension");
                return (StatusCode::BAD_REQUEST, "Missing upgrade extension")
                    .into_response();
            },
        };

    // Build and send the upgrade request to Vite
    let client = Client::builder(TokioExecutor::new()).build_http::<Body>();

    let (mut parts, body) = req.into_parts();
    parts.uri = upstream_uri;
    parts.headers.remove(hyper::header::HOST);

    let vite_req = Request::from_parts(parts, body);

    let vite_resp = match client.request(vite_req).await {
        Ok(resp) => resp,
        Err(e) => {
            error!(error = %e, "Failed to proxy WebSocket to Vite");
            return (
                StatusCode::BAD_GATEWAY,
                format!("Dev proxy WebSocket error: {}", e),
            )
                .into_response();
        },
    };

    if vite_resp.status() != StatusCode::SWITCHING_PROTOCOLS {
        warn!(
            status = %vite_resp.status(),
            "Vite did not accept WebSocket upgrade"
        );
        return vite_resp.into_response();
    }

    debug!("Vite accepted WebSocket upgrade, setting up bidirectional pipe");

    // Build the 101 response to return to the browser, copying Vite's headers
    let mut resp_builder =
        Response::builder().status(StatusCode::SWITCHING_PROTOCOLS);
    for (name, value) in vite_resp.headers() {
        resp_builder = resp_builder.header(name, value);
    }

    // Spawn a background task that:
    // 1. Waits for Vite's upgrade IO (from the 101 response)
    // 2. Waits for the browser's upgrade IO (from our 101 response)
    // 3. Pipes both streams bidirectionally until one side closes
    tokio::spawn(async move {
        let vite_upgraded = match hyper::upgrade::on(vite_resp).await {
            Ok(io) => io,
            Err(e) => {
                error!(error = %e, "Vite WebSocket upgrade IO failed");
                return;
            },
        };

        let browser_upgraded = match browser_upgrade.await {
            Ok(io) => io,
            Err(e) => {
                error!(error = %e, "Browser WebSocket upgrade IO failed");
                return;
            },
        };

        let mut vite_io = TokioIo::new(vite_upgraded);
        let mut browser_io = TokioIo::new(browser_upgraded);

        match tokio::io::copy_bidirectional(&mut browser_io, &mut vite_io).await
        {
            Ok((browser_to_vite, vite_to_browser)) => {
                debug!(
                    browser_to_vite,
                    vite_to_browser, "WebSocket proxy connection closed"
                );
            },
            Err(e) => {
                // Connection reset is normal when a side closes
                debug!(error = %e, "WebSocket proxy pipe ended");
            },
        }
    });

    // Return the 101 response to the browser
    match resp_builder.body(Body::empty()) {
        Ok(resp) => resp,
        Err(e) => {
            error!(error = %e, "Failed to build WebSocket upgrade response");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to build upgrade response",
            )
                .into_response()
        },
    }
}
