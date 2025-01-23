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
            }
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
                        return Ok(());  // Stop execution if any command fails
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
        },
        _ => return Err(anyhow!("Invalid script type: must be string or array of strings")),
    }

    Ok(())
}

pub fn list_commands(config_path: &PathBuf) -> Result<()> {
    let config = RobinConfig::load(config_path)
        .with_context(|| "No .robin.json found. Run 'robin init' first")?;

    for (name, script) in &config.scripts {
        match script {
            serde_json::Value::String(cmd) => {
                println!("==> {} # {}", name.blue(), cmd);
            },
            serde_json::Value::Array(commands) => {
                println!("==> {} # [", name.blue());
                for cmd in commands {
                    if let Some(cmd_str) = cmd.as_str() {
                        println!("       {}", cmd_str);
                    }
                }
                println!("     ]");
            },
            _ => println!("==> {} # <invalid script type>", name.blue()),
        }
    }

    Ok(())
}

pub fn interactive_mode(config_path: &PathBuf) -> Result<()> {
    let config = RobinConfig::load(config_path)
        .with_context(|| "No .robin.json found. Run 'robin init' first")?;

    let commands: Vec<String> = config.scripts.keys().cloned().collect();
    if commands.is_empty() {
        println!("{}", "No commands available".red());
        return Ok(());
    }

    let selection = Select::new("Select a command to run:", commands).prompt()?;
    if let Some(script) = config.scripts.get(&selection) {
        run_script(script, false)?;
    }

    Ok(())
} 