//! Extension lifecycle. Currently only the VS Code installer kind is wired up.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use crate::{
    config::{Config, Extension},
    paths::{copy_dir_contents, disp},
    shell::{run_cmd_args, run_cmd_owned},
};

pub fn build_extension(root: &Path, e: &Extension) -> Result<(), String> {
    let tag = e.name.as_str();
    let dir = root.join(&e.source_dir);
    info!(tag, "build: {} (cwd={})", e.build_cmd.join(" "), disp(&dir));
    run_cmd_owned(&e.build_cmd, &dir, tag)
}

pub fn install_extension(_cfg: &Config, root: &Path, e: &Extension) -> Result<(), String> {
    // Build first so out/ is fresh.
    build_extension(root, e)?;
    match e.kind.as_str() {
        "vscode" => install_vscode_extension(root, e),
        other => Err(format!(
            "unknown extension kind `{other}` — viewer-ctl only knows `vscode`"
        )),
    }
}

#[derive(serde::Deserialize)]
struct PkgJson {
    publisher: Option<String>,
    name: String,
    version: String,
}

fn install_vscode_extension(root: &Path, e: &Extension) -> Result<(), String> {
    let tag = e.name.as_str();
    let ext_dir = root.join(&e.source_dir);

    let pkg_path = ext_dir.join(&e.package_json);
    let pkg_text = fs::read_to_string(&pkg_path)
        .map_err(|err| format!("failed to read {}: {err}", disp(&pkg_path)))?;
    let pkg: PkgJson = serde_json::from_str(&pkg_text)
        .map_err(|err| format!("failed to parse {}: {err}", disp(&pkg_path)))?;
    let publisher = pkg.publisher.as_deref().unwrap_or("undefined_publisher");
    let dirname = format!("{}.{}-{}", publisher, pkg.name, pkg.version);

    let user_home = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .map_err(|_| "neither USERPROFILE nor HOME is set".to_string())?;
    let install_dir = PathBuf::from(&user_home)
        .join(".vscode")
        .join("extensions")
        .join(&dirname);

    info!(tag, "install dir: {}", disp(&install_dir));

    if install_dir.is_dir() {
        // Fast path: in-place sync.
        info!(tag, "extension dir exists — syncing in-place...");

        let out_dst = install_dir.join("out");
        let _ = fs::remove_dir_all(&out_dst);
        fs::create_dir_all(&out_dst).map_err(|err| err.to_string())?;
        copy_dir_contents(&ext_dir.join("out"), &out_dst, tag)?;

        let res_src = ext_dir.join("resources");
        if res_src.is_dir() {
            let res_dst = install_dir.join("resources");
            fs::create_dir_all(&res_dst).map_err(|err| err.to_string())?;
            copy_dir_contents(&res_src, &res_dst, tag)?;
        }

        fs::copy(&pkg_path, install_dir.join(&e.package_json))
            .map_err(|err| format!("failed to copy package.json: {err}"))?;

        let nm = ext_dir.join("node_modules");
        if nm.is_dir() {
            let nm_dst = install_dir.join("node_modules");
            fs::create_dir_all(&nm_dst).map_err(|err| err.to_string())?;
            copy_dir_contents(&nm, &nm_dst, tag)?;
        }

        info!(tag, "sync complete. Reload the VS Code window to activate.");
    } else {
        // Slow path: package + install via vsce + code.
        info!(tag, "first-time install — packaging VSIX...");
        run_cmd_args(
            "vsce",
            &[
                "package",
                "--no-dependencies",
                "--allow-missing-repository",
                "--skip-license",
            ],
            &ext_dir,
            tag,
        )?;
        let vsix = find_newest_vsix(&ext_dir)?;
        info!(tag, "installing {}...", disp(&vsix));
        run_cmd_args(
            "code",
            &["--install-extension", vsix.to_str().unwrap_or(""), "--force"],
            &ext_dir,
            tag,
        )?;
        info!(tag, "done. Reload the VS Code window to activate.");
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
        .ok_or_else(|| "no .vsix file found in extension dir".to_string())
}
