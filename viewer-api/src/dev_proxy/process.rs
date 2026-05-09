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
};

use tracing::{
    debug,
    info,
};

pub(super) fn ensure_npm_installed(
    frontend_dir: &Path
) -> Result<(), Box<dyn std::error::Error>> {
    let bin_dir = frontend_dir.join("node_modules/.bin");
    let has_vite =
        bin_dir.join("vite").exists() || bin_dir.join("vite.cmd").exists();
    if has_vite {
        debug!(dir = %frontend_dir.display(), "vite binary found, skipping npm install");
        return Ok(());
    }

    info!(dir = %frontend_dir.display(), "vite not found — running npm install");

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

pub(super) fn spawn_vite_process(
    frontend_dir: &Path,
    port: u16,
) -> Result<Child, Box<dyn std::error::Error>> {
    Command::new("npx")
        .args(["vite", "--port", &port.to_string(), "--strictPort"])
        .current_dir(frontend_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .or_else(|_| {
            debug!("npx not found directly, trying via cmd.exe (WSL/bash.exe)");
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
        .map_err(|error| {
            format!(
                "Failed to spawn Vite dev server (is Node.js installed?): {}",
                error
            )
            .into()
        })
}

fn run_npm_install(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let status = if cfg!(windows) {
        Command::new("cmd")
            .args(["/c", "npm", "install"])
            .current_dir(dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
    } else {
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
    .map_err(|error| {
        format!(
            "Failed to run npm install in {} (is Node.js installed?): {}",
            dir.display(),
            error
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

fn resolve_file_deps(
    pkg_json: &str,
    base_dir: &Path,
) -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    for line in pkg_json.lines() {
        let trimmed = line.trim();
        if let Some(pos) = trimmed.find("\"file:") {
            let after = &trimmed[pos + 6..];
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
