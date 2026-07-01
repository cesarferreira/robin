use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::CONFIG_FILE;

/// Canonical location of the published JSON Schema for `.robin.json`. Generated
/// configs point their `$schema` here so editors can offer autocomplete and
/// validation.
pub const SCHEMA_URL: &str =
    "https://raw.githubusercontent.com/cesarferreira/robin/refs/heads/main/schema/robin.schema.json";

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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RobinConfig {
    /// Optional pointer to the JSON Schema, preserved across edits so editor
    /// tooling keeps working. Serialized as the conventional `$schema` key.
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include: Vec<String>,
    pub scripts: HashMap<String, Value>,
}

/// Returns the executable part of a script entry.
///
/// A script may be written in three shapes:
///   - a bare string:          `"cargo build"`
///   - an array of commands:    `["cargo build", "cargo test"]`
///   - an object with metadata: `{ "cmd": <string|array>, "desc": "..." }`
///
/// For the string/array forms the entry is itself the command; for the object
/// form the command lives under `cmd`. Returns `None` for unsupported shapes.
pub fn script_command(entry: &Value) -> Option<&Value> {
    match entry {
        Value::String(_) | Value::Array(_) => Some(entry),
        Value::Object(map) => map.get("cmd"),
        _ => None,
    }
}

/// Returns the human-readable description of a script entry, when it uses the
/// object form and carries a non-empty `desc`.
pub fn script_description(entry: &Value) -> Option<&str> {
    entry
        .as_object()?
        .get("desc")?
        .as_str()
        .filter(|s| !s.is_empty())
}

impl RobinConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let mut config = Self::load_raw(path)?;

        // Load and merge included configs
        if !config.include.is_empty() {
            let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
            config = config.merge_includes(base_dir)?;
        }

        Ok(config)
    }

    /// Parses a single config file without following `include` — the scripts are
    /// exactly those declared in this file. Used by commands (like `migrate`)
    /// that rewrite the file in place and must not inline included scripts.
    pub fn load_raw(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "No .robin.json found. Run 'robin init' first"
            ));
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Self = serde_json::from_str(&content)
            .with_context(|| "The .robin.json file exists but contains malformed JSON. Please check the file format.")?;

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
            schema: self.schema.clone(),
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
            schema: Some(SCHEMA_URL.to_string()),
            include: Vec::new(),
            scripts,
        }
    }

    /// Renames a task from `from` to `to`, preserving its definition. Errors if
    /// `from` does not exist or `to` is already taken, so no task is silently
    /// overwritten.
    pub fn rename_script(&mut self, from: &str, to: &str) -> Result<()> {
        if self.scripts.contains_key(to) {
            return Err(anyhow::anyhow!("A command named '{}' already exists", to));
        }
        let value = self
            .scripts
            .remove(from)
            .ok_or_else(|| anyhow::anyhow!("Unknown command: {}", from))?;
        self.scripts.insert(to.to_string(), value);
        Ok(())
    }

    /// Rewrites every script into the object form `{ "cmd": ..., "desc": "" }`,
    /// leaving an empty `desc` ready to be filled in. Entries already in object
    /// form are kept as-is. This is what `robin migrate` applies so users can
    /// start attaching descriptions to existing tasks.
    pub fn migrated(&self) -> Self {
        let scripts = self
            .scripts
            .iter()
            .map(|(name, entry)| {
                let migrated = match entry {
                    Value::Object(_) => entry.clone(),
                    other => serde_json::json!({ "cmd": other, "desc": "" }),
                };
                (name.clone(), migrated)
            })
            .collect();

        Self {
            // Ensure the migrated file points at the schema for editor tooling,
            // keeping any pointer the user already set.
            schema: self
                .schema
                .clone()
                .or_else(|| Some(SCHEMA_URL.to_string())),
            include: self.include.clone(),
            scripts,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn script_command_reads_bare_string_and_array() {
        assert_eq!(script_command(&json!("cargo build")), Some(&json!("cargo build")));
        let arr = json!(["a", "b"]);
        assert_eq!(script_command(&arr), Some(&arr));
    }

    #[test]
    fn script_command_reads_cmd_field_from_object() {
        let entry = json!({ "cmd": "cargo build", "desc": "Build it" });
        assert_eq!(script_command(&entry), Some(&json!("cargo build")));

        let entry_arr = json!({ "cmd": ["a", "b"], "desc": "seq" });
        assert_eq!(script_command(&entry_arr), Some(&json!(["a", "b"])));
    }

    #[test]
    fn script_command_is_none_for_unsupported_shapes() {
        assert_eq!(script_command(&json!(42)), None);
        assert_eq!(script_command(&json!({ "desc": "no cmd" })), None);
    }

    #[test]
    fn script_description_reads_only_non_empty_desc() {
        assert_eq!(
            script_description(&json!({ "cmd": "x", "desc": "hello" })),
            Some("hello")
        );
        assert_eq!(script_description(&json!({ "cmd": "x", "desc": "" })), None);
        assert_eq!(script_description(&json!("cargo build")), None);
    }

    #[test]
    fn rename_script_moves_definition_to_new_key() {
        let mut scripts = HashMap::new();
        scripts.insert("old".to_string(), json!("cargo build"));
        let mut config = RobinConfig {
            schema: None,
            include: vec![],
            scripts,
        };

        config.rename_script("old", "new").unwrap();

        assert!(!config.scripts.contains_key("old"));
        assert_eq!(config.scripts["new"], json!("cargo build"));
    }

    #[test]
    fn rename_script_errors_on_unknown_source() {
        let mut config = RobinConfig {
            schema: None,
            include: vec![],
            scripts: HashMap::new(),
        };
        let err = config.rename_script("missing", "new").unwrap_err();
        assert!(err.to_string().contains("Unknown command"), "{err}");
    }

    #[test]
    fn rename_script_refuses_to_overwrite_existing_target() {
        let mut scripts = HashMap::new();
        scripts.insert("a".to_string(), json!("1"));
        scripts.insert("b".to_string(), json!("2"));
        let mut config = RobinConfig {
            schema: None,
            include: vec![],
            scripts,
        };

        let err = config.rename_script("a", "b").unwrap_err();
        assert!(err.to_string().contains("already exists"), "{err}");
        // Both tasks must remain untouched after a refused rename.
        assert_eq!(config.scripts["a"], json!("1"));
        assert_eq!(config.scripts["b"], json!("2"));
    }

    #[test]
    fn migrated_wraps_strings_and_arrays_but_keeps_objects() {
        let mut scripts = HashMap::new();
        scripts.insert("s".to_string(), json!("cargo build"));
        scripts.insert("a".to_string(), json!(["x", "y"]));
        scripts.insert(
            "o".to_string(),
            json!({ "cmd": "already", "desc": "kept" }),
        );
        let config = RobinConfig {
            schema: None,
            include: vec!["base.json".to_string()],
            scripts,
        };

        let migrated = config.migrated();

        assert_eq!(migrated.scripts["s"], json!({ "cmd": "cargo build", "desc": "" }));
        assert_eq!(migrated.scripts["a"], json!({ "cmd": ["x", "y"], "desc": "" }));
        assert_eq!(migrated.scripts["o"], json!({ "cmd": "already", "desc": "kept" }));
        // `include` is preserved untouched.
        assert_eq!(migrated.include, vec!["base.json".to_string()]);
    }
}
