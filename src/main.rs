use std::path::PathBuf;
use anyhow::{Context, Result, anyhow};
use clap::Parser;
use colored::*;
use serde_json;
use dialoguer::Confirm;
use reqwest;

use robin::{
    Cli, Commands,
    RobinConfig,
    check_environment, update_tools,
    send_notification, split_command_and_args, replace_variables,
    run_script, list_commands, interactive_mode,
    CONFIG_FILE
};

const GITHUB_TEMPLATE_BASE: &str = "https://raw.githubusercontent.com/cesarferreira/robin/refs/heads/master/templates";

async fn fetch_template(template_name: &str) -> Result<RobinConfig> {
    let url = format!("{}/{}.json", GITHUB_TEMPLATE_BASE, template_name);
    let response = reqwest::get(&url)
        .await
        .with_context(|| format!("Failed to fetch template from: {}", url))?;
    
    if !response.status().is_success() {
        return Err(anyhow!("Template '{}' not found", template_name));
    }

    let content = response.text()
        .await
        .with_context(|| "Failed to read template content")?;
    
    let config: RobinConfig = serde_json::from_str(&content)
        .with_context(|| "Failed to parse template JSON")?;
    
    Ok(config)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_path = PathBuf::from(CONFIG_FILE);

    match &cli.command {
        Some(Commands::Init { template }) => {
            if config_path.exists() {
                let should_override = Confirm::new()
                    .with_prompt("Config file already exists. Do you want to override it?")
                    .default(false)
                    .interact()?;
                
                if !should_override {
                    println!("{}", "Operation cancelled.".yellow());
                    return Ok(());
                }
            }

            let config = if let Some(template_name) = template {
                println!("Fetching template '{}'...", template_name);
                match fetch_template(template_name).await {
                    Ok(config) => config,
                    Err(e) => {
                        println!("{} {}", "Error:".red(), e);
                        println!("Available templates: android, ios, flutter, rails, node, python, rust, go");
                        return Err(e);
                    }
                }
            } else {
                RobinConfig::create_template()
            };

            config.save(&config_path)?;
            println!("{} {}", "Created".green(), config_path.display());
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
