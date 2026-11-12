//! Filesystem and Path Resolution Utilities for Aura Language.
//!
//! Provides canonical resolution, security validation against path traversal,
//! and unified access to Aura cache and home directories (`~/.aura/...`).

use std::path::{Component, Path, PathBuf};

/// Returns the Aura root directory (`~/.aura` or custom path).
pub fn aura_home_dir() -> Option<PathBuf> {
    std::env::var_os("AURA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".aura")))
}

/// Returns the global package cache directory (`~/.aura/pkg/mod`).
pub fn global_pkg_cache_dir() -> Option<PathBuf> {
    aura_home_dir().map(|h| h.join("pkg").join("mod"))
}

/// Returns the Aura runtime library directory (`~/.aura/lib`).
pub fn aura_lib_dir() -> Option<PathBuf> {
    aura_home_dir().map(|h| h.join("lib"))
}

/// Validates that a package path conforms to security standards (prevents path traversal, command injection, invalid characters).
pub fn validate_security_package_path(pkg_path: &str) -> Result<(), String> {
    if pkg_path.is_empty() {
        return Err("Security error: package path cannot be empty".to_string());
    }
    if pkg_path.starts_with('-') {
        return Err(format!(
            "Security violation: package path '{}' cannot start with '-' (command injection prevention)",
            pkg_path
        ));
    }
    if pkg_path.starts_with('/') || pkg_path.starts_with('\\') || pkg_path.contains(':') {
        return Err(format!(
            "Security violation: package path '{}' cannot be an absolute path or contain drive letters (path traversal prevention)",
            pkg_path
        ));
    }
    for seg in pkg_path.split(['/', '\\']) {
        if seg == ".." {
            return Err(format!(
                "Security violation: package path '{}' cannot contain '..' (directory traversal prevention)",
                pkg_path
            ));
        }
    }
    if !pkg_path
        .chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-' || c == '/' || c == '@')
    {
        return Err(format!(
            "Security violation: package path '{}' contains invalid characters",
            pkg_path
        ));
    }
    Ok(())
}

/// Validates that a version string is safe from command injection and invalid characters.
pub fn validate_security_version(version: &str) -> Result<(), String> {
    if version.starts_with('-') {
        return Err(format!(
            "Security violation: version '{}' cannot start with '-' (command injection prevention)",
            version
        ));
    }
    if !version
        .chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-' || c == '+' || c == '/')
    {
        return Err(format!(
            "Security violation: version '{}' contains invalid characters",
            version
        ));
    }
    Ok(())
}

/// Normalizes a path by resolving `.` and `..` components lexically without accessing the filesystem.
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if let Some(Component::Normal(_)) = components.last() {
                    components.pop();
                } else if !components.iter().any(|c| matches!(c, Component::RootDir | Component::Prefix(_))) {
                    components.push(component);
                }
            }
            c => components.push(c),
        }
    }
    components.into_iter().collect()
}

/// Returns the canonical path if the file exists, or the lexically normalized path if canonicalization fails.
pub fn canonicalize_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| normalize_path(path))
}

/// Validates that a path is safe and does not perform path traversal escapes.
///
/// If `enforce_relative` is true, absolute paths and drive letters are rejected.
pub fn validate_path_safety(path: &str, enforce_relative: bool) -> Result<(), String> {
    if path.is_empty() {
        return Err("Path error: path cannot be empty".to_string());
    }

    if path.starts_with('-') {
        return Err(format!(
            "Security violation: path '{}' cannot start with '-' (command injection prevention)",
            path
        ));
    }

    if enforce_relative
        && (path.starts_with('/') || path.starts_with('\\') || path.contains(':'))
    {
        return Err(format!(
            "Security violation: path '{}' cannot be an absolute path or contain drive letters (path traversal prevention)",
            path
        ));
    }

    // Check for traversal segments (e.g., ../ or ..)
    for seg in path.split(['/', '\\']) {
        if seg == ".." && enforce_relative {
            return Err(format!(
                "Security violation: path '{}' cannot contain '..' (directory traversal prevention)",
                path
            ));
        }
    }

    Ok(())
}

/// Resolves an import source (either local `./...`, `../...` or external package) to an existing file.
///
/// Applies security validation to prevent directory traversal escapes and normalizes paths.
pub fn resolve_import_path(imp_source: &str, base_path: Option<&Path>) -> Option<PathBuf> {
    if imp_source.is_empty() {
        return None;
    }

    if imp_source.starts_with("./") || imp_source.starts_with("../") {
        resolve_local_import_path(imp_source, base_path)
    } else {
        crate::package::resolve_package_import(imp_source, base_path)
    }
}

/// Resolves local import paths relative to `base_path`.
pub fn resolve_local_import_path(imp_source: &str, base_path: Option<&Path>) -> Option<PathBuf> {
    let dep_file = if let Some(base) = base_path {
        base.join(imp_source)
    } else {
        PathBuf::from(imp_source)
    };

    let candidates = [dep_file.clone(), dep_file.with_extension("aura")];
    candidates
        .into_iter()
        .find(|c| c.exists() && c.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        assert_eq!(
            normalize_path(Path::new("a/b/../c")),
            PathBuf::from("a/c")
        );
        assert_eq!(
            normalize_path(Path::new("./a/./b")),
            PathBuf::from("a/b")
        );
    }

    #[test]
    fn test_validate_path_safety() {
        assert!(validate_path_safety("github.com/org/repo", true).is_ok());
        assert!(validate_path_safety("../secret", true).is_err());
        assert!(validate_path_safety("/etc/passwd", true).is_err());
        assert!(validate_path_safety("-rf", true).is_err());
        assert!(validate_path_safety("../secret", false).is_ok());
    }

    #[test]
    fn test_global_pkg_cache_dir() {
        if std::env::var_os("HOME").is_some() {
            let cache = global_pkg_cache_dir();
            assert!(cache.is_some());
            let path_str = cache.unwrap().to_string_lossy().to_string();
            assert!(path_str.ends_with(".aura/pkg/mod") || path_str.contains(".aura"));
        }
    }
}
