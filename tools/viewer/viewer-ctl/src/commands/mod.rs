//! Command implementations for `viewer-ctl`.
//!
//! - `list` / `status` — read-only inspection
//! - `build` / `install` — dispatched per component kind via [`for_matching`]
//! - `start` / `stop` / `restart` — see [`server`]
//! - `task` — see [`task`]

pub mod extension;
pub mod frontend;
pub mod server;
pub mod task;

use std::path::Path;

use crate::{
    cli::KindArg,
    config::{Component, Config},
    paths::disp,
    process::pids_on_port,
};

use self::{
    extension::{build_extension, install_extension},
    frontend::{build_frontend, install_frontend},
    server::{build_server, install_server},
};

pub use self::{
    server::{cmd_start, cmd_stop},
    task::cmd_task,
};

// ── list / status ────────────────────────────────────────────────────────────

pub fn cmd_list(cfg: &Config) -> Result<(), String> {
    println!("servers:");
    for s in &cfg.servers {
        let fe = cfg
            .frontend_for_server(&s.name)
            .map(|f| format!(" ↔ frontend `{}`", f.name))
            .unwrap_or_default();
        println!(
            "  {:<16}  package={}  port={}{}",
            s.name, s.package, s.port, fe
        );
    }
    println!("frontends:");
    for f in &cfg.frontends {
        let install = cfg.frontend_install_dir(&f.name);
        println!(
            "  {:<16}  serves={:<14}  install={}",
            f.name,
            f.serves.as_deref().unwrap_or("-"),
            disp(&install)
        );
    }
    println!("extensions:");
    for e in &cfg.extensions {
        println!("  {:<16}  kind={}  source={}", e.name, e.kind, e.source_dir);
    }
    println!("tasks:");
    for t in &cfg.tasks {
        let desc = if t.description.is_empty() {
            String::new()
        } else {
            format!(" — {}", t.description)
        };
        println!("  {}{desc}", t.name);
    }
    Ok(())
}

pub fn cmd_status(cfg: &Config, name: Option<&str>) -> Result<(), String> {
    let servers: Vec<_> = match name {
        Some(n) => match cfg.server(n) {
            Some(s) => vec![s],
            None => return Err(format!("no [[server]] named `{n}` in viewer-ctl.toml")),
        },
        None => cfg.servers.iter().collect(),
    };
    for s in servers {
        let pids = pids_on_port(s.port);
        if pids.is_empty() {
            println!("  {:<16}  port={}  (down)", s.name, s.port);
        } else {
            let pid_list: Vec<String> = pids.iter().map(|p| p.as_u32().to_string()).collect();
            println!(
                "  {:<16}  port={}  pids=[{}]",
                s.name,
                s.port,
                pid_list.join(",")
            );
        }
    }
    Ok(())
}

// ── build / install dispatch ─────────────────────────────────────────────────

/// Apply a per-kind action to every component matching `name` in `cfg`.
///
/// When `kind` is `None` and multiple matches exist, all are processed in
/// the order: server, frontend, extension.
fn for_matching<F>(
    cfg: &Config,
    name: &str,
    kind: Option<KindArg>,
    mut action: F,
) -> Result<(), String>
where
    F: FnMut(Component<'_>) -> Result<(), String>,
{
    let mut matched = false;

    if matches!(kind, None | Some(KindArg::Server))
        && let Some(s) = cfg.server(name)
    {
        action(Component::Server(s))?;
        matched = true;
    }
    if matches!(kind, None | Some(KindArg::Frontend))
        && let Some(f) = cfg.frontend(name)
    {
        action(Component::Frontend(f))?;
        matched = true;
    }
    if matches!(kind, None | Some(KindArg::Extension))
        && let Some(e) = cfg.extension(name)
    {
        action(Component::Extension(e))?;
        matched = true;
    }

    if !matched {
        return Err(format!(
            "no component named `{name}` (kind filter: {:?}). Run `viewer-ctl list`.",
            kind
        ));
    }
    Ok(())
}

pub fn cmd_build(
    cfg: &Config,
    root: &Path,
    name: &str,
    kind: Option<KindArg>,
) -> Result<(), String> {
    for_matching(cfg, name, kind, |c| match c {
        Component::Server(s) => build_server(root, s),
        Component::Frontend(f) => build_frontend(root, f),
        Component::Extension(e) => build_extension(root, e),
    })
}

pub fn cmd_install(
    cfg: &Config,
    root: &Path,
    name: &str,
    kind: Option<KindArg>,
) -> Result<(), String> {
    for_matching(cfg, name, kind, |c| match c {
        Component::Server(s) => install_server(root, s),
        Component::Frontend(f) => install_frontend(cfg, root, f),
        Component::Extension(e) => install_extension(cfg, root, e),
    })
}
