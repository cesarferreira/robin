use anyhow::{Context, Result, anyhow};
use colored::*;
use inquire::Select;
use serde_json;
use serde_json::Value;
use std::collections::HashMap;
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

pub fn interactive_mode(config_path: &Path) -> Result<()> {
    let config = RobinConfig::load(config_path)
        .with_context(|| "No .robin.json found. Run 'robin init' first")?;

    let commands: Vec<String> = config.scripts.keys().cloned().collect();
    if commands.is_empty() {
        println!("{}", "No commands available".red());
        return Ok(());
    }

    let selection = Select::new("Select a command to run:", commands).prompt()?;
    if let Some(script) = config.scripts.get(&selection) {
        if let Some(cmd) = script_command(script) {
            let resolved = resolve_task_command(cmd, &config.scripts)?;
            run_script(&resolved, false)?;
        }
    }

    Ok(())
}
