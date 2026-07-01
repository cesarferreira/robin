use anyhow::{Context, Result, anyhow};
use colored::*;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use inquire::Select;
use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::process::Command;

use crate::config::{RobinConfig, script_command, script_description};
use crate::utils::send_notification;

/// Expands a task's command, resolving any `@task` references into the commands
/// of the referenced task (recursively), and returns a flattened command ready
/// to run. Reference cycles are detected and reported as errors.
///
/// A bare string command with no reference is returned unchanged so simple
/// tasks keep their single-command behaviour (including notifications). Anything
/// that contains a reference, or is an array, flattens to an array of shell
/// commands. Variable substitution is applied later, by the caller.
pub fn resolve_task_command(cmd: &Value, scripts: &HashMap<String, Value>) -> Result<Value> {
    // Fast path: a plain command with no reference needs no expansion.
    if let Value::String(s) = cmd {
        if !s.trim_start().starts_with('@') {
            return Ok(cmd.clone());
        }
    }

    let mut out = Vec::new();
    let mut stack = Vec::new();
    resolve_into(cmd, scripts, &mut stack, &mut out)?;
    Ok(Value::Array(out.into_iter().map(Value::String).collect()))
}

/// Flattens a resolved command into the individual shell command lines it will
/// run: a single string yields one line, an array yields one line per element.
/// Used by `--dry-run` to preview exactly what would execute.
pub fn command_lines(script: &Value) -> Vec<String> {
    match script {
        Value::String(s) => vec![s.clone()],
        Value::Array(items) => items
            .iter()
            .filter_map(|c| c.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

fn resolve_into(
    cmd: &Value,
    scripts: &HashMap<String, Value>,
    stack: &mut Vec<String>,
    out: &mut Vec<String>,
) -> Result<()> {
    match cmd {
        Value::String(s) => resolve_command_str(s, scripts, stack, out),
        Value::Array(items) => {
            for item in items {
                if let Some(s) = item.as_str() {
                    resolve_command_str(s, scripts, stack, out)?;
                }
            }
            Ok(())
        }
        _ => Err(anyhow!(
            "Invalid script type: must be string or array of strings"
        )),
    }
}

fn resolve_command_str(
    s: &str,
    scripts: &HashMap<String, Value>,
    stack: &mut Vec<String>,
    out: &mut Vec<String>,
) -> Result<()> {
    match s.trim_start().strip_prefix('@') {
        Some(reference) => {
            let name = reference.trim();
            if stack.iter().any(|n| n == name) {
                stack.push(name.to_string());
                return Err(anyhow!(
                    "Cycle detected in task references: {}",
                    stack.join(" -> ")
                ));
            }
            let entry = scripts
                .get(name)
                .ok_or_else(|| anyhow!("Referenced task '{}' not found", name))?;
            let referenced = script_command(entry).ok_or_else(|| {
                anyhow!("Referenced task '{}' has an invalid script definition", name)
            })?;

            stack.push(name.to_string());
            resolve_into(referenced, scripts, stack, out)?;
            stack.pop();
            Ok(())
        }
        None => {
            out.push(s.to_string());
            Ok(())
        }
    }
}

/// Builds the shell command used to run a single script line, optionally in a
/// specific working directory.
fn shell_command(cmd: &str, cwd: Option<&Path>) -> Command {
    let mut command = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.args(["/C", cmd]);
        c
    } else {
        let mut c = Command::new("sh");
        c.arg("-c").arg(cmd);
        c
    };
    if let Some(dir) = cwd {
        command.current_dir(dir);
    }
    command
}

pub fn run_script(script: &serde_json::Value, notify: bool) -> Result<()> {
    run_script_in(script, notify, None)
}

/// Runs a script, executing each command in `cwd` when provided (otherwise in
/// the process's current directory).
pub fn run_script_in(script: &serde_json::Value, notify: bool, cwd: Option<&Path>) -> Result<()> {
    let start_time = std::time::Instant::now();

    match script {
        serde_json::Value::String(cmd) => {
            let status = shell_command(cmd, cwd)
                .status()
                .with_context(|| format!("Failed to execute script: {}", cmd))?;

            if notify {
                let duration = start_time.elapsed();
                let success = status.success();
                let message = if success {
                    format!("Completed in {:.1}s", duration.as_secs_f32())
                } else {
                    "Failed".to_string()
                };

                send_notification(
                    "Robin",
                    &format!(
                        "Command '{}' {}",
                        cmd.split_whitespace().next().unwrap_or(cmd),
                        message
                    ),
                    success,
                )?;
            }

            if !status.success() {
                println!("{}", "Script failed!".red());
                return Err(anyhow!("Script failed: {}", cmd));
            }
            Ok(())
        }
        serde_json::Value::Array(commands) => {
            for cmd in commands {
                if let Some(cmd_str) = cmd.as_str() {
                    // Echo each step so the user can follow a multi-command
                    // sequence and see exactly which command is running.
                    println!("{} {}", "▶".cyan().bold(), cmd_str);

                    let status = shell_command(cmd_str, cwd)
                        .status()
                        .with_context(|| format!("Failed to execute script: {}", cmd_str))?;

                    if !status.success() {
                        println!("{}", format!("Script failed: {}", cmd_str).red());
                        return Err(anyhow!("Script failed: {}", cmd_str));
                    }
                }
            }

            if notify {
                let duration = start_time.elapsed();
                send_notification(
                    "Robin",
                    &format!(
                        "Command sequence completed in {:.1}s",
                        duration.as_secs_f32()
                    ),
                    true,
                )?;
            }
            Ok(())
        }
        _ => Err(anyhow!(
            "Invalid script type: must be string or array of strings"
        )),
    }
}

pub fn list_commands(config_path: &Path) -> Result<()> {
    let config = RobinConfig::load(config_path)
        .with_context(|| "No .robin.json found. Run 'robin init' first")?;

    // Find the longest command name for padding
    let max_len = config
        .scripts
        .keys()
        .map(|name| name.len())
        .max()
        .unwrap_or(0);

    // Convert to sorted vec for alphabetical ordering
    let mut commands: Vec<_> = config.scripts.iter().collect();
    commands.sort_by(|a, b| a.0.cmp(b.0));

    for (name, script) in commands {
        if let Some(desc) = script_description(script) {
            println!("    {:<width$}   {}", "", desc.dimmed(), width = max_len);
        }
        match script_command(script) {
            Some(serde_json::Value::String(cmd)) => {
                println!("==> {:<width$} # {}", name.blue(), cmd, width = max_len);
            }
            Some(serde_json::Value::Array(commands)) => {
                println!("==> {:<width$} # [", name.blue(), width = max_len);
                for cmd in commands {
                    if let Some(cmd_str) = cmd.as_str() {
                        println!("       {}", cmd_str);
                    }
                }
                println!("     ]");
            }
            _ => println!(
                "==> {:<width$} # <invalid script type>",
                name.blue(),
                width = max_len
            ),
        }
    }

    Ok(())
}

/// One selectable row in the interactive picker.
struct CommandChoice {
    /// The task name, used to look the script back up after selection.
    name: String,
    /// Plain-text `name description` used for fuzzy matching (never colored,
    /// so ANSI codes in `label` don't pollute the search).
    search: String,
    /// Pre-rendered, column-aligned label shown in the picker.
    label: String,
}

impl fmt::Display for CommandChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label)
    }
}

