//! viewer-ctl — build, install, start, and stop context-engine viewer servers.
//!
//! Replaces `scripts/start-viewer.sh` with a cross-platform Rust binary.
//!
//! Usage:
//!   viewer-ctl start  <viewer> [--no-build] [-- <extra server args>]
//!   viewer-ctl stop   <viewer>
//!   viewer-ctl build  <viewer>
//!   viewer-ctl install <viewer>
//!
//! Viewers: doc-viewer  log-viewer  ticket-viewer  spec-viewer
//!
//! Environment:
//!   PORT — override the default port for any viewer.

use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
    time::Duration,
};

use clap::{Parser, Subcommand, ValueEnum};
use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};

// ── CLI definition ────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "viewer-ctl",
    about = "Build, install, start, and stop context-engine viewer servers",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Build frontend artifacts and (if not already installed) install the
    /// server binary, then launch it.
    Start {
        viewer: Viewer,
        /// Skip the frontend build step.
        #[arg(long)]
        no_build: bool,
        /// Extra arguments forwarded to the server binary.
        #[arg(last = true)]
        extra: Vec<String>,
    },
    /// Stop the running viewer server by killing the process on its port.
    Stop {
        viewer: Viewer,
    },
    /// Build the frontend artifacts only (no server install or launch).
    Build {
        viewer: Viewer,
    },
    /// Install (or reinstall) the viewer server binary to ~/.cargo/bin.
    Install {
        viewer: Viewer,
    },
}

#[derive(Clone, Copy, ValueEnum, Debug)]
enum Viewer {
    DocViewer,
    LogViewer,
    TicketViewer,
    SpecViewer,
}

// ── Per-viewer configuration ──────────────────────────────────────────────────

#[derive(Debug)]
enum FrontendKind {
    Vite,
    Trunk,
}

struct ViewerConfig {
    /// Cargo package name (also the installed binary name).
    pkg: &'static str,
    /// Default TCP port.
    default_port: u16,
    /// Frontend build tool.
    frontend_kind: FrontendKind,
    /// Path to the frontend source directory, relative to the viewer root.
    frontend_subdir: &'static str,
    /// Path to the built static output, relative to the viewer root.
    static_subdir: &'static str,
    /// Extra args always passed to the server (e.g. --static for log-viewer).
    fixed_server_args: &'static [&'static str],
}

impl ViewerConfig {
    fn for_viewer(v: Viewer) -> Self {
        match v {
            Viewer::DocViewer => Self {
                pkg: "doc-viewer",
                default_port: 3001,
                frontend_kind: FrontendKind::Vite,
                frontend_subdir: "frontend",
                static_subdir: "static",
                fixed_server_args: &[],
            },
            Viewer::LogViewer => Self {
                pkg: "log-viewer",
                default_port: 3000,
                frontend_kind: FrontendKind::Vite,
                frontend_subdir: "frontend",
                static_subdir: "static",
                fixed_server_args: &["--static"],
            },
            Viewer::TicketViewer => Self {
                pkg: "ticket-viewer",
                default_port: 3002,
                frontend_kind: FrontendKind::Trunk,
                frontend_subdir: "frontend/dioxus",
                static_subdir: "frontend/dioxus/dist",
                fixed_server_args: &[],
            },
            Viewer::SpecViewer => Self {
                pkg: "spec-viewer",
                default_port: 4002,
                frontend_kind: FrontendKind::Trunk,
                frontend_subdir: "frontend/dioxus",
                static_subdir: "frontend/dioxus/dist",
                fixed_server_args: &[],
            },
        }
    }
}

// ── Logging helpers ───────────────────────────────────────────────────────────

macro_rules! info {
    ($tag:expr, $($arg:tt)*) => {
        println!("\x1b[36m[{}]\x1b[0m {}", $tag, format!($($arg)*));
    };
}
macro_rules! warn {
    ($tag:expr, $($arg:tt)*) => {
        println!("\x1b[33m[{}]\x1b[0m {}", $tag, format!($($arg)*));
    };
}
macro_rules! error {
    ($tag:expr, $($arg:tt)*) => {
        eprintln!("\x1b[31m[{}]\x1b[0m {}", $tag, format!($($arg)*));
    };
}

