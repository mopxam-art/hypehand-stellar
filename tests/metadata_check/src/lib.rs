use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;

pub const PUBLISHED_CRATES: &[&str] = &[
    "crates/cli",
    "crates/config",
    "crates/events",
    "crates/keys",
    "crates/metrics",
    "crates/telemetry",
];

pub fn find_workspace_root() -> PathBuf {
    let mut current = std::env::current_dir().expect("Failed to get current dir");
    while !current.join("Cargo.toml").exists() || !current.join("crates").exists() {
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }
    current
}

pub fn validate_root_manifest(root: &Path) {
    let manifest_path = root.join("Cargo.toml");
    assert!(manifest_path.exists(), "Root Cargo.toml must exist");

    let content = fs::read_to_string(&manifest_path).expect("Read root Cargo.toml");
    let parsed: Value = toml::from_str(&content).expect("Parse root Cargo.toml");

    let workspace = parsed.get("workspace").expect("workspace section must exist");
    let ws_package = workspace.get("package").expect("workspace.package must exist");

    assert!(ws_package.get("version").is_some(), "version required in workspace.package");
    assert!(ws_package.get("edition").is_some(), "edition required in workspace.package");
    assert!(ws_package.get("authors").is_some(), "authors required in workspace.package");
    assert!(ws_package.get("license").is_some(), "license required in workspace.package");
    assert!(ws_package.get("repository").is_some(), "repository required in workspace.package");
    assert!(ws_package.get("homepage").is_some(), "homepage required in workspace.package");

    let keywords = ws_package.get("keywords").and_then(Value::as_array).expect("keywords array required");
    assert!(!keywords.is_empty() && keywords.len() <= 5, "keywords count must be between 1 and 5");

    let categories = ws_package.get("categories").and_then(Value::as_array).expect("categories array required");
    assert!(!categories.is_empty() && categories.len() <= 5, "categories count must be between 1 and 5");
}

pub fn validate_crate_manifest(root: &Path, crate_rel_path: &str) {
    let crate_dir = root.join(crate_rel_path);
    let manifest_path = crate_dir.join("Cargo.toml");
    assert!(manifest_path.exists(), "Manifest {:?} must exist", manifest_path);

    let content = fs::read_to_string(&manifest_path).expect("Read crate Cargo.toml");
    let parsed: Value = toml::from_str(&content).expect("Parse crate Cargo.toml");

    let package = parsed.get("package").expect("package section must exist");

    let name = package.get("name").and_then(Value::as_str).expect("package.name required");
    assert!(!name.is_empty(), "package.name cannot be empty");

    let description = package.get("description").and_then(Value::as_str).expect("package.description required");
    assert!(description.len() >= 15, "description must be descriptive (>= 15 chars)");

    let readme = package.get("readme").and_then(Value::as_str).expect("package.readme required");
    let readme_path = crate_dir.join(readme);
    assert!(readme_path.exists(), "README at {:?} must exist", readme_path);
    let readme_content = fs::read_to_string(&readme_path).expect("Read README");
    assert!(!readme_content.trim().is_empty(), "README must not be empty");

    let include = package.get("include").and_then(Value::as_array).expect("package.include required");
    assert!(!include.is_empty(), "include list must not be empty");

    // Ensure publish = false is NOT set
    if let Some(publish) = package.get("publish") {
        if let Some(is_publish) = publish.as_bool() {
            assert!(is_publish, "publish must not be false for publishable crate {}", name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_license_files_exist() {
        let root = find_workspace_root();
        let mit = root.join("LICENSE-MIT");
        let apache = root.join("LICENSE-APACHE");

        assert!(mit.exists(), "LICENSE-MIT must exist");
        assert!(apache.exists(), "LICENSE-APACHE must exist");
        assert!(fs::read_to_string(&mit).unwrap().contains("MIT License"));
        assert!(fs::read_to_string(&apache).unwrap().contains("Apache License"));
    }

    #[test]
    fn test_workspace_root_metadata() {
        let root = find_workspace_root();
        validate_root_manifest(&root);
    }

    #[test]
    fn test_all_crates_metadata_complete() {
        let root = find_workspace_root();
        for crate_path in PUBLISHED_CRATES {
            validate_crate_manifest(&root, crate_path);
        }
    }
}
