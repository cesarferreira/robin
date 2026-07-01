use robin::config::{RobinConfig, find_config_from};
use std::fs;
use tempfile::tempdir;

#[test]
fn find_config_walks_up_to_ancestor() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::write(root.join(".robin.json"), r#"{"scripts":{}}"#).unwrap();

    let nested = root.join("a").join("b").join("c");
    fs::create_dir_all(&nested).unwrap();

    let found = find_config_from(&nested).expect("should find config in an ancestor");
    assert_eq!(found, root.join(".robin.json"));
}

#[test]
fn find_config_prefers_the_nearest_config() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::write(root.join(".robin.json"), r#"{"scripts":{}}"#).unwrap();

    let sub = root.join("sub");
    fs::create_dir_all(&sub).unwrap();
    fs::write(sub.join(".robin.json"), r#"{"scripts":{}}"#).unwrap();

    let found = find_config_from(&sub).expect("should find nearest config");
    assert_eq!(found, sub.join(".robin.json"));
}

#[test]
fn find_config_returns_none_when_absent() {
    let dir = tempdir().unwrap();
    assert!(find_config_from(dir.path()).is_none());
}

#[test]
fn load_missing_file_gives_helpful_error() {
    let dir = tempdir().unwrap();
    let path = dir.path().join(".robin.json");

    let err = RobinConfig::load(&path).unwrap_err();
    assert!(
        err.to_string().contains("No .robin.json found"),
        "unexpected error: {err}"
    );
}

#[test]
fn load_malformed_json_gives_helpful_error() {
    let dir = tempdir().unwrap();
    let path = dir.path().join(".robin.json");
    fs::write(&path, "{ this is not valid json ").unwrap();

    let err = RobinConfig::load(&path).unwrap_err();
    assert!(
        err.to_string().contains("malformed JSON"),
        "unexpected error: {err}"
    );
}

#[test]
fn load_defaults_include_to_empty() {
    // `include` is optional; a config without it must still load.
    let dir = tempdir().unwrap();
    let path = dir.path().join(".robin.json");
    fs::write(&path, r#"{"scripts":{"build":"cargo build"}}"#).unwrap();

    let config = RobinConfig::load(&path).unwrap();
    assert!(config.include.is_empty());
    assert_eq!(config.scripts.len(), 1);
}

#[test]
fn includes_are_merged_with_base_taking_precedence() {
    let dir = tempdir().unwrap();
    let base = dir.path().join(".robin.json");
    let child = dir.path().join("child.json");

    fs::write(
        &base,
        r#"{"include":["child.json"],"scripts":{"shared":"base wins","only_base":"b"}}"#,
    )
    .unwrap();
    fs::write(
        &child,
        r#"{"scripts":{"shared":"child loses","only_child":"c"}}"#,
    )
    .unwrap();

    let config = RobinConfig::load(&base).unwrap();

    assert_eq!(
        config.scripts.get("shared").unwrap().as_str().unwrap(),
        "base wins"
    );
    assert_eq!(
        config.scripts.get("only_base").unwrap().as_str().unwrap(),
        "b"
    );
    assert_eq!(
        config.scripts.get("only_child").unwrap().as_str().unwrap(),
        "c"
    );
    assert_eq!(config.scripts.len(), 3);
}

#[test]
fn includes_resolve_relative_to_config_dir() {
    // The included path is resolved against the parent of the config file,
    // not the process's current directory.
    let dir = tempdir().unwrap();
    let sub = dir.path().join("sub");
    fs::create_dir(&sub).unwrap();

    let base = sub.join(".robin.json");
    fs::write(&base, r#"{"include":["extra.json"],"scripts":{"a":"1"}}"#).unwrap();
    fs::write(sub.join("extra.json"), r#"{"scripts":{"b":"2"}}"#).unwrap();

    let config = RobinConfig::load(&base).unwrap();
    assert!(config.scripts.contains_key("a"));
    assert!(config.scripts.contains_key("b"));
}

#[test]
fn missing_include_reports_which_file_failed() {
    let dir = tempdir().unwrap();
    let base = dir.path().join(".robin.json");
    fs::write(&base, r#"{"include":["does_not_exist.json"],"scripts":{}}"#).unwrap();

    let err = RobinConfig::load(&base).unwrap_err();
    assert!(
        err.to_string().contains("does_not_exist.json"),
        "error should name the missing include: {err}"
    );
}

#[test]
fn schema_field_is_preserved_across_load_and_save() {
    // A user-provided $schema must survive edits (add/remove/rename all save).
    let dir = tempdir().unwrap();
    let path = dir.path().join(".robin.json");
    fs::write(
        &path,
        r#"{"$schema":"https://example.com/robin.json","scripts":{"a":"echo hi"}}"#,
    )
    .unwrap();

    let mut config = RobinConfig::load(&path).unwrap();
    assert_eq!(config.schema.as_deref(), Some("https://example.com/robin.json"));

    config
        .scripts
        .insert("b".to_string(), serde_json::Value::String("echo bye".into()));
    config.save(&path).unwrap();

    let reloaded = RobinConfig::load(&path).unwrap();
    assert_eq!(
        reloaded.schema.as_deref(),
        Some("https://example.com/robin.json")
    );
}

#[test]
fn created_template_points_at_the_published_schema() {
    let config = RobinConfig::create_template();
    assert_eq!(config.schema.as_deref(), Some(robin::config::SCHEMA_URL));
}

#[test]
fn save_then_load_roundtrips() {
    let dir = tempdir().unwrap();
    let path = dir.path().join(".robin.json");

    let config = RobinConfig::create_template();
    config.save(&path).unwrap();
    assert!(path.exists());

    let loaded = RobinConfig::load(&path).unwrap();
    assert_eq!(loaded.scripts.len(), config.scripts.len());
}