// ── Port / process utilities ──────────────────────────────────────────────────

/// Find PIDs of processes listening on `port`.
///
/// Uses `sysinfo` for process metadata and falls back to platform-specific
/// commands (`ss`, `lsof`, `netstat`) to map port → PID because `sysinfo`
/// doesn't expose TCP socket info directly.
fn pids_on_port(port: u16) -> Vec<Pid> {
    // Try ss (Linux/modern macOS/WSL)
    if let Ok(out) = Command::new("ss")
        .args(["-ltnp", &format!("sport = :{port}")])
        .output()
    {
        if out.status.success() {
            return parse_ss_pids(&String::from_utf8_lossy(&out.stdout), port);
        }
    }

    // Try lsof (macOS / Linux)
    if let Ok(out) = Command::new("lsof")
        .args(["-ti", &format!(":{port}"), "-sTCP:LISTEN"])
        .output()
    {
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout)
                .split_whitespace()
                .filter_map(|s| s.parse::<usize>().ok().map(Pid::from))
                .collect();
        }
    }

    // Fallback: netstat (Windows / MSYS)
    if let Ok(out) = Command::new("netstat").args(["-ano"]).output() {
        if out.status.success() {
            return parse_netstat_pids(&String::from_utf8_lossy(&out.stdout), port);
        }
    }

    vec![]
}

fn parse_ss_pids(output: &str, port: u16) -> Vec<Pid> {
    let suffix = format!(":{port}");
    let mut pids = Vec::new();
    for line in output.lines() {
        // Local address is field 4; pid= is in field 6 (Netid State Recv-Q Send-Q Local Peer Process)
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 5 {
            continue;
        }
        if !cols[3].ends_with(&suffix) {
            continue;
        }
        // pid= pattern inside the last column: "users:(("name",pid=1234,fd=8))"
        if let Some(pid_str) = line
            .split("pid=")
            .nth(1)
            .and_then(|s| s.split(',').next())
            .and_then(|s| s.split(')').next())
        {
            if let Ok(n) = pid_str.trim().parse::<usize>() {
                pids.push(Pid::from(n));
            }
        }
    }
    pids
}

fn parse_netstat_pids(output: &str, port: u16) -> Vec<Pid> {
    let suffix = format!(":{port}");
    let mut pids = Vec::new();
    for line in output.lines() {
        let cols: Vec<&str> = line.split_whitespace().collect();
        // TCP  <local>  <foreign>  LISTENING  <pid>
        if cols.len() < 5 {
            continue;
        }
        if cols[0] != "TCP" {
            continue;
        }
        if !cols[1].ends_with(&suffix) {
            continue;
        }
        if cols[3] != "LISTENING" {
            continue;
        }
        if let Ok(n) = cols[4].parse::<usize>() {
            pids.push(Pid::from(n));
        }
    }
    pids.sort_unstable();
    pids.dedup();
    pids
}

