#![cfg(feature = "test-utils")]
mod common;

use robin::config::RobinConfig;
use robin::fetch_template;

#[tokio::test]
async fn test_init_without_template() {
    let (_temp_dir, config_path) = common::setup().await;
    
    // Test init without template
    let config = RobinConfig::create_template();
    config.save(&config_path).unwrap();
    
    assert!(config_path.exists());
    let loaded_config = RobinConfig::load(&config_path).unwrap();
    assert_eq!(loaded_config.scripts.len(), config.scripts.len());
}

#[tokio::test]
async fn test_init_with_template() {
    let (_temp_dir, config_path) = common::setup().await;
    
    let config = fetch_template("node").await.unwrap();
    config.save(&config_path).unwrap();
    
    assert!(config_path.exists());
    let loaded_config = RobinConfig::load(&config_path).unwrap();
    assert!(loaded_config.scripts.contains_key("start"));
}

#[tokio::test]
async fn test_add_command() {
    let (_temp_dir, config_path) = common::setup().await;
    let mut config = RobinConfig::create_template();
    
    let script_name = "test";
    let script_content = "echo 'test'";
    config.scripts.insert(
        script_name.to_string(),
        serde_json::Value::String(script_content.to_string())
    );
    config.save(&config_path).unwrap();
    
    let loaded_config = RobinConfig::load(&config_path).unwrap();
    assert_eq!(
        loaded_config.scripts.get(script_name).unwrap().as_str().unwrap(),
        script_content
    );
} 