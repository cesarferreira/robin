mod cli;
mod config;
mod tools;
mod utils;
mod scripts;

use std::path::PathBuf;
use std::process::Command;
use anyhow::{Context, Result, anyhow};
use clap::Parser;
use colored::*;
use regex::Regex;
use serde_json;

use cli::{Cli, Commands};
use config::RobinConfig;
use tools::{check_environment, update_tools};
use utils::{send_notification, split_command_and_args, replace_variables};
use scripts::{run_script, list_commands, interactive_mode};

const CONFIG_FILE: &str = ".robin.json";


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

            config.scripts.insert(name.clone(), serde_json::Value::String(script.clone()));
            config.save(&config_path)?;
            println!("{} {}", "Added command:".green(), name);
        }

        Some(Commands::Doctor) => {
            let start_time = std::time::Instant::now();
            let (success, found, missing, duration) = check_environment()?;
            
            if cli.notify {
                let message = if success {
                    format!("All {} tools found ({:.1}s)", found, duration.as_secs_f32())
                } else {
                    format!("{} tools found, {} missing ({:.1}s)", found, missing, duration.as_secs_f32())
                };
                send_notification("Robin Doctor", &message, success)?;
            }
        }

        Some(Commands::DoctorUpdate) => {
            let start_time = std::time::Instant::now();
            let (success, updated_tools) = update_tools()?;
            
            if cli.notify {
                let duration = start_time.elapsed();
                let message = if success {
                    format!("Tools updated in {:.1}s", duration.as_secs_f32())
                } else {
                    "Update failed".to_string()
                };
                send_notification("Robin Doctor Update", &message, success)?;
            }
        }

        Some(Commands::Run(args)) => {
            let config = RobinConfig::load(&config_path)
                .with_context(|| "No .robin.json found. Run 'robin init' first")?;

            let (script_name, var_args) = split_command_and_args(args);

            if let Some(script) = config.scripts.get(&script_name) {
                let script_with_vars = replace_variables(script, &var_args)?;
                run_script(&script_with_vars, cli.notify)?;
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
