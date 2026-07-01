mod common;

use robin::config::RobinConfig;
use robin::scripts::{
    command_lines, list_commands, resolve_task_command, run_script, run_script_in,
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tempfile::tempdir;

#[test]
fn run_script_in_uses_the_given_working_directory() {
    // A marker file exists only inside the temp dir; the command succeeds only
    // if it runs there.
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("marker.txt"), "hi").unwrap();

    let script = Value::String("test -f marker.txt".to_string());
    assert!(run_script_in(&script, false, Some(dir.path())).is_ok());
}

#[test]
fn run_script_in_without_dir_does_not_see_the_marker() {
    // Same command, but run from the process CWD (the repo root), where the
    // marker does not exist — so it must fail.
    let script = Value::String("test -f this_marker_should_not_exist_12345.txt".to_string());
    assert!(run_script_in(&script, false, None).is_err());
}

fn scripts_from(pairs: &[(&str, Value)]) -> HashMap<String, Value> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect()
}

#[test]
fn command_lines_splits_string_and_array() {
    assert_eq!(command_lines(&json!("cargo build")), vec!["cargo build"]);
    assert_eq!(
        command_lines(&json!(["a", "b"])),
        vec!["a".to_string(), "b".to_string()]
    );
    assert!(command_lines(&json!(42)).is_empty());
}

#[test]
fn resolve_leaves_plain_string_unchanged() {
    let scripts = scripts_from(&[("build", json!("cargo build"))]);
    let resolved = resolve_task_command(&json!("cargo build"), &scripts).unwrap();
    assert_eq!(resolved, json!("cargo build"));
}

#[test]
fn resolve_expands_reference_into_referenced_commands() {
    let scripts = scripts_from(&[
        ("build", json!("cargo build")),
        ("deploy", json!(["@build", "scp app server:/srv"])),
    ]);
    let resolved = resolve_task_command(&json!(["@build", "scp app server:/srv"]), &scripts).unwrap();
    assert_eq!(resolved, json!(["cargo build", "scp app server:/srv"]));
}

#[test]
fn resolve_expands_nested_references() {
    let scripts = scripts_from(&[
        ("clean", json!("rm -rf build")),
        ("build", json!(["@clean", "cargo build"])),
        ("ship", json!(["@build", "scp app server:/srv"])),
    ]);
    let resolved = resolve_task_command(&json!("@ship"), &scripts).unwrap();
    assert_eq!(
        resolved,
        json!(["rm -rf build", "cargo build", "scp app server:/srv"])
    );
}

#[test]
fn resolve_detects_reference_cycles() {
    let scripts = scripts_from(&[("a", json!("@b")), ("b", json!("@a"))]);
    let err = resolve_task_command(&json!("@a"), &scripts).unwrap_err();
    assert!(
        err.to_string().contains("Cycle detected"),
        "expected cycle error, got: {err}"
    );
}

#[test]
fn resolve_errors_on_unknown_reference() {
    let scripts = scripts_from(&[("a", json!("@missing"))]);
    let err = resolve_task_command(&json!("@a"), &scripts).unwrap_err();
    assert!(
        err.to_string().contains("not found"),
        "expected not-found error, got: {err}"
    );
}

#[test]
fn resolve_follows_reference_into_object_form() {
    let scripts = scripts_from(&[
        ("build", json!({ "cmd": "cargo build", "desc": "Build" })),
        ("ship", json!(["@build", "deploy"])),
    ]);
    let resolved = resolve_task_command(&json!("@ship"), &scripts).unwrap();
    assert_eq!(resolved, json!(["cargo build", "deploy"]));
}

#[tokio::test]
async fn test_list_commands() {
    let (_temp_dir, config_path) = common::setup().await;
    let mut config = RobinConfig::create_template();

    config.scripts.insert(
        "test1".to_string(),
        Value::String("echo 'test1'".to_string()),
    );
    config.scripts.insert(
        "test2".to_string(),
        Value::String("echo 'test2'".to_string()),
    );

    config.save(&config_path).unwrap();

    let result = list_commands(&config_path);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_script() {
    let script = Value::String("echo 'test'".to_string());
    let result = run_script(&script, false);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_multiple_scripts() {
    let scripts = Value::Array(vec![
        Value::String("echo 'test1'".to_string()),
        Value::String("echo 'test2'".to_string()),
    ]);
    let result = run_script(&scripts, false);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_script_with_error() {
    let script = Value::String("nonexistent_command".to_string());
    let result = run_script(&script, false);
    assert!(result.is_err());
}

#[test]
fn run_script_array_stops_on_first_failure() {
    // The failing command is in the middle; the sequence must error out.
    let scripts = Value::Array(vec![
        Value::String("echo first".to_string()),
        Value::String("false".to_string()),
        Value::String("echo should-not-matter".to_string()),
    ]);
    let result = run_script(&scripts, false);
    assert!(result.is_err());
}

#[test]
fn run_script_rejects_invalid_script_type() {
    let script = Value::from(42);
    let err = run_script(&script, false).unwrap_err();
    assert!(err.to_string().contains("Invalid script type"), "{err}");
}

#[test]
fn list_commands_errors_without_config() {
    use std::path::Path;
    let result = list_commands(Path::new("/definitely/not/here/.robin.json"));
    assert!(result.is_err());
}
