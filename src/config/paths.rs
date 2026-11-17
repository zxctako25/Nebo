use std::path::{Path, PathBuf};

pub(crate) fn application_data_dir() -> PathBuf {
    let cwd = std::env::current_dir().ok();
    let executable_dir = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf));

    if let Some(dir) = &executable_dir {
        if dir.join("config.yml").exists() || dir.join("forwarding.secret").exists() {
            return dir.clone();
        }
    }
    if let Some(dir) = &cwd {
        if dir.join("config.yml").exists() || dir.join("Cargo.toml").exists() {
            return dir.clone();
        }
    }
    executable_dir.or(cwd).unwrap_or_else(|| PathBuf::from("."))
}
