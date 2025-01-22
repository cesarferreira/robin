use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct RobinConfig {
    pub scripts: HashMap<String, String>,
}

impl RobinConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        
        serde_json::from_str(&content)
            .with_context(|| "Failed to parse .robin.json")
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;
        
        fs::write(path, content)
            .with_context(|| format!("Failed to write config to: {}", path.display()))?;
        
        Ok(())
    }

    pub fn create_template() -> Self {
        let mut scripts = HashMap::new();
        scripts.insert("clean".to_string(), "...".to_string());
        scripts.insert("deploy staging".to_string(), "echo 'ruby deploy tool --staging'".to_string());
        scripts.insert("deploy production".to_string(), "...".to_string());
        scripts.insert("release beta".to_string(), "...".to_string());
        scripts.insert("release alpha".to_string(), "...".to_string());
        scripts.insert("release dev".to_string(), "...".to_string());

        Self { scripts }
    }
} 