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
    #[serde(default)]
    pub tasks: Vec<Task>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub name: String,
    pub command: Value, // Can be string or array
    pub description: String,
}

// Legacy config format for migration
#[derive(Debug, Deserialize)]
struct LegacyRobinConfig {
    #[serde(default)]
    pub include: Vec<String>,
    pub scripts: HashMap<String, Value>,
}

impl RobinConfig {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(anyhow::anyhow!("No config file found. Run 'robin init' first"));
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        
        let mut config: Self = serde_json::from_str(&content)
            .with_context(|| "The config file exists but contains malformed JSON. Please check the file format.")?;

        // Load and merge included configs
        if !config.include.is_empty() {
            let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
            config = config.merge_includes(base_dir)?;
        }

        Ok(config)
    }

    fn merge_includes(&self, base_dir: &Path) -> Result<Self> {
        let mut merged_tasks = self.tasks.clone();

        for include_path in &self.include {
            let full_path = base_dir.join(include_path);
            let included_config = Self::load(&full_path)
                .with_context(|| format!("Failed to load included config: {}", include_path))?;
            
            // Merge tasks from included config
            for task in included_config.tasks {
                if !merged_tasks.iter().any(|t| t.name == task.name) {
                    merged_tasks.push(task);
                }
            }
        }

        Ok(Self {
            include: self.include.clone(),
            tasks: merged_tasks,
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
        let mut tasks = Vec::new();
        tasks.push(Task {
            name: "clean".to_string(),
            command: Value::String("...".to_string()),
            description: "Clean the project".to_string(),
        });
        tasks.push(Task {
            name: "deploy:staging".to_string(),
            command: Value::String("echo 'ruby deploy tool --staging'".to_string()),
            description: "Deploy to staging environment".to_string(),
        });
        tasks.push(Task {
            name: "deploy:production".to_string(),
            command: Value::String("...".to_string()),
            description: "Deploy to production environment".to_string(),
        });
        tasks.push(Task {
            name: "release:beta".to_string(),
            command: Value::String("...".to_string()),
            description: "Release beta version".to_string(),
        });
        tasks.push(Task {
            name: "release:alpha".to_string(),
            command: Value::String("...".to_string()),
            description: "Release alpha version".to_string(),
        });
        tasks.push(Task {
            name: "release:dev".to_string(),
            command: Value::String("...".to_string()),
            description: "Release dev version".to_string(),
        });

        Self { 
            include: Vec::new(),
            tasks,
        }
    }
    
    pub fn migrate_from_v1(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(anyhow::anyhow!("No legacy config file found for migration"));
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read legacy config file: {}", path.display()))?;
        
        let legacy_config: LegacyRobinConfig = serde_json::from_str(&content)
            .with_context(|| "The legacy config file exists but contains malformed JSON")?;

        let mut tasks = Vec::new();
        
        // Convert scripts to tasks
        for (name, command) in legacy_config.scripts {
            tasks.push(Task {
                name,
                command,
                description: "Migrated from v1 config".to_string(),
            });
        }
        
        Ok(Self {
            include: legacy_config.include,
            tasks,
        })
    }
} 