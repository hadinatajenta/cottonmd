use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const CONFIG_FILE_NAME: &str = "last_vault.txt";

#[cfg(target_os = "windows")]
fn config_file_path() -> Option<PathBuf> {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).or_else(|| {
        std::env::var_os("USERPROFILE")
            .map(|home| PathBuf::from(home).join("AppData").join("Roaming"))
    })?;
    Some(base.join("CottonMD").join(CONFIG_FILE_NAME))
}

#[cfg(target_os = "macos")]
fn config_file_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("CottonMD")
            .join(CONFIG_FILE_NAME),
    )
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn config_file_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("cottonmd").join(CONFIG_FILE_NAME))
}

pub fn load_last_vault_path() -> Option<PathBuf> {
    let path = config_file_path()?;
    let raw = fs::read_to_string(path).ok()?;
    let vault_path = PathBuf::from(raw.trim());
    vault_path.is_dir().then_some(vault_path)
}

pub fn save_last_vault_path(vault_root: &Path) -> io::Result<()> {
    let Some(config_path) = config_file_path() else {
        return Ok(());
    };
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(config_path, vault_root.to_string_lossy().as_bytes())
}