/// Kill a process, trying SIGTERM first then SIGKILL on Unix, taskkill on Windows.
/// Returns true if the process is no longer alive.
fn kill_process(pid: Pid, tag: &str) -> bool {
    warn!(tag, "killing PID {pid}");

    // Windows / MSYS2
    if cfg!(windows)
        || Command::new("taskkill")
            .args(["/?"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok()
    {
        let result = Command::new("taskkill")
            .args(["/F", "/PID", &pid.as_u32().to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        if result.is_ok() {
            std::thread::sleep(Duration::from_millis(300));
            return pids_on_port(0).contains(&pid) == false
                && !process_exists(pid);
        }
    }

    // Unix: SIGTERM then SIGKILL
    let _ = Command::new("kill")
        .arg(pid.as_u32().to_string())
        .status();
    std::thread::sleep(Duration::from_millis(500));
    if !process_exists(pid) {
        return true;
    }
    let _ = Command::new("kill")
        .args(["-9", &pid.as_u32().to_string()])
        .status();
    std::thread::sleep(Duration::from_millis(500));
    !process_exists(pid)
}

fn process_exists(pid: Pid) -> bool {
    let sys = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::new()),
    );
    sys.process(pid).is_some()
}

fn print_process_info(pid: Pid, tag: &str) {
    // Windows: tasklist
    if let Ok(out) = Command::new("tasklist")
        .args([
            "/FI",
            &format!("PID eq {}", pid.as_u32()),
            "/FO",
            "LIST",
        ])
        .output()
    {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                if line.starts_with("Image Name")
                    || line.starts_with("PID")
                    || line.starts_with("Mem Usage")
                {
                    warn!(tag, "  {line}");
                }
            }
            return;
        }
    }
    // Unix: ps
    if let Ok(out) = Command::new("ps")
        .args([
            "-p",
            &pid.as_u32().to_string(),
            "-o",
            "pid,comm,args",
            "--no-headers",
        ])
        .output()
    {
        if out.status.success() {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                warn!(tag, "  {line}");
            }
            return;
        }
    }
    warn!(tag, "  PID: {}", pid.as_u32());
}

// ── Frontend build helpers ────────────────────────────────────────────────────

fn build_vite(frontend_dir: &Path, static_dir: &Path, tag: &str) -> Result<(), String> {
    if !frontend_dir.is_dir() {
        return Err(format!("vite frontend dir not found: {}", frontend_dir.display()));
    }

    let needs_install = !frontend_dir.join("node_modules").is_dir()
        || !frontend_dir
            .join("node_modules/@context-engine/viewer-api-frontend")
            .is_dir();

    if needs_install {
        info!(tag, "npm install in {}", frontend_dir.display());
        run_cmd("npm", &["install"], frontend_dir, tag)?;
    }

    info!(tag, "vite build → {}", static_dir.display());
    run_cmd("npx", &["vite", "build"], frontend_dir, tag)?;

    if !static_dir.join("index.html").exists() {
        return Err(format!(
            "vite build did not produce {}/index.html",
            static_dir.display()
        ));
    }
    Ok(())
}

fn build_trunk(
    frontend_dir: &Path,
    static_dir: &Path,
    viewer_root: &Path,
    tag: &str,
) -> Result<(), String> {
    if !frontend_dir.is_dir() {
        return Err(format!(
            "trunk frontend dir not found: {}",
            frontend_dir.display()
        ));
    }
    which("trunk").map_err(|_| {
        "trunk not found on PATH. Install with: cargo install trunk".to_string()
    })?;

    info!(tag, "trunk build (release) in {}", frontend_dir.display());
    run_cmd("trunk", &["build", "--release"], frontend_dir, tag)?;

    if !static_dir.join("index.html").exists() {
        return Err(format!(
            "trunk build did not produce {}/index.html",
            static_dir.display()
        ));
    }

    // Copy public/ assets that trunk may skip when wasm-opt crashes on Windows.
    let public_dir = viewer_root.join("frontend/dioxus/public");
    if public_dir.is_dir() {
        copy_dir_contents(&public_dir, static_dir, tag)?;
    }

    Ok(())
}

/// Recursively copy `src` directory contents into `dst`.
fn copy_dir_contents(src: &Path, dst: &Path, tag: &str) -> Result<(), String> {
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let dest_path = dst.join(entry.file_name());
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            fs::create_dir_all(&dest_path).map_err(|e| e.to_string())?;
            copy_dir_contents(&entry.path(), &dest_path, tag)?;
        } else {
            fs::copy(entry.path(), &dest_path).map_err(|e| e.to_string())?;
            info!(tag, "copied {}", dest_path.display());
        }
    }
    Ok(())
}

