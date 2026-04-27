//! Command-line interface for `viewer-ctl`.

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "viewer-ctl",
    about = "Config-driven lifecycle manager for context-engine viewers",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Cmd,
}

#[derive(Subcommand)]
pub enum Cmd {
    /// List every component defined in viewer-ctl.toml.
    List,
    /// Show running state of one or all servers.
    Status { name: Option<String> },
    /// Build artifacts for a component (no install).
    Build {
        name: String,
        /// Restrict to a single component kind when the name is ambiguous.
        #[arg(long)]
        kind: Option<KindArg>,
    },
    /// Install built artifacts.
    /// - server     → cargo install --path <source_dir> --force
    /// - frontend   → copy build_output to <install_root>/<name>/
    /// - extension  → kind-specific (e.g. vscode sync)
    Install {
        name: String,
        #[arg(long)]
        kind: Option<KindArg>,
    },
    /// Start a server. Sets STATIC_DIR if a frontend is linked AND installed.
    Start {
        server: String,
        /// Extra args forwarded to the server binary.
        #[arg(last = true)]
        extra: Vec<String>,
    },
    /// Stop the server (kills the process listening on its port).
    Stop { server: String },
    /// Stop then start the server.
    Restart {
        server: String,
        #[arg(last = true)]
        extra: Vec<String>,
    },
    /// Run a named task (sequence of shell commands).
    Task { name: String },
}

#[derive(Clone, Copy, ValueEnum, Debug)]
pub enum KindArg {
    Server,
    Frontend,
    Extension,
}
