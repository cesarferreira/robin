pub const CONFIG_FILE: &str = ".robin.json";
const GITHUB_TEMPLATE_BASE: &str = "https://raw.githubusercontent.com/cesarferreira/robin/refs/heads/main/templates";

pub mod cli;
pub mod config;
pub mod tools;
pub mod utils;
pub mod scripts;

pub use cli::{Cli, Commands};
pub use config::RobinConfig;
pub use tools::{check_environment, update_tools};
pub use utils::{send_notification, split_command_and_args, replace_variables};
pub use scripts::{run_script, list_commands, interactive_mode};

use anyhow::{Context, Result, anyhow};
use reqwest;

#[cfg(not(feature = "test-utils"))]
pub async fn fetch_template(template_name: &str) -> Result<RobinConfig> {
    let url = format!("{}/{}.json", GITHUB_TEMPLATE_BASE, template_name);
    let response = reqwest::get(&url)
        .await
        .with_context(|| format!("Failed to fetch template from: {}", url))?;
    
    if !response.status().is_success() {
        return Err(anyhow!("Template '{}' not found", template_name));
    }

    let content = response.text()
        .await
        .with_context(|| "Failed to read template content")?;
    
    let config: RobinConfig = serde_json::from_str(&content)
        .with_context(|| "Failed to parse template JSON")?;
    
    Ok(config)
}

#[cfg(feature = "test-utils")]
pub async fn fetch_template(_template_name: &str) -> Result<RobinConfig> {
    use std::collections::HashMap;
    let mut scripts = HashMap::new();
    scripts.insert("start".to_string(), serde_json::Value::String("npm start".to_string()));
    Ok(RobinConfig { scripts, include: vec![] })
} 