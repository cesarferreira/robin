use anyhow::{Context, Result, anyhow};
use clap::Parser;
use colored::*;
use dialoguer::Confirm;
use std::path::PathBuf;

use robin::{
    CONFIG_FILE, Cli, Commands, RobinConfig, check_environment, check_for_update, find_config_path,
    command_lines, interactive_mode, list_commands, replace_variables, resolve_task_command,
    run_script, script_command, send_notification, split_command_and_args, update_tools,
};

const GITHUB_TEMPLATE_BASE: &str =
    "https://raw.githubusercontent.com/cesarferreira/robin/refs/heads/main/templates";

async fn fetch_template(template_name: &str) -> Result<RobinConfig> {
    let url = format!("{}/{}.json", GITHUB_TEMPLATE_BASE, template_name);
    let response = reqwest::get(&url)
        .await
        .with_context(|| format!("Failed to fetch template from: {}", url))?;

    if !response.status().is_success() {
        return Err(anyhow!("Template '{}' not found", template_name));
    }

    let content = response
        .text()
        .await
        .with_context(|| "Failed to read template content")?;

    let config: RobinConfig =
        serde_json::from_str(&content).with_context(|| "Failed to parse template JSON")?;

    Ok(config)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Run the requested command first, then surface any available update.
    let outcome = dispatch(&cli).await;
    check_for_update().await;
    outcome
}

async fn dispatch(cli: &Cli) -> Result<()> {
    // Read/edit commands locate the nearest `.robin.json` by walking up from the
    // current directory; `init` always targets the current directory so it never
    // overwrites a parent project's config.
    let config_path = find_config_path();

    match &cli.command {
        Some(Commands::Init { template }) => {
            let config_path = PathBuf::from(CONFIG_FILE);
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

        Some(Commands::Add { name, script }) => {
            // Edit only the local file — never inline included scripts.
            let mut config = if config_path.exists() {
                RobinConfig::load_raw(&config_path)?
            } else {
                RobinConfig::create_template()
            };

            config
                .scripts
                .insert(name.clone(), serde_json::Value::String(script.clone()));
            config.save(&config_path)?;
            println!("{} {}", "Added command:".green(), name);
        }

        Some(Commands::Remove { name }) => {
            let mut config = RobinConfig::load_raw(&config_path)
                .with_context(|| "No .robin.json found. Run 'robin init' first")?;

            if config.scripts.remove(name).is_some() {
                config.save(&config_path)?;
                println!("{} {}", "Removed command:".green(), name);
            } else {
                return Err(anyhow!("Unknown command: {}", name));
            }
        }

        Some(Commands::Rename { from, to }) => {
            let mut config = RobinConfig::load_raw(&config_path)
                .with_context(|| "No .robin.json found. Run 'robin init' first")?;

            config.rename_script(from, to)?;
            config.save(&config_path)?;
            println!("{} {} {} {}", "Renamed command:".green(), from, "→".dimmed(), to);
        }

        Some(Commands::Migrate) => {
            let config = RobinConfig::load_raw(&config_path)
                .with_context(|| "No .robin.json found. Run 'robin init' first")?;
            let migrated = config.migrated();
            migrated.save(&config_path)?;
            println!(
                "{} {} (added a 'desc' field to each task)",
                "Migrated".green(),
                config_path.display()
            );
        }

        Some(Commands::Doctor) => {
            let config = RobinConfig::load(&config_path)
                .with_context(|| "No .robin.json found. Run 'robin init' first")?;
            let (success, found, missing, duration) = check_environment(&config)?;

            if cli.notify {
                let message = if success {
                    format!("All {} tools found ({:.1}s)", found, duration.as_secs_f32())
                } else {
                    format!(
                        "{} tools found, {} missing ({:.1}s)",
                        found,
                        missing,
                        duration.as_secs_f32()
                    )
                };
                send_notification("Robin Doctor", &message, success)?;
            }
        }

        Some(Commands::DoctorUpdate) => {
            let config = RobinConfig::load(&config_path)
                .with_context(|| "No .robin.json found. Run 'robin init' first")?;
            let start_time = std::time::Instant::now();
            let (success, _updated_tools) = update_tools(&config)?;

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

            // Robin's own flags are also accepted after the task name (e.g.
            // `robin build --dry-run`), since the external subcommand captures
            // everything trailing the command into these args.
            let dry_run = cli.dry_run || var_args.iter().any(|a| a == "--dry-run");
            let notify = cli.notify || var_args.iter().any(|a| a == "--notify");
            let var_args: Vec<String> = var_args
                .into_iter()
                .filter(|a| a != "--dry-run" && a != "--notify")
                .collect();

            if let Some(entry) = config.scripts.get(&script_name) {
                let script = script_command(entry).ok_or_else(|| {
                    anyhow!("Command '{}' has an invalid script definition", script_name)
                })?;
                let resolved = resolve_task_command(script, &config.scripts)?;
                let script_with_vars = replace_variables(&resolved, &var_args)?;

                if dry_run {
                    println!("{}", format!("Would run '{}':", script_name).dimmed());
                    for line in command_lines(&script_with_vars) {
                        println!("  {}", line);
                    }
                } else {
                    run_script(&script_with_vars, notify)?;
                }
            } else {
                println!("{} {}", "Unknown command:".red(), script_name);
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
