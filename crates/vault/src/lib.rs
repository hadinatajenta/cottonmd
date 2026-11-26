use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Markdown,
    Canvas,
    Base,
    Pdf,
    Attachment,
}

impl EntryKind {
    pub fn from_extension(ext: Option<&str>) -> Self {
        match ext.map(|s| s.to_ascii_lowercase()).as_deref() {
            Some("md") | Some("markdown") => Self::Markdown,
            Some("canvas") => Self::Canvas,
            Some("base") => Self::Base,
            Some("pdf") => Self::Pdf,
            _ => Self::Attachment,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultFile {
    pub relative_path: String,
    pub stem: String,
    pub extension: Option<String>,
    pub kind: EntryKind,
    pub size_bytes: u64,
    pub modified: Option<SystemTime>,
    pub created: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub struct Vault {
    root: PathBuf,
    config_dir: String,
}

impl Vault {
    pub fn open(root: impl AsRef<Path>) -> io::Result<Self> {
        let canonical = fs::canonicalize(root.as_ref())?;
        if !canonical.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "vault root must be a directory",
            ));
        }
        Ok(Self {
            root: canonical,
            config_dir: ".cottonmd".to_string(),
        })
    }

    pub fn with_config_dir(mut self, config_dir: impl Into<String>) -> Self {
        self.config_dir = config_dir.into();
        self
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn config_dir(&self) -> &str {
        &self.config_dir
    }

    pub fn list_files(&self) -> io::Result<Vec<VaultFile>> {
        let mut entries = Vec::new();
        self.walk(&self.root, &mut entries)?;
        entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
        Ok(entries)
    }

    pub fn read_to_string(&self, relative_path: &str) -> io::Result<String> {
        let full = self.resolve_checked(relative_path)?;
        fs::read_to_string(full)
    }

    pub fn write_string(&self, relative_path: &str, content: &str) -> io::Result<()> {
        let full = self.resolve_checked(relative_path)?;
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(full, content)
    }

    pub fn resolve_checked(&self, relative_path: &str) -> io::Result<PathBuf> {
        let candidate = Path::new(relative_path);
        if candidate.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "absolute paths are not permitted inside vault operations",
            ));
        }
        let mut normalized = self.root.clone();
        for comp in candidate.components() {
            match comp {
                std::path::Component::Normal(part) => normalized.push(part),
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir
                | std::path::Component::RootDir
                | std::path::Component::Prefix(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "path traversal outside vault root is rejected",
                    ));
                }
            }
        }
        Ok(normalized)
    }

    fn walk(&self, dir: &Path, out: &mut Vec<VaultFile>) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();
            if name_str.starts_with('.') || name_str == self.config_dir {
                continue;
            }
            let meta = entry.metadata()?;
            if meta.is_dir() {
                self.walk(&path, out)?;
            } else if meta.is_file() {
                let rel = path
                    .strip_prefix(&self.root)
                    .map_err(io::Error::other)?
                    .to_string_lossy()
                    .replace('\\', "/");
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_string());
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string();
                out.push(VaultFile {
                    relative_path: rel,
                    stem,
                    kind: EntryKind::from_extension(ext.as_deref()),
                    extension: ext,
                    size_bytes: meta.len(),
                    modified: meta.modified().ok(),
                    created: meta.created().ok(),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_path_traversal() {
        let dir = std::env::temp_dir().join("cottonmd_vault_test_traversal");
        let _ = fs::create_dir_all(&dir);
        let vault = Vault::open(&dir).expect("open temp vault");
        assert!(vault.resolve_checked("../etc/passwd").is_err());
        assert!(vault.resolve_checked("/etc/passwd").is_err());
        assert!(vault.resolve_checked("notes/hello.md").is_ok());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn classifies_vault_extensions() {
        assert_eq!(EntryKind::from_extension(Some("md")), EntryKind::Markdown);
        assert_eq!(EntryKind::from_extension(Some("canvas")), EntryKind::Canvas);
        assert_eq!(EntryKind::from_extension(Some("base")), EntryKind::Base);
        assert_eq!(EntryKind::from_extension(Some("PDF")), EntryKind::Pdf);
        assert_eq!(EntryKind::from_extension(Some("png")), EntryKind::Attachment);
    }
}