fn run_cmd(
    program: &str,
    args: &[&str],
    cwd: &Path,
    tag: &str,
) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .status()
        .map_err(|e| format!("failed to run `{program}`: {e}"))?;
    if !status.success() {
        return Err(format!(
            "`{program} {}` exited with status {status}",
            args.join(" ")
        ));
    }
    let _ = tag; // used only in log macros at call site
    Ok(())
}

fn which(name: &str) -> Result<PathBuf, ()> {
    let cmd = if cfg!(windows) { "where" } else { "which" };
    let out = Command::new(cmd).arg(name).output().map_err(|_| ())?;
    if out.status.success() {
        Ok(PathBuf::from(
            String::from_utf8_lossy(&out.stdout).trim().lines().next().unwrap_or(""),
        ))
    } else {
        Err(())
    }
}

// ── Repository root resolution ────────────────────────────────────────────────

/// Walk up from the current executable's location to find the workspace root
/// (directory that contains Cargo.toml with `[workspace]`).  Falls back to
/// `$CARGO_MANIFEST_DIR` (set by cargo) or the process working directory.
fn repo_root() -> PathBuf {
    // Prefer CARGO_MANIFEST_DIR so `cargo run` always works.
    if let Ok(dir) = env::var("CARGO_MANIFEST_DIR") {
        // viewer-ctl lives at <repo>/tools/viewer/viewer-ctl — go up 3 levels.
        let p = PathBuf::from(&dir);
        let candidate = p.ancestors().nth(3).map(|a| a.to_path_buf());
        if let Some(root) = candidate {
            if root.join("Cargo.toml").exists() {
                return root;
            }
        }
    }
    // Walk up from cwd to find a Cargo.toml containing [workspace].
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for ancestor in cwd.ancestors() {
        let cargo_toml = ancestor.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(contents) = fs::read_to_string(&cargo_toml) {
                if contents.contains("[workspace]") {
                    return ancestor.to_path_buf();
                }
            }
        }
    }
    cwd
}

// ── Commands ──────────────────────────────────────────────────────────────────

fn cmd_build(viewer: Viewer) -> Result<(), String> {
    let cfg = ViewerConfig::for_viewer(viewer);
    let root = repo_root();
    let viewer_root = root.join("tools/viewer").join(cfg.pkg);
    let frontend_dir = viewer_root.join(cfg.frontend_subdir);
    let static_dir = viewer_root.join(cfg.static_subdir);
    let tag = cfg.pkg;

    match cfg.frontend_kind {
        FrontendKind::Vite => build_vite(&frontend_dir, &static_dir, tag)?,
        FrontendKind::Trunk => {
            build_trunk(&frontend_dir, &static_dir, &viewer_root, tag)?
        }
    }
    info!(tag, "frontend artifacts ready.");
    Ok(())
}

fn cmd_install(viewer: Viewer) -> Result<(), String> {
    let cfg = ViewerConfig::for_viewer(viewer);
    let root = repo_root();
    let crate_path = root.join("tools/viewer").join(cfg.pkg);
    let tag = cfg.pkg;

    info!(tag, "cargo install --path {} --force", crate_path.display());
    run_cmd(
        "cargo",
        &[
            "install",
            "--path",
            crate_path.to_str().unwrap_or(cfg.pkg),
            "--force",
        ],
        &root,
        tag,
    )
}

fn cmd_stop(viewer: Viewer) -> Result<(), String> {
    let cfg = ViewerConfig::for_viewer(viewer);
    let tag = cfg.pkg;
    let port = port_for(&cfg);

    info!(tag, "looking for {name} on port {port}...", name = cfg.pkg);
    let pids = pids_on_port(port);
    if pids.is_empty() {
        info!(tag, "no process listening on port {port} — {name} is not running.", name = cfg.pkg);
        return Ok(());
    }

    for pid in pids {
        warn!(tag, "found process on port {port}:");
        print_process_info(pid, tag);

        if kill_process(pid, tag) {
            info!(tag, "PID {} terminated.", pid.as_u32());
        } else {
            error!(tag, "could not terminate PID {} automatically.", pid.as_u32());
            error!(tag, "Kill it manually:");
            if cfg!(windows) {
                error!(tag, "  taskkill /F /PID {}", pid.as_u32());
            }
            error!(tag, "  kill -9 {}", pid.as_u32());
            return Err(format!("failed to kill PID {}", pid.as_u32()));
        }
    }
    Ok(())
}

