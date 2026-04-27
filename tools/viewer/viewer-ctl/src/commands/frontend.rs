//! Frontend lifecycle: build (with optional prebuild steps) and install.

use std::{fs, path::Path};

use crate::{
    config::{Config, Frontend, PrebuildStep},
    paths::{copy_dir_contents, disp},
    shell::run_cmd_owned,
};

pub fn build_frontend(root: &Path, f: &Frontend) -> Result<(), String> {
    let tag = f.name.as_str();
    let source = root.join(&f.source_dir);
    if !source.is_dir() {
        return Err(format!("frontend source not found: {}", disp(&source)));
    }

    // Run prebuild steps (e.g. npm install).
    for step in &f.prebuild {
        let dir = root.join(&step.dir);
        if !should_run_prebuild(&dir, step) {
            info!(tag, "prebuild skipped ({}): condition not met", step.dir);
            continue;
        }
        info!(tag, "prebuild {} in {}", step.cmd.join(" "), disp(&dir));
        run_cmd_owned(&step.cmd, &dir, tag)?;
    }

    info!(tag, "build: {} (cwd={})", f.build_cmd.join(" "), disp(&source));
    run_cmd_owned(&f.build_cmd, &source, tag)?;

    let output = root.join(&f.build_output);
    if !output.join("index.html").exists() {
        return Err(format!(
            "build did not produce {}/index.html",
            disp(&output)
        ));
    }

    // Copy extra assets (e.g. public/) into build_output so the install step
    // sees a single self-contained dir.
    for asset_dir in &f.extra_assets {
        let dir = root.join(asset_dir);
        if dir.is_dir() {
            copy_dir_contents(&dir, &output, tag)?;
        }
    }
    info!(tag, "frontend built → {}", disp(&output));
    Ok(())
}

pub fn install_frontend(cfg: &Config, root: &Path, f: &Frontend) -> Result<(), String> {
    let tag = f.name.as_str();
    let output = root.join(&f.build_output);
    if !output.join("index.html").exists() {
        return Err(format!(
            "build output not found at {} — run `viewer-ctl build {}` first",
            disp(&output),
            f.name
        ));
    }
    let dest = cfg.frontend_install_dir(&f.name);
    info!(tag, "installing to {}", disp(&dest));

    // Wipe the existing install dir, then mirror build_output into it. Done
    // in two steps so stale files from previous builds (e.g. old hashed
    // wasm/js bundles) don't linger and confuse the running server.
    if dest.exists() {
        fs::remove_dir_all(&dest)
            .map_err(|e| format!("failed to clear {}: {e}", disp(&dest)))?;
    }
    fs::create_dir_all(&dest)
        .map_err(|e| format!("failed to create {}: {e}", disp(&dest)))?;
    copy_dir_contents(&output, &dest, tag)?;

    info!(tag, "frontend installed.");
    Ok(())
}

fn should_run_prebuild(dir: &Path, step: &PrebuildStep) -> bool {
    let Some(cond) = &step.condition else {
        return true;
    };
    if let Some(rel) = cond.strip_prefix("missing:") {
        return !dir.join(rel).exists();
    }
    // Unknown condition → run anyway (fail safe).
    true
}
