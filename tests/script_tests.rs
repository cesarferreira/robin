mod common;

use robin::scripts::{run_script, list_commands};
use robin::config::RobinConfig;
use serde_json::Value;

#[tokio::test]
async fn test_list_commands() {
    let (_temp_dir, config_path) = common::setup().await;
    let mut config = RobinConfig::create_template();
    
    config.scripts.insert(
        "test1".to_string(),
        Value::String("echo 'test1'".to_string())
    );
    config.scripts.insert(
        "test2".to_string(),
        Value::String("echo 'test2'".to_string())
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
        Value::String("echo 'test2'".to_string())
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