//! viewer-ctl — lifecycle manager for context-engine viewer servers.
//!
//! Replaces build_frontend.sh, start-viewer.sh, generate-types.sh, and
//! tools/ticket-vscode/build-and-install.sh with cross-platform Rust code.
//!
//! Usage:
//!   viewer-ctl start   <viewer> [--no-build] [-- <extra server args>]
//!   viewer-ctl stop    <viewer>
//!   viewer-ctl build   <viewer>
//!   viewer-ctl install <viewer>
//!   viewer-ctl gen-types
//!   viewer-ctl vscode-ext build
//!   viewer-ctl vscode-ext install
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
    about = "Lifecycle manager for context-engine viewer servers",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Build frontend artifacts, auto-install the server binary if needed,
    /// then launch it.
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
    /// Generate TypeScript type bindings from Rust ts-rs exports, then build
    /// the @context-engine/types npm package.
    GenTypes,
    /// Build or install the ticket-vscode VS Code extension.
    VscodeExt {
        #[command(subcommand)]
        action: VscodeExtAction,
    },
}

#[derive(Subcommand, Clone, Copy)]
enum VscodeExtAction {
    /// Compile the extension TypeScript only (npm run compile).
    Build,
    /// Compile then install to the VS Code extensions directory.
    Install,
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
// No ANSI escape codes: VS Code task background pattern matchers read raw
// stdout text and ANSI codes would prevent the regex patterns from matching.

macro_rules! info {
    ($tag:expr, $($arg:tt)*) => {
        println!("[{}] {}", $tag, format!($($arg)*));
    };
}
macro_rules! warn {
    ($tag:expr, $($arg:tt)*) => {
        println!("[{}] WARN {}", $tag, format!($($arg)*));
    };
}
macro_rules! error {
    ($tag:expr, $($arg:tt)*) => {
        eprintln!("[{}] ERROR {}", $tag, format!($($arg)*));
    };
}

// ── Port / process utilities ──────────────────────────────────────────────────

/// Find PIDs of processes listening on `port`.
///
/// Uses `sysinfo` for process metadata and falls back to platform-specific
/// commands to map port → PID because `sysinfo` doesn't expose TCP socket
/// info directly.
///
/// Tool selection priority (most reliable first):
///   Windows: PowerShell Get-NetTCPConnection (locale-agnostic enum) → ss → netstat
///   Linux/macOS: ss → lsof → netstat
fn pids_on_port(port: u16) -> Vec<Pid> {
    // ── Windows-first: PowerShell Get-NetTCPConnection ────────────────────
    // The -State parameter is a .NET enum value, not a localized string, so
    // this works correctly on all Windows locales (including German "ABHÖREN").
    if cfg!(windows) {
        if let Ok(out) = Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &format!(
                    "(Get-NetTCPConnection -LocalPort {port} -State Listen \
                     -ErrorAction SilentlyContinue).OwningProcess"
                ),
            ])
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            let pids: Vec<Pid> = text
                .split_whitespace()
                .filter_map(|s| s.parse::<usize>().ok())
                .map(Pid::from)
                .collect();
            if !pids.is_empty() || out.status.success() {
                // PowerShell succeeded (even if no results) — trust its output.
                return pids;
            }
        }
    }

    // ── Unix: ss (Linux/modern macOS/WSL) ─────────────────────────────────
    if let Ok(out) = Command::new("ss")
        .args(["-ltnp", &format!("sport = :{port}")])
        .output()
    {
        if out.status.success() {
            return parse_ss_pids(&String::from_utf8_lossy(&out.stdout), port);
        }
    }

    // ── Unix: lsof (macOS / Linux) ────────────────────────────────────────
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

    // ── Last resort: netstat (cross-platform fallback) ────────────────────
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
        // Windows netstat -ano format:
        //   TCP  <local>  <foreign>  <state>  <pid>
        // The state column is locale-dependent (e.g. "LISTENING" on English
        // Windows, "ABHÖREN" on German Windows).  We therefore do NOT check
        // the state string.  Instead we identify listening sockets by:
        //   1. Protocol is TCP
        //   2. Local address ends with :<port>
        //   3. Foreign address ends with :0  (remote port 0 → listening socket)
        if cols.len() < 5 {
            continue;
        }
        if cols[0] != "TCP" {
            continue;
        }
        if !cols[1].ends_with(&suffix) {
            continue;
        }
        // Foreign address ends with ":0" for LISTENING sockets (locale-agnostic).
        if !cols[2].ends_with(":0") {
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
            return !process_exists(pid);
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

    // The shared viewer-api TypeScript package lives at a well-known path
    // relative to each Vite viewer's frontend dir.  It must have its own
    // node_modules so that Rollup can resolve imports (e.g. prismjs) from
    // files in that directory.
    //
    // Layout:  tools/viewer/<viewer>/frontend/
    //                              ../../viewer-api/frontend/ts/
    let viewer_api_dir = frontend_dir.join("../../viewer-api/frontend/ts");
    if viewer_api_dir.is_dir() {
        if !viewer_api_dir.join("node_modules").is_dir() {
            info!(tag, "npm install in {}", viewer_api_dir.display());
            run_cmd("npm", &["install"], &viewer_api_dir, tag)?;
        }
    }

    if !frontend_dir.join("node_modules").is_dir() {
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
    let _ = tag; // used only in log macros at call site

    // On Windows, `.cmd` / `.bat` wrappers (npm, vsce, trunk, …) are not
    // directly executable by CreateProcess — we must route them through cmd.
    #[cfg(windows)]
    let status = {
        let mut cmd_args = vec!["/C", program];
        cmd_args.extend_from_slice(args);
        Command::new("cmd")
            .args(&cmd_args)
            .current_dir(cwd)
            .status()
            .map_err(|e| format!("failed to run `{program}` via cmd: {e}"))?
    };
    #[cfg(not(windows))]
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

    // Step 4: final port check — something may have grabbed the port during
    // the build.  Kill it now so the bind attempt below succeeds.
    let remaining = pids_on_port(port);
    if !remaining.is_empty() {
        warn!(
            tag,
            "port {port} was grabbed during build by PID(s): {:?} — evicting",
            remaining.iter().map(|p| p.as_u32()).collect::<Vec<_>>()
        );
        for pid in &remaining {
            kill_process(*pid, tag);
        }
        std::thread::sleep(Duration::from_millis(500));
        let still_remaining = pids_on_port(port);
        if !still_remaining.is_empty() {
            return Err(format!(
                "port {port} still occupied by {:?} after eviction — aborting",
                still_remaining
                    .iter()
                    .map(|p| p.as_u32())
                    .collect::<Vec<_>>()
            ));
        }
    }

    // Step 5: exec the server.
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

// ── gen-types ─────────────────────────────────────────────────────────────────

/// Generate TypeScript bindings via ts-rs cargo tests, then build the npm
/// package in `packages/context-types`.
fn cmd_gen_types() -> Result<(), String> {
    const TAG: &str = "gen-types";
    let root = repo_root();
    let generated_dir = root.join("packages/context-types/src/generated");

    // Remove stale .ts files; preserve .gitkeep
    if generated_dir.is_dir() {
        for entry in fs::read_dir(&generated_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.extension().map(|e| e == "ts").unwrap_or(false) {
                fs::remove_file(&path).map_err(|e| e.to_string())?;
            }
        }
    }

    info!(TAG, "generating TypeScript types from Rust ts-rs exports...");

    // Each entry: (cargo package, extra cargo args)
    let crates: &[(&str, &[&str])] = &[
        ("context-api", &["--features", "ts-gen"]),
        ("context-trace", &[]),
        ("log-viewer", &[]),
    ];

    for (pkg, extra) in crates {
        info!(TAG, "cargo test -p {pkg} export_bindings...");
        let mut cmd = Command::new("cargo");
        cmd.arg("test").arg("-p").arg(pkg);
        for arg in *extra {
            cmd.arg(arg);
        }
        cmd.args(["export_bindings", "--", "--ignored"])
            .current_dir(&root)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        // Ignore failures — not all crates may have export_bindings
        let _ = cmd.status();
    }

    // Count generated files
    let count = fs::read_dir(&generated_dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|x| x == "ts").unwrap_or(false))
                .count()
        })
        .unwrap_or(0);
    info!(TAG, "generated {count} TypeScript type file(s).");

    // Build the npm package
    let npm_dir = root.join("packages/context-types");
    info!(TAG, "npm run build in packages/context-types...");
    run_cmd("npm", &["run", "build"], &npm_dir, TAG)?;

    info!(TAG, "done.");
    Ok(())
}