/// Builds the aligned, sorted list of picker rows from the config's scripts.
/// Names are sorted alphabetically (HashMap order isn't stable) and padded to a
/// common width so descriptions line up in one column.
fn command_choices(scripts: &HashMap<String, Value>) -> Vec<CommandChoice> {
    let mut entries: Vec<(&String, &Value)> = scripts.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));

    let max_len = entries.iter().map(|(name, _)| name.len()).max().unwrap_or(0);

    entries
        .into_iter()
        .map(|(name, script)| match script_description(script) {
            Some(desc) => CommandChoice {
                name: name.clone(),
                search: format!("{} {}", name, desc),
                label: format!("{:<width$}   {}", name, desc.dimmed(), width = max_len),
            },
            None => CommandChoice {
                name: name.clone(),
                search: name.clone(),
                label: name.clone(),
            },
        })
        .collect()
}

pub fn interactive_mode(config_path: &Path) -> Result<()> {
    let config = RobinConfig::load(config_path)
        .with_context(|| "No .robin.json found. Run 'robin init' first")?;

    if config.scripts.is_empty() {
        println!("{}", "No commands available".red());
        return Ok(());
    }

    let choices = command_choices(&config.scripts);

    // Fuzzy-match against the plain `search` text (name + description) rather
    // than the colored label inquire would use by default.
    let matcher = SkimMatcherV2::default();
    let scorer = |input: &str, choice: &CommandChoice, _: &str, _: usize| -> Option<i64> {
        if input.is_empty() {
            return Some(0);
        }
        matcher.fuzzy_match(&choice.search, input)
    };

    let selection = Select::new("Select a command to run:", choices)
        .with_scorer(&scorer)
        .prompt()?;

    if let Some(script) = config.scripts.get(&selection.name) {
        if let Some(cmd) = script_command(script) {
            let resolved = resolve_task_command(cmd, &config.scripts)?;
            run_script(&resolved, false)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn strip_ansi(s: &str) -> String {
        let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
        re.replace_all(s, "").to_string()
    }

    #[test]
    fn choices_are_sorted_and_description_column_is_aligned() {
        let mut scripts = HashMap::new();
        scripts.insert("build".to_string(), json!({ "cmd": "cargo build", "desc": "Compile" }));
        scripts.insert("deploy".to_string(), json!({ "cmd": "ship", "desc": "Release it" }));
        scripts.insert("clean".to_string(), json!("rm -rf target/"));

        let choices = command_choices(&scripts);

        // Alphabetical order, independent of HashMap iteration order.
        assert_eq!(
            choices.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
            vec!["build", "clean", "deploy"]
        );

        // The description column starts at the same offset for every row that
        // has a description (name padded to the longest name, "deploy" = 6).
        let build = strip_ansi(&choices[0].label);
        let deploy = strip_ansi(&choices[2].label);
        assert_eq!(build.find("Compile"), deploy.find("Release it"));
        assert_eq!(build, "build    Compile");
        assert_eq!(deploy, "deploy   Release it");

        // A task without a description renders as just its (unpadded) name.
        assert_eq!(choices[1].label, "clean");
    }

    #[test]
    fn search_text_is_plain_name_plus_description() {
        let mut scripts = HashMap::new();
        scripts.insert("build".to_string(), json!({ "cmd": "x", "desc": "Compile the app" }));
        scripts.insert("clean".to_string(), json!("rm -rf target/"));

        let choices = command_choices(&scripts);

        // Description is searchable, and the search text carries no ANSI codes.
        let build = choices.iter().find(|c| c.name == "build").unwrap();
        assert_eq!(build.search, "build Compile the app");
        assert!(!build.search.contains('\u{1b}'));

        // No description -> search is just the name.
        let clean = choices.iter().find(|c| c.name == "clean").unwrap();
        assert_eq!(clean.search, "clean");
    }
}
