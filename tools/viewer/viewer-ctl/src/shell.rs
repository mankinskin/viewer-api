//! Subprocess helpers.
//!
//! On Windows, `.cmd`/`.bat` shims (npm, vsce, trunk, …) are routed through
//! `cmd /C` because [`std::process::Command`] cannot launch them directly.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// Run a command from string slices.
pub fn run_cmd_args(
    program: &str,
    args: &[&str],
    cwd: &Path,
    tag: &str,
) -> Result<(), String> {
    let owned: Vec<String> = std::iter::once(program.to_string())
        .chain(args.iter().map(|s| s.to_string()))
        .collect();
    run_cmd_owned(&owned, cwd, tag)
}

/// Run a command described as `[program, arg1, arg2, …]`. On Windows, routes
/// `.cmd`/`.bat` wrappers (npm, vsce, trunk, …) through `cmd /C`.
pub fn run_cmd_owned(parts: &[String], cwd: &Path, tag: &str) -> Result<(), String> {
    if parts.is_empty() {
        return Err(format!("[{tag}] empty command"));
    }
    let program = &parts[0];
    let args: Vec<&str> = parts[1..].iter().map(String::as_str).collect();

    #[cfg(windows)]
    let status = {
        let mut cmd_args = vec!["/C", program.as_str()];
        cmd_args.extend_from_slice(&args);
        Command::new("cmd")
            .args(&cmd_args)
            .current_dir(cwd)
            .status()
            .map_err(|e| format!("failed to run `{program}` via cmd: {e}"))?
    };
    #[cfg(not(windows))]
    let status = Command::new(program)
        .args(&args)
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

/// Resolve an executable on PATH (`where` on Windows, `which` elsewhere).
pub fn which(name: &str) -> Result<PathBuf, ()> {
    let cmd = if cfg!(windows) { "where" } else { "which" };
    let out = Command::new(cmd).arg(name).output().map_err(|_| ())?;
    if out.status.success() {
        Ok(PathBuf::from(
            String::from_utf8_lossy(&out.stdout)
                .trim()
                .lines()
                .next()
                .unwrap_or(""),
        ))
    } else {
        Err(())
    }
}
