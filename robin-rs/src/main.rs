mod cli;
mod config;

use std::path::PathBuf;
use std::process::Command;
use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use inquire::Select;

use cli::{Cli, Commands};
use config::RobinConfig;

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

            config.scripts.insert(name.clone(), script.clone());
            config.save(&config_path)?;
            println!("{} {}", "Added command:".green(), name);
        }

        Some(Commands::Run(args)) => {
            let config = RobinConfig::load(&config_path)
                .with_context(|| "No .robin.json found. Run 'robin init' first")?;

            let script_name = args.join(" ");
            if let Some(script) = config.scripts.get(&script_name) {
                run_script(script)?;
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
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", script])
            .output()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(script)
            .output()
    }.with_context(|| format!("Failed to execute script: {}", script))?;

    if !output.status.success() {
        println!("{}", "Script failed!".red());
        return Ok(());
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
