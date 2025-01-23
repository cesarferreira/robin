use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct RobinConfig {
    #[serde(default)]
    pub include: Vec<String>,
    pub scripts: HashMap<String, Value>,
}

impl RobinConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        
        let mut config: Self = serde_json::from_str(&content)
            .with_context(|| "Failed to parse .robin.json")?;

        // Load and merge included configs
        if !config.include.is_empty() {
            let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
            config = config.merge_includes(base_dir)?;
        }

        Ok(config)
    }

    fn merge_includes(&self, base_dir: &Path) -> Result<Self> {
        let mut merged_scripts = self.scripts.clone();

        for include_path in &self.include {
            let full_path = base_dir.join(include_path);
            let included_config = Self::load(&full_path)
                .with_context(|| format!("Failed to load included config: {}", include_path))?;
            
            // Merge scripts from included config
            for (key, value) in included_config.scripts {
                if !merged_scripts.contains_key(&key) {
                    merged_scripts.insert(key, value);
                }
            }
        }

        Ok(Self {
            include: self.include.clone(),
            scripts: merged_scripts,
        })
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
        scripts.insert("clean".to_string(), Value::String("...".to_string()));
        scripts.insert("deploy staging".to_string(), Value::String("echo 'ruby deploy tool --staging'".to_string()));
        scripts.insert("deploy production".to_string(), Value::String("...".to_string()));
        scripts.insert("release beta".to_string(), Value::String("...".to_string()));
        scripts.insert("release alpha".to_string(), Value::String("...".to_string()));
        scripts.insert("release dev".to_string(), Value::String("...".to_string()));

        Self { 
            include: Vec::new(),
            scripts 
        }
    }
} 