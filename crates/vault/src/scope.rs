use std::fs;
use std::io;
use std::path::Path;

use serde::Deserialize;

pub const SCOPE_FILE_NAME: &str = "vault.json";
const DEFAULT_EXCLUDED_DIRECTORIES: [&str; 3] = ["node_modules", "vendor", "target"];

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VaultScope {
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

impl VaultScope {
    pub fn load(config_dir_path: &Path) -> io::Result<Self> {
        let scope_path = config_dir_path.join(SCOPE_FILE_NAME);
        let raw_scope = match fs::read_to_string(&scope_path) {
            Ok(raw_scope) => raw_scope,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(err) => return Err(err),
        };
        let mut scope: Self = serde_json::from_str(&raw_scope).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid {}: {err}", scope_path.display()),
            )
        })?;
        scope.include = scope.include.iter().filter_map(|path| normalize_folder(path)).collect();
        Ok(scope)
    }

    pub fn should_enter_directory(&self, relative_dir: &str, dir_name: &str) -> bool {
        if self.is_excluded_name(dir_name) {
            return false;
        }
        self.include.is_empty()
            || self.include.iter().any(|included| {
                is_same_or_inside(relative_dir, included) || is_same_or_inside(included, relative_dir)
            })
    }

    pub fn should_index_file(&self, relative_path: &str) -> bool {
        self.include.is_empty()
            || self
                .include
                .iter()
                .any(|included| is_same_or_inside(relative_path, included))
    }

    fn is_excluded_name(&self, dir_name: &str) -> bool {
        DEFAULT_EXCLUDED_DIRECTORIES.contains(&dir_name)
            || self.exclude.iter().any(|excluded| excluded == dir_name)
    }
}

fn normalize_folder(path: &str) -> Option<String> {
    let trimmed = path.trim().trim_matches('/');
    (!trimmed.is_empty()).then(|| trimmed.replace('\\', "/"))
}

fn is_same_or_inside(path: &str, folder: &str) -> bool {
    path == folder
        || path
            .strip_prefix(folder)
            .is_some_and(|remainder| remainder.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope(include: &[&str], exclude: &[&str]) -> VaultScope {
        VaultScope {
            include: include.iter().map(|path| path.to_string()).collect(),
            exclude: exclude.iter().map(|name| name.to_string()).collect(),
        }
    }

    #[test]
    fn empty_scope_indexes_everything_except_default_heavy_directories() {
        let open_scope = VaultScope::default();
        assert!(open_scope.should_enter_directory("docs", "docs"));
        assert!(open_scope.should_index_file("notes/a.md"));
        assert!(!open_scope.should_enter_directory("app/node_modules", "node_modules"));
        assert!(!open_scope.should_enter_directory("service/vendor", "vendor"));
    }

    #[test]
    fn include_limits_walk_to_listed_folders_and_their_ancestors() {
        let limited = scope(&["_library", "docs/sprints"], &["archive"]);
        assert!(limited.should_enter_directory("docs", "docs"));
        assert!(limited.should_enter_directory("docs/sprints/s1", "s1"));
        assert!(!limited.should_enter_directory("docs/other", "other"));
        assert!(!limited.should_enter_directory("arctic", "arctic"));
        assert!(!limited.should_enter_directory("_library/archive", "archive"));
        assert!(limited.should_index_file("_library/AGENTS.md"));
        assert!(!limited.should_index_file("docs/README.md"));
        assert!(!limited.should_index_file("_library_old/x.md"));
    }

    #[test]
    fn loads_and_normalizes_scope_file_and_rejects_invalid_json() {
        let config_dir = std::env::temp_dir().join("cottonmd_scope_test_config");
        fs::create_dir_all(&config_dir).expect("create config dir");
        fs::write(config_dir.join(SCOPE_FILE_NAME), r#"{"include": ["/docs/", " "]}"#)
            .expect("write scope");
        assert_eq!(VaultScope::load(&config_dir).expect("load").include, vec!["docs"]);
        fs::write(config_dir.join(SCOPE_FILE_NAME), r#"{"includes": []}"#).expect("write scope");
        let load_error = VaultScope::load(&config_dir).expect_err("unknown field must fail");
        assert_eq!(load_error.kind(), io::ErrorKind::InvalidData);
        fs::remove_dir_all(&config_dir).expect("cleanup");
        assert_eq!(VaultScope::load(&config_dir).expect("missing file"), VaultScope::default());
    }
}
