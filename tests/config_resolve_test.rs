use std::{fs, path::Path};

use gobject_linter::config::{
    CONFIG_FILENAMES, LEGACY_CONFIG_FILENAMES, has_config_file, resolve_config_in_dir,
    resolve_config_path,
};

fn touch(path: &Path) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, "").unwrap();
}

// --- has_config_file ---

#[test]
fn has_config_file_canonical() {
    let dir = tempfile::tempdir().unwrap();
    assert!(!has_config_file(dir.path()));

    touch(&dir.path().join("gobject-linter.toml"));
    assert!(has_config_file(dir.path()));
}

#[test]
fn has_config_file_dot() {
    let dir = tempfile::tempdir().unwrap();
    touch(&dir.path().join(".gobject-linter.toml"));
    assert!(has_config_file(dir.path()));
}

#[test]
fn has_config_file_legacy() {
    let dir = tempfile::tempdir().unwrap();
    touch(&dir.path().join("goblint.toml"));
    assert!(has_config_file(dir.path()));
}

#[test]
fn has_config_file_empty_dir() {
    let dir = tempfile::tempdir().unwrap();
    assert!(!has_config_file(dir.path()));
}

// --- resolve_config_in_dir ---

#[test]
fn resolve_in_dir_returns_none_when_empty() {
    let dir = tempfile::tempdir().unwrap();
    assert!(resolve_config_in_dir(dir.path()).is_none());
}

#[test]
fn resolve_in_dir_prefers_canonical_over_dot() {
    let dir = tempfile::tempdir().unwrap();
    touch(&dir.path().join("gobject-linter.toml"));
    touch(&dir.path().join(".gobject-linter.toml"));

    let resolved = resolve_config_in_dir(dir.path()).unwrap();
    assert_eq!(resolved, dir.path().join("gobject-linter.toml"));
}

#[test]
fn resolve_in_dir_prefers_dot_over_legacy() {
    let dir = tempfile::tempdir().unwrap();
    touch(&dir.path().join(".gobject-linter.toml"));
    touch(&dir.path().join("goblint.toml"));

    let resolved = resolve_config_in_dir(dir.path()).unwrap();
    assert_eq!(resolved, dir.path().join(".gobject-linter.toml"));
}

#[test]
fn resolve_in_dir_falls_back_to_legacy() {
    let dir = tempfile::tempdir().unwrap();
    touch(&dir.path().join("goblint.toml"));

    let resolved = resolve_config_in_dir(dir.path()).unwrap();
    assert_eq!(resolved, dir.path().join("goblint.toml"));
}

// --- resolve_config_path: explicit config ---

#[test]
fn explicit_config_exists() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("custom.toml");
    touch(&config);

    let result = resolve_config_path(dir.path(), dir.path(), Some(&config));
    assert_eq!(result, Ok(config));
}

#[test]
fn explicit_config_missing() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("missing.toml");

    let result = resolve_config_path(dir.path(), dir.path(), Some(&config));
    assert_eq!(result, Err(config));
}

// --- resolve_config_path: auto-discovery priority ---

#[test]
fn target_canonical_wins_over_everything() {
    let target = tempfile::tempdir().unwrap();
    let base = tempfile::tempdir().unwrap();

    // Place config files everywhere
    touch(&target.path().join("gobject-linter.toml"));
    touch(&target.path().join(".gobject-linter.toml"));
    touch(&target.path().join("goblint.toml"));
    touch(&base.path().join("gobject-linter.toml"));
    touch(&base.path().join(".gobject-linter.toml"));
    touch(&base.path().join("goblint.toml"));

    let result = resolve_config_path(target.path(), base.path(), None).unwrap();
    assert_eq!(result, target.path().join("gobject-linter.toml"));
}

#[test]
fn target_dot_over_base_canonical() {
    let target = tempfile::tempdir().unwrap();
    let base = tempfile::tempdir().unwrap();

    touch(&target.path().join(".gobject-linter.toml"));
    touch(&base.path().join("gobject-linter.toml"));

    let result = resolve_config_path(target.path(), base.path(), None).unwrap();
    assert_eq!(result, target.path().join(".gobject-linter.toml"));
}

#[test]
fn base_canonical_over_target_legacy() {
    let target = tempfile::tempdir().unwrap();
    let base = tempfile::tempdir().unwrap();

    touch(&target.path().join("goblint.toml"));
    touch(&base.path().join("gobject-linter.toml"));

    let result = resolve_config_path(target.path(), base.path(), None).unwrap();
    assert_eq!(result, base.path().join("gobject-linter.toml"));
}

#[test]
fn base_dot_over_target_legacy() {
    let target = tempfile::tempdir().unwrap();
    let base = tempfile::tempdir().unwrap();

    touch(&target.path().join("goblint.toml"));
    touch(&base.path().join(".gobject-linter.toml"));

    let result = resolve_config_path(target.path(), base.path(), None).unwrap();
    assert_eq!(result, base.path().join(".gobject-linter.toml"));
}

#[test]
fn target_legacy_over_base_legacy() {
    let target = tempfile::tempdir().unwrap();
    let base = tempfile::tempdir().unwrap();

    touch(&target.path().join("goblint.toml"));
    touch(&base.path().join("goblint.toml"));

    let result = resolve_config_path(target.path(), base.path(), None).unwrap();
    assert_eq!(result, target.path().join("goblint.toml"));
}

#[test]
fn falls_back_to_base_legacy() {
    let target = tempfile::tempdir().unwrap();
    let base = tempfile::tempdir().unwrap();

    touch(&base.path().join("goblint.toml"));

    let result = resolve_config_path(target.path(), base.path(), None).unwrap();
    assert_eq!(result, base.path().join("goblint.toml"));
}

#[test]
fn no_config_anywhere_returns_default() {
    let target = tempfile::tempdir().unwrap();
    let base = tempfile::tempdir().unwrap();

    let result = resolve_config_path(target.path(), base.path(), None).unwrap();
    assert_eq!(result, base.path().join("gobject-linter.toml"));
}

// --- resolve_config_path: same directory for target and base ---

#[test]
fn same_dir_picks_canonical() {
    let dir = tempfile::tempdir().unwrap();
    touch(&dir.path().join("gobject-linter.toml"));
    touch(&dir.path().join(".gobject-linter.toml"));
    touch(&dir.path().join("goblint.toml"));

    let result = resolve_config_path(dir.path(), dir.path(), None).unwrap();
    assert_eq!(result, dir.path().join("gobject-linter.toml"));
}

#[test]
fn same_dir_dot_only() {
    let dir = tempfile::tempdir().unwrap();
    touch(&dir.path().join(".gobject-linter.toml"));

    let result = resolve_config_path(dir.path(), dir.path(), None).unwrap();
    assert_eq!(result, dir.path().join(".gobject-linter.toml"));
}

#[test]
fn same_dir_legacy_only() {
    let dir = tempfile::tempdir().unwrap();
    touch(&dir.path().join("goblint.toml"));

    let result = resolve_config_path(dir.path(), dir.path(), None).unwrap();
    assert_eq!(result, dir.path().join("goblint.toml"));
}

// --- constants are consistent ---

#[test]
fn config_filenames_order() {
    assert_eq!(
        CONFIG_FILENAMES,
        &["gobject-linter.toml", ".gobject-linter.toml"]
    );
    assert_eq!(LEGACY_CONFIG_FILENAMES, &["goblint.toml"]);
}
