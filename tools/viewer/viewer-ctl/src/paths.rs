//! Filesystem helpers and repo-root resolution.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

/// Render a path for human-readable output with forward slashes regardless
/// of platform. Use everywhere instead of [`Path::display`] so log lines
/// don't mix `\` and `/` separators on Windows.
pub fn disp(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

/// Recursively copy `src` directory contents into `dst`.
///
/// `tag` is currently unused but kept in the signature so callers may attach
/// per-step logging in the future.
pub fn copy_dir_contents(src: &Path, dst: &Path, _tag: &str) -> Result<(), String> {
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let dest_path = dst.join(entry.file_name());
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            fs::create_dir_all(&dest_path).map_err(|e| e.to_string())?;
            copy_dir_contents(&entry.path(), &dest_path, _tag)?;
        } else {
            fs::copy(entry.path(), &dest_path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Path to `<crate_path>/Cargo.toml` as a String.
pub fn crate_manifest_path_str(crate_path: &Path) -> Result<String, String> {
    let p = crate_path.join("Cargo.toml");
    p.to_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("non-UTF-8 path: {}", disp(&p)))
}

/// Find the repo root by locating `viewer-ctl.toml`.
///
/// Search order:
///   1. `CARGO_MANIFEST_DIR` ancestors (set by `cargo run`)
///   2. cwd ancestors
pub fn repo_root() -> PathBuf {
    if let Ok(dir) = env::var("CARGO_MANIFEST_DIR") {
        let p = PathBuf::from(&dir);
        for ancestor in p.ancestors() {
            if ancestor.join("viewer-ctl.toml").exists() {
                return ancestor.to_path_buf();
            }
        }
    }
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for ancestor in cwd.ancestors() {
        if ancestor.join("viewer-ctl.toml").exists() {
            return ancestor.to_path_buf();
        }
    }
    cwd
}