fn cmd_start(viewer: Viewer, no_build: bool, extra: Vec<String>) -> Result<(), String> {
    let cfg = ViewerConfig::for_viewer(viewer);
    let tag = cfg.pkg;
    let port = port_for(&cfg);
    let root = repo_root();

    // Step 1: free the port.
    info!(tag, "checking port {port} for existing instances...");
    let listeners = pids_on_port(port);
    if !listeners.is_empty() {
        warn!(tag, "port {port} in use by PID(s): {:?}", listeners.iter().map(|p| p.as_u32()).collect::<Vec<_>>());
        for pid in &listeners {
            kill_process(*pid, tag);
        }
        std::thread::sleep(Duration::from_secs(1));
        let remaining = pids_on_port(port);
        if !remaining.is_empty() {
            return Err(format!(
                "port {port} still occupied by {:?} — aborting",
                remaining.iter().map(|p| p.as_u32()).collect::<Vec<_>>()
            ));
        }
        info!(tag, "port {port} freed.");
    } else {
        info!(tag, "port {port} is free.");
    }

    // Step 2: build frontend.
    if !no_build {
        let viewer_root = root.join("tools/viewer").join(cfg.pkg);
        let frontend_dir = viewer_root.join(cfg.frontend_subdir);
        let static_dir = viewer_root.join(cfg.static_subdir);
        match cfg.frontend_kind {
            FrontendKind::Vite => build_vite(&frontend_dir, &static_dir, tag)?,
            FrontendKind::Trunk => {
                build_trunk(&frontend_dir, &static_dir, &viewer_root, tag)?
            }
        }
        info!(tag, "frontend artifacts ready.");
    } else {
        info!(tag, "skipping frontend build (--no-build)");
    }

    // Step 3: find or install binary.
    let bin_path = match which(cfg.pkg) {
        Ok(p) => p,
        Err(_) => {
            info!(tag, "{} not found on PATH — installing from source...", cfg.pkg);
            cmd_install(viewer)?;
            which(cfg.pkg).map_err(|_| format!(
                "installed {} but still not found on PATH — check ~/.cargo/bin is in PATH",
                cfg.pkg
            ))?
        }
    };

    // Step 4: exec the server.
    let mut server_args: Vec<String> = cfg
        .fixed_server_args
        .iter()
        .map(|s| s.to_string())
        .collect();
    server_args.extend(extra);

    info!(tag, "starting {} on port {port}", bin_path.display());

    // Use exec-style replacement on Unix; on Windows spawn + wait.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = Command::new(&bin_path)
            .env("PORT", port.to_string())
            .args(&server_args)
            .current_dir(&root)
            .exec();
        return Err(format!("exec failed: {err}"));
    }

    #[cfg(not(unix))]
    {
        let status = Command::new(&bin_path)
            .env("PORT", port.to_string())
            .args(&server_args)
            .current_dir(&root)
            .status()
            .map_err(|e| format!("failed to launch {}: {e}", bin_path.display()))?;
        if !status.success() {
            return Err(format!("{} exited with {status}", cfg.pkg));
        }
        Ok(())
    }
}

fn port_for(cfg: &ViewerConfig) -> u16 {
    env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(cfg.default_port)
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Cmd::Start { viewer, no_build, extra } => cmd_start(viewer, no_build, extra),
        Cmd::Stop { viewer } => cmd_stop(viewer),
        Cmd::Build { viewer } => cmd_build(viewer),
        Cmd::Install { viewer } => cmd_install(viewer),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("\x1b[31merror:\x1b[0m {e}");
            ExitCode::FAILURE
        }
    }
}
