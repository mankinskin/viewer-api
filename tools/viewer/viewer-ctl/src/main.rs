//! viewer-ctl — config-driven lifecycle manager for context-engine viewers,
//! frontends, and extensions.
//!
//! All component definitions live in `viewer-ctl.toml` at the repo root.
//! See [`config`] for the schema.
//!
//! Commands:
//! ```text
//! viewer-ctl list                       List every component.
//! viewer-ctl status [<name>]            Show running state of servers.
//! viewer-ctl build   <name>             Build artifacts (no install).
//! viewer-ctl install <name>             Install artifacts.
//! viewer-ctl start   <server>           Launch a server.
//! viewer-ctl stop    <server>           Kill the server's port owner.
//! viewer-ctl restart <server>           stop + start.
//! viewer-ctl task    <name>             Run a multi-step task.
//! ```
//!
//! Frontend/server lifecycle is decoupled. The expected workflow is:
//! ```text
//! viewer-ctl install spec-viewer --kind server     # one-off
//! viewer-ctl build   spec-viewer --kind frontend   # rebuild
//! viewer-ctl install spec-viewer --kind frontend   # publish to install dir
//! ```
//! When a server and frontend share the same `name` and no `--kind` is
//! given, viewer-ctl applies the command to BOTH (servers first).

#[macro_use]
mod logging;

mod cli;
mod commands;
mod config;
mod paths;
mod process;
mod shell;

use std::{process::ExitCode, time::Duration};

use clap::Parser;

use crate::{
    cli::{Cli, Cmd},
    config::Config,
};

fn main() -> ExitCode {
    let cli = Cli::parse();
    let root = paths::repo_root();
    let cfg = match Config::load(&root) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let result = match cli.command {
        Cmd::List => commands::cmd_list(&cfg),
        Cmd::Status { name } => commands::cmd_status(&cfg, name.as_deref()),
        Cmd::Build { name, kind } => commands::cmd_build(&cfg, &root, &name, kind),
        Cmd::Install { name, kind } => commands::cmd_install(&cfg, &root, &name, kind),
        Cmd::Start { server, extra } => commands::cmd_start(&cfg, &root, &server, extra),
        Cmd::Stop { server } => commands::cmd_stop(&cfg, &server),
        Cmd::Restart { server, extra } => match commands::cmd_stop(&cfg, &server) {
            Ok(()) => {
                std::thread::sleep(Duration::from_millis(500));
                commands::cmd_start(&cfg, &root, &server, extra)
            }
            Err(e) => Err(e),
        },
        Cmd::Task { name } => commands::cmd_task(&cfg, &root, &name),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
