mod cli;
mod config;

use std::path::PathBuf;
use std::process::Command;
use anyhow::{Context, Result, anyhow};
use clap::Parser;
use colored::*;
use inquire::Select;
use regex::Regex;

use cli::{Cli, Commands};
use config::RobinConfig;

const CONFIG_FILE: &str = ".robin.json";

fn replace_variables(script: &str, args: &[String]) -> Result<String> {
    // Updated regex to support both patterns:
    // 1. {{variable=default}}
    // 2. {{variable=[value1, value2, ...]}}
    let var_regex = Regex::new(r"\{\{(\w+)(?:=([^}\]]+|\[[^\]]+\]))\}\}").unwrap();
    let mut result = script.to_string();
    
    for capture in var_regex.captures_iter(script) {
        let full_match = &capture[0];
        let var_name = &capture[1];
        let default_or_enum = capture.get(2).map(|m| m.as_str()).unwrap_or("");
        let var_pattern = format!("--{}=", var_name);

        // Check if this is an enum validation pattern
        if default_or_enum.starts_with('[') && default_or_enum.ends_with(']') {
            let allowed_values: Vec<&str> = default_or_enum[1..default_or_enum.len()-1]
                .split(',')
                .map(|s| s.trim())
                .collect();
            
            // Find the matching argument and validate against allowed values
            let value = args.iter()
                .find(|arg| arg.starts_with(&var_pattern))
                .map(|arg| arg.trim_start_matches(&var_pattern))
                .ok_or_else(|| anyhow!("Missing required variable: {}", var_name))?;

            if !allowed_values.contains(&value) {
                return Err(anyhow!("Value '{}' for {} must be one of: {}", 
                    value, var_name, allowed_values.join(", ")));
            }
            
            result = result.replace(full_match, value);
        } else {
            // Handle regular variable substitution with optional default
            let value = args.iter()
                .find(|arg| arg.starts_with(&var_pattern))
                .map(|arg| arg.trim_start_matches(&var_pattern))
                .or(Some(default_or_enum))
                .ok_or_else(|| anyhow!("Missing required variable: {}", var_name))?;

            result = result.replace(full_match, value);
        }
    }
    
    Ok(result)
}

fn split_command_and_args(args: &[String]) -> (String, Vec<String>) {
    if args.is_empty() {
        return (String::new(), vec![]);
    }

    let mut command_parts = Vec::new();
    let mut var_args = Vec::new();
    let mut found_args = false;

    for arg in args {
        if arg.starts_with("--") {
            found_args = true;
            var_args.push(arg.clone());
        } else if !found_args {
            command_parts.push(arg.clone());
        } else {
            var_args.push(arg.clone());
        }
    }

    (command_parts.join(" "), var_args)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_path = PathBuf::from(CONFIG_FILE);

    match &cli.command {
        Some(Commands::Init { template }) => {
            if config_path.exists() {
                println!("{}", "Config file already exists!".red());
                return Ok(());
            }

            let config = RobinConfig::create_template();
            config.save(&config_path)?;
            println!("{}", "Created .robin.json template".green());
        }

        Some(Commands::Add { name, script }) => {
            let mut config = if config_path.exists() {
                RobinConfig::load(&config_path)?
            } else {
                RobinConfig::create_template()
            };

            config.scripts.insert(name.clone(), script.clone());
            config.save(&config_path)?;
            println!("{} {}", "Added command:".green(), name);
        }

        Some(Commands::Run(args)) => {
            let config = RobinConfig::load(&config_path)
                .with_context(|| "No .robin.json found. Run 'robin init' first")?;

            let (script_name, var_args) = split_command_and_args(args);

            if let Some(script) = config.scripts.get(&script_name) {
                let script_with_vars = replace_variables(script, &var_args)?;
                run_script(&script_with_vars)?;
            } else {
                println!("{} {}", "Unknown command:".red(), script_name);
            }
        }

        None => {
            if cli.list {
                list_commands(&config_path)?;
            } else if cli.interactive {
                interactive_mode(&config_path)?;
            } else {
                println!("Use --help to see available commands");
            }
        }
    }

    Ok(())
}

fn run_script(script: &str) -> Result<()> {
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", script])
            .status()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(script)
            .status()
    }.with_context(|| format!("Failed to execute script: {}", script))?;

    if !status.success() {
        println!("{}", "Script failed!".red());
    }

    Ok(())
}

fn list_commands(config_path: &PathBuf) -> Result<()> {
    let config = RobinConfig::load(config_path)
        .with_context(|| "No .robin.json found. Run 'robin init' first")?;

    for (name, script) in &config.scripts {
        println!("==> {} # {}", name.blue(), script);
    }

    Ok(())
}

fn interactive_mode(config_path: &PathBuf) -> Result<()> {
    let config = RobinConfig::load(config_path)
        .with_context(|| "No .robin.json found. Run 'robin init' first")?;

    let commands: Vec<String> = config.scripts.keys().cloned().collect();
    if commands.is_empty() {
        println!("{}", "No commands available".red());
        return Ok(());
    }

    let selection = Select::new("Select a command to run:", commands).prompt()?;
    if let Some(script) = config.scripts.get(&selection) {
        run_script(script)?;
    }

    Ok(())
}