// ── vscode-ext ────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct PkgJson {
    publisher: Option<String>,
    name: String,
    version: String,
}

/// Build (`npm run compile`) or build+install the ticket-vscode extension.
///
/// Install fast-path: if the VS Code extension dir already exists, syncs
/// `out/`, `resources/`, and `package.json` directly without packaging a VSIX.
/// Install slow-path: packages a VSIX with `vsce` and calls
/// `code --install-extension` (first-time only).
fn cmd_vscode_ext(action: VscodeExtAction) -> Result<(), String> {
    const TAG: &str = "vscode-ext";
    let root = repo_root();
    let ext_dir = root.join("tools/ticket-vscode");

    info!(TAG, "compiling TypeScript (npm run compile)...");
    run_cmd("npm", &["run", "compile"], &ext_dir, TAG)?;

    if matches!(action, VscodeExtAction::Build) {
        info!(TAG, "build complete.");
        return Ok(());
    }

    // Read package metadata
    let pkg_text = fs::read_to_string(ext_dir.join("package.json"))
        .map_err(|e| format!("failed to read package.json: {e}"))?;
    let pkg: PkgJson = serde_json::from_str(&pkg_text)
        .map_err(|e| format!("failed to parse package.json: {e}"))?;
    let publisher = pkg.publisher.as_deref().unwrap_or("undefined_publisher");
    let dirname = format!("{}.{}-{}", publisher, pkg.name, pkg.version);

    // Derive VS Code extension install path
    let user_home = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .map_err(|_| "neither USERPROFILE nor HOME is set".to_string())?;
    let install_dir = PathBuf::from(&user_home)
        .join(".vscode")
        .join("extensions")
        .join(&dirname);

    info!(TAG, "install dir: {}", install_dir.display());

    if install_dir.is_dir() {
        // ── Fast path: in-place sync ──────────────────────────────────────
        info!(TAG, "extension dir exists — syncing in-place...");

        let out_dst = install_dir.join("out");
        let _ = fs::remove_dir_all(&out_dst);
        fs::create_dir_all(&out_dst).map_err(|e| e.to_string())?;
        copy_dir_contents(&ext_dir.join("out"), &out_dst, TAG)?;

        let res_dst = install_dir.join("resources");
        fs::create_dir_all(&res_dst).map_err(|e| e.to_string())?;
        copy_dir_contents(&ext_dir.join("resources"), &res_dst, TAG)?;

        fs::copy(ext_dir.join("package.json"), install_dir.join("package.json"))
            .map_err(|e| format!("failed to copy package.json: {e}"))?;

        let nm = ext_dir.join("node_modules");
        if nm.is_dir() {
            let nm_dst = install_dir.join("node_modules");
            fs::create_dir_all(&nm_dst).map_err(|e| e.to_string())?;
            copy_dir_contents(&nm, &nm_dst, TAG)?;
        }

        info!(TAG, "sync complete. Reload the VS Code window to activate.");
    } else {
        // ── Slow path: VSIX package + install ─────────────────────────────
        info!(TAG, "install dir not found — performing first-time VSIX install...");
        run_cmd(
            "vsce",
            &[
                "package",
                "--no-dependencies",
                "--allow-missing-repository",
                "--skip-license",
            ],
            &ext_dir,
            TAG,
        )?;

        let vsix = find_newest_vsix(&ext_dir)?;
        info!(TAG, "installing {}...", vsix.display());
        run_cmd(
            "code",
            &["--install-extension", vsix.to_str().unwrap_or(""), "--force"],
            &ext_dir,
            TAG,
        )?;
        info!(TAG, "done. Reload the VS Code window to activate the extension.");
    }

    Ok(())
}

fn find_newest_vsix(dir: &Path) -> Result<PathBuf, String> {
    let mut vsix: Vec<_> = fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "vsix").unwrap_or(false))
        .collect();
    vsix.sort_by_key(|e| {
        e.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });
    vsix.last()
        .map(|e| e.path())
        .ok_or_else(|| "no .vsix file found in ticket-vscode/".to_string())
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Cmd::Start { viewer, no_build, extra } => cmd_start(viewer, no_build, extra),
        Cmd::Stop { viewer } => cmd_stop(viewer),
        Cmd::Build { viewer } => cmd_build(viewer),
        Cmd::Install { viewer } => cmd_install(viewer),
        Cmd::GenTypes => cmd_gen_types(),
        Cmd::VscodeExt { action } => cmd_vscode_ext(action),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
