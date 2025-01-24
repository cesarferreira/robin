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