//! Server lifecycle: build, install, start, stop.

use std::{
    env,
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};

use crate::{
    config::{Config, Server},
    paths::{crate_manifest_path_str, disp},
    process::{kill_process, pids_on_port, print_process_info},
    shell::{run_cmd_args, which},
};

pub fn build_server(root: &Path, s: &Server) -> Result<(), String> {
    let tag = s.name.as_str();
    let crate_path = root.join(&s.source_dir);
    let manifest = crate_manifest_path_str(&crate_path)?;
    info!(tag, "cargo build --release in {}", disp(&crate_path));
    run_cmd_args(
        "cargo",
        &["build", "--release", "--manifest-path", manifest.as_str()],
        root,
        tag,
    )
}

pub fn install_server(root: &Path, s: &Server) -> Result<(), String> {
    let tag = s.name.as_str();
    let crate_path = root.join(&s.source_dir);
    info!(tag, "cargo install --path {} --force", disp(&crate_path));
    run_cmd_args(
        "cargo",
        &[
            "install",
            "--path",
            crate_path.to_str().unwrap_or("."),
            "--force",
        ],
        root,
        tag,
    )
}

pub fn cmd_start(
    cfg: &Config,
    root: &Path,
    server: &str,
    extra: Vec<String>,
) -> Result<(), String> {
    let s = cfg
        .server(server)
        .ok_or_else(|| format!("no [[server]] named `{server}` in viewer-ctl.toml"))?;
    let tag = s.name.as_str();
    let port = port_for(s);

    // Step 1: ensure port is free.
    info!(tag, "checking port {port} for existing instances...");
    let listeners = pids_on_port(port);
    if !listeners.is_empty() {
        warn!(
            tag,
            "port {port} in use by PID(s): {:?}",
            listeners.iter().map(|p| p.as_u32()).collect::<Vec<_>>()
        );
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
    }

    // Step 2: locate the binary; auto-install on first run.
    let bin_path = match which(&s.package) {
        Ok(p) => p,
        Err(_) => {
            info!(tag, "{} not found on PATH — installing from source...", s.package);
            install_server(root, s)?;
            which(&s.package).map_err(|_| {
                format!(
                    "installed {} but still not found on PATH — check ~/.cargo/bin is in PATH",
                    s.package
                )
            })?
        }
    };

    // Step 3: derive STATIC_DIR from linked frontend if installed.
    let mut env_vars: Vec<(String, String)> =
        s.env.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    if let Some(fe) = cfg.frontend_for_server(&s.name) {
        let install_dir = cfg.frontend_install_dir(&fe.name);
        if install_dir.is_dir() {
            info!(tag, "STATIC_DIR={}", disp(&install_dir));
            env_vars.push((
                "STATIC_DIR".into(),
                install_dir.to_string_lossy().into_owned(),
            ));
        } else {
            warn!(
                tag,
                "frontend `{}` not installed (no {}); server will use its built-in default",
                fe.name,
                disp(&install_dir)
            );
        }
    }
    env_vars.push(("PORT".into(), port.to_string()));

    // Step 4: assemble args and launch.
    let mut server_args: Vec<String> = s.start_args.clone();
    server_args.extend(extra);

    info!(tag, "starting {} on port {port}", disp(&bin_path));

    spawn_server(&bin_path, &server_args, &env_vars, root, tag)
}

pub fn cmd_stop(cfg: &Config, server: &str) -> Result<(), String> {
    let s = cfg
        .server(server)
        .ok_or_else(|| format!("no [[server]] named `{server}` in viewer-ctl.toml"))?;
    let tag = s.name.as_str();
    let port = port_for(s);

    info!(tag, "looking for {} on port {port}...", s.package);
    let pids = pids_on_port(port);
    if pids.is_empty() {
        info!(tag, "no process listening on port {port}.");
        return Ok(());
    }
    for pid in pids {
        warn!(tag, "found process on port {port}:");
        print_process_info(pid, tag);
        if kill_process(pid, tag) {
            info!(tag, "PID {} terminated.", pid.as_u32());
        } else {
            error!(tag, "could not terminate PID {}.", pid.as_u32());
            if cfg!(windows) {
                error!(tag, "  taskkill /F /PID {}", pid.as_u32());
            }
            error!(tag, "  kill -9 {}", pid.as_u32());
            return Err(format!("failed to kill PID {}", pid.as_u32()));
        }
    }
    Ok(())
}

fn port_for(s: &Server) -> u16 {
    env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(s.port)
}

fn spawn_server(
    bin_path: &Path,
    args: &[String],
    env_vars: &[(String, String)],
    cwd: &Path,
    tag: &str,
) -> Result<(), String> {
    // On Unix, exec() replaces this process image entirely.
    // On Windows, spawn detached so viewer-ctl exits and releases file locks.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let _ = tag;
        let mut cmd = Command::new(bin_path);
        for (k, v) in env_vars {
            cmd.env(k, v);
        }
        let err = cmd.args(args).current_dir(cwd).exec();
        return Err(format!("exec failed: {err}"));
    }

    #[cfg(not(unix))]
    {
        let mut cmd = Command::new(bin_path);
        for (k, v) in env_vars {
            cmd.env(k, v);
        }
        cmd.args(args)
            .current_dir(cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("failed to launch {}: {e}", disp(bin_path)))?;
        info!(tag, "launched (detached). viewer-ctl exiting.");
        Ok(())
    }
}
