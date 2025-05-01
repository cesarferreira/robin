use std::path::PathBuf;
use anyhow::{Context, Result, anyhow};
use clap::Parser;
use colored::*;
use serde_json;
use dialoguer::Confirm;
use reqwest;
use std::fs;

use robin::{
    Cli, Commands,
    RobinConfig,
    check_environment, update_tools,
    send_notification, split_command_and_args, replace_variables,
    run_script, list_commands, interactive_mode,
    CONFIG_FILE
};

const GITHUB_TEMPLATE_BASE: &str = "https://raw.githubusercontent.com/cesarferreira/robin/refs/heads/main/templates";

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
                        return Err(e);
                    }
                }
            } else {
                RobinConfig::create_template()
            };

            config.save(&config_path)?;
            println!("{} {}", "Created".green(), config_path.display());
        }

        Some(Commands::Add { name, script, description }) => {
            let mut config = if config_path.exists() {
                RobinConfig::load(&config_path)?
            } else {
                RobinConfig::create_template()
            };

            let desc = description.clone().unwrap_or_else(|| "No description provided".to_string());
            config.tasks.push(serde_json::from_value(serde_json::json!({
                "name": name,
                "command": script,
                "description": desc
            })).unwrap());
            
            config.save(&config_path)?;
            println!("{} {}", "Added task:".green(), name);
        }

        Some(Commands::Migrate { input, output, force }) => {
            let input_path = PathBuf::from(input);
            let output_path = PathBuf::from(output);
            let same_file = input_path == output_path;
            
            if !input_path.exists() {
                println!("{} {}", "Error:".red(), "Input file does not exist");
                return Err(anyhow!("Input file does not exist: {}", input_path.display()));
            }
            
            // Read the input file first
            let content = fs::read_to_string(&input_path)?;
            
            // Check if output file exists and if we should override
            if output_path.exists() && !force {
                let should_override = Confirm::new()
                    .with_prompt(format!("{} already exists. Do you want to override it?", 
                        if same_file { "Config file" } else { "Output file" }))
                    .default(false)
                    .interact()?;
                
                if !should_override {
                    println!("{}", "Migration cancelled.".yellow());
                    return Ok(());
                }
            }
            
            println!("Migrating to new format...");
            let new_config = RobinConfig::migrate_from_v1(&input_path)?;
            
            // If same file, create a backup
            if same_file {
                let backup_path = format!("{}.bak", input_path.display());
                println!("Creating backup at {}", backup_path);
                fs::write(&backup_path, content)?;
            }
            
            new_config.save(&output_path)?;
            println!("{} {}", "Migration complete.".green(), 
                if same_file { 
                    "Config file updated with new format".to_string() 
                } else {
                    format!("Config saved to {}", output_path.display())
                });
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
            let config = RobinConfig::load(&config_path)?;

            let (task_name, var_args) = split_command_and_args(args);

            let task = config.tasks.iter()
                .find(|t| t.name == task_name);
                
            if let Some(task) = task {
                let command_with_vars = replace_variables(&task.command, &var_args)?;
                run_script(&command_with_vars, cli.notify)?;
            } else {
                println!("{} {}", "Unknown task:".red(), task_name);
            }
        }

        None => {
            if cli.list {
                list_commands(&config_path)?;
            } else {
                interactive_mode(&config_path)?;
            }
        }
    }

    Ok(())
}
