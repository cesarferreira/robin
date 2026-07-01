use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::CONFIG_FILE;

/// Walks up from `start` (inclusive) looking for the first directory that
/// contains a `.robin.json`. Returns the path to that config, or `None` when no
/// ancestor holds one. This lets `robin` be run from any subdirectory of a
/// project, mirroring how `git` and `cargo` locate their root files.
pub fn find_config_from(start: &Path) -> Option<PathBuf> {
    let mut dir = Some(start);
    while let Some(current) = dir {
        let candidate = current.join(CONFIG_FILE);
        if candidate.is_file() {
            return Some(candidate);
        }
        dir = current.parent();
    }
    None
}

/// Resolves the config path for read commands: the nearest `.robin.json` found
/// by walking up from the current directory, falling back to `./.robin.json`
/// so that "not found" errors still name a sensible location.
pub fn find_config_path() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    find_config_from(&cwd).unwrap_or_else(|| cwd.join(CONFIG_FILE))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RobinConfig {
    #[serde(default)]
    pub include: Vec<String>,
    pub scripts: HashMap<String, Value>,
}

impl RobinConfig {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "No .robin.json found. Run 'robin init' first"
            ));
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let mut config: Self = serde_json::from_str(&content)
            .with_context(|| "The .robin.json file exists but contains malformed JSON. Please check the file format.")?;

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

            // Merge scripts from included config; existing keys take precedence.
            for (key, value) in included_config.scripts {
                merged_scripts.entry(key).or_insert(value);
            }
        }

        Ok(Self {
            include: self.include.clone(),
            scripts: merged_scripts,
        })
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content =
            serde_json::to_string_pretty(self).with_context(|| "Failed to serialize config")?;

        fs::write(path, content)
            .with_context(|| format!("Failed to write config to: {}", path.display()))?;

        Ok(())
    }

    pub fn create_template() -> Self {
        let mut scripts = HashMap::new();
        scripts.insert("clean".to_string(), Value::String("...".to_string()));
        scripts.insert(
            "deploy staging".to_string(),
            Value::String("echo 'ruby deploy tool --staging'".to_string()),
        );
        scripts.insert(
            "deploy production".to_string(),
            Value::String("...".to_string()),
        );
        scripts.insert("release beta".to_string(), Value::String("...".to_string()));
        scripts.insert(
            "release alpha".to_string(),
            Value::String("...".to_string()),
        );
        scripts.insert("release dev".to_string(), Value::String("...".to_string()));

        Self {
            include: Vec::new(),
            scripts,
        }
    }
}
