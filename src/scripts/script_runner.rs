use std::process::Command;
use std::path::PathBuf;
use anyhow::{Result, Context, anyhow};
use colored::*;
use inquire::Select;
use serde_json;

use crate::config::RobinConfig;
use crate::utils::send_notification;

pub fn run_script(script: &serde_json::Value, notify: bool) -> Result<()> {
    let start_time = std::time::Instant::now();
    
    match script {
        serde_json::Value::String(cmd) => {
            let status = if cfg!(target_os = "windows") {
                Command::new("cmd")
                    .args(["/C", cmd])
                    .status()
            } else {
                Command::new("sh")
                    .arg("-c")
                    .arg(cmd)
                    .status()
            }.with_context(|| format!("Failed to execute script: {}", cmd))?;

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
                    &format!("Command '{}' {}", cmd.split_whitespace().next().unwrap_or(cmd), message),
                    success,
                )?;
            }

            if !status.success() {
                println!("{}", "Script failed!".red());
                return Err(anyhow!("Script failed: {}", cmd));
            }
            Ok(())
        },
        serde_json::Value::Array(commands) => {
            for cmd in commands {
                if let Some(cmd_str) = cmd.as_str() {
                    let status = if cfg!(target_os = "windows") {
                        Command::new("cmd")
                            .args(["/C", cmd_str])
                            .status()
                    } else {
                        Command::new("sh")
                            .arg("-c")
                            .arg(cmd_str)
                            .status()
                    }.with_context(|| format!("Failed to execute script: {}", cmd_str))?;

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
                    &format!("Command sequence completed in {:.1}s", duration.as_secs_f32()),
                    true,
                )?;
            }
            Ok(())
        },
        _ => Err(anyhow!("Invalid script type: must be string or array of strings")),
    }
}

pub fn list_commands(config_path: &PathBuf) -> Result<()> {
    let config = RobinConfig::load(config_path)
        .with_context(|| "No config file found. Run 'robin init' first")?;

    // Find the longest command name for padding
    let max_len = config.tasks.iter()
        .map(|task| task.name.len())
        .max()
        .unwrap_or(0);

    // Sort tasks alphabetically by name
    let mut tasks = config.tasks.clone();
    tasks.sort_by(|a, b| a.name.cmp(&b.name));

    for task in tasks {
        println!("==> {:<width$} # {}", task.name.blue(), task.description, width = max_len);
    }

    Ok(())
}

pub fn interactive_mode(config_path: &PathBuf) -> Result<()> {
    let config = RobinConfig::load(config_path)
        .with_context(|| "No config file found. Run 'robin init' first")?;

    if config.tasks.is_empty() {
        println!("{}", "No tasks available".red());
        return Ok(());
    }

    let options: Vec<String> = config.tasks.iter()
        .map(|task| format!("{} - {}", task.name, task.description))
        .collect();

    let selection = Select::new("Select a task to run:", options).prompt()?;
    let task_name = selection.split(" - ").next().unwrap_or("");
    
    let task = config.tasks.iter()
        .find(|t| t.name == task_name)
        .ok_or_else(|| anyhow!("Task not found"))?;
    
    run_script(&task.command, false)?;

    Ok(())
} 