mod cli;
mod config;

use std::path::PathBuf;
use std::process::Command;
use anyhow::{Context, Result, anyhow};
use clap::Parser;
use colored::*;
use inquire::Select;
use regex::Regex;
use notify_rust::Notification;

use cli::{Cli, Commands};
use config::RobinConfig;

const CONFIG_FILE: &str = ".robin.json";

#[derive(Debug, PartialEq)]
struct RequiredTool {
    name: &'static str,
    command: &'static str,
    version_arg: &'static str,
    patterns: &'static [&'static str],
}

const KNOWN_TOOLS: &[RequiredTool] = &[
    RequiredTool {
        name: "Node.js",
        command: "node",
        version_arg: "--version",
        patterns: &["node ", "npm ", "npx "],
    },
    RequiredTool {
        name: "Python",
        command: "python",
        version_arg: "--version",
        patterns: &["python ", "pip ", "python3 "],
    },
    RequiredTool {
        name: "Ruby",
        command: "ruby",
        version_arg: "--version",
        patterns: &["ruby ", "gem ", "bundle "],
    },
    RequiredTool {
        name: "Fastlane",
        command: "fastlane",
        version_arg: "--version",
        patterns: &["fastlane "],
    },
    RequiredTool {
        name: "Flutter",
        command: "flutter",
        version_arg: "--version",
        patterns: &["flutter "],
    },
    RequiredTool {
        name: "Cargo",
        command: "cargo",
        version_arg: "--version",
        patterns: &["cargo "],
    },
    RequiredTool {
        name: "Go",
        command: "go",
        version_arg: "version",
        patterns: &["go "],
    },
    RequiredTool {
        name: "ADB",
        command: "adb",
        version_arg: "version",
        patterns: &["adb "],
    },
    RequiredTool {
        name: "Gradle",
        command: "gradle",
        version_arg: "--version",
        patterns: &["gradle ", "./gradlew "],
    },
    RequiredTool {
        name: "CocoaPods",
        command: "pod",
        version_arg: "--version",
        patterns: &["pod ", "cocoapods "],
    },
    RequiredTool {
        name: "Xcode CLI",
        command: "xcrun",
        version_arg: "--version",
        patterns: &["xcrun ", "xcodebuild "],
    },
    RequiredTool {
        name: "Docker",
        command: "docker",
        version_arg: "--version",
        patterns: &["docker "],
    },
    RequiredTool {
        name: "Git",
        command: "git",
        version_arg: "--version",
        patterns: &["git "],
    },
    RequiredTool {
        name: "Maven",
        command: "mvn",
        version_arg: "--version",
        patterns: &["mvn ", "maven "],
    },
];

fn detect_required_tools(config: &RobinConfig) -> Vec<&'static RequiredTool> {
    KNOWN_TOOLS
        .iter()
        .filter(|tool| {
            config.scripts.values().any(|script| {
                tool.patterns.iter().any(|&pattern| script.contains(pattern))
            })
        })
        .collect()
}

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

fn check_environment() -> Result<(bool, usize, usize, std::time::Duration)> {
    let start_time = std::time::Instant::now();
    let mut all_checks_passed = true;
    let mut found_tools = 0;
    let mut missing_tools = 0;

    let config_path = PathBuf::from(CONFIG_FILE);
    let config = RobinConfig::load(&config_path)
        .with_context(|| "No .robin.json found. Run 'robin init' first")?;

    println!("🔍 Checking development environment...\n");

    // Check Required Tools
    let required_tools = detect_required_tools(&config);
    if !required_tools.is_empty() {
        println!("📦 Required Tools:");
        for tool in &required_tools {
            match Command::new(tool.command).arg(tool.version_arg).output() {
                Ok(output) if output.status.success() => {
                    found_tools += 1;
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let version = stdout.lines().next().unwrap_or("").trim();
                    println!("✅ {}: {}", tool.name, version);
                }
                _ => {
                    missing_tools += 1;
                    all_checks_passed = false;
                    println!("❌ {} not found", tool.name);
                }
            }
        }
    }

    // Check Environment Variables if needed tools are detected
    let needs_android = required_tools.iter().any(|t| t.name == "Flutter");
    let needs_java = needs_android || config.scripts.values().any(|s| s.contains("java ") || s.contains("gradle "));
    
    if needs_android || needs_java {
        println!("\n🔧 Environment Variables:");
        if needs_android {
            if std::env::var("ANDROID_HOME").is_ok() {
                found_tools += 1;
                println!("✅ ANDROID_HOME is set");
            } else {
                missing_tools += 1;
                all_checks_passed = false;
                println!("❌ ANDROID_HOME is not set");
            }
        }
        if needs_java {
            if std::env::var("JAVA_HOME").is_ok() {
                found_tools += 1;
                println!("✅ JAVA_HOME is set");
            } else {
                missing_tools += 1;
                all_checks_passed = false;
                println!("❌ JAVA_HOME is not set");
            }
        }
    }

    // Check Git Configuration if git commands are used
    if config.scripts.values().any(|s| s.contains("git ")) {
        println!("\n🔐 Git Configuration:");
        for key in ["user.name", "user.email"].iter() {
            match Command::new("git").args(["config", key]).output() {
                Ok(output) if output.status.success() => {
                    found_tools += 1;
                    println!("✅ Git {} is set", key);
                }
                _ => {
                    missing_tools += 1;
                    all_checks_passed = false;
                    println!("❌ Git {} is not set", key);
                }
            }
        }
    }

    let duration = start_time.elapsed();
    Ok((all_checks_passed, found_tools, missing_tools, duration))
}

fn update_tools() -> Result<(bool, Vec<String>)> {
    let config_path = PathBuf::from(CONFIG_FILE);
    let config = RobinConfig::load(&config_path)
        .with_context(|| "No .robin.json found. Run 'robin init' first")?;

    let required_tools = detect_required_tools(&config);
    let mut updated_tools = Vec::new();
    let mut all_success = true;

    println!("🔄 Updating development tools...\n");

    for tool in required_tools {
        match tool.name {
            "Node.js" => {
                if Command::new("npm").arg("--version").output().is_ok() {
                    println!("Updating npm packages...");
                    if !run_update_command("npm", &["update", "-g"])? {
                        all_success = false;
                    } else {
                        updated_tools.push("npm packages".to_string());
                    }
                }
            },
            "Ruby" | "Fastlane" => {
                if Command::new("gem").arg("--version").output().is_ok() {
                    println!("Updating Fastlane...");
                    if !run_update_command("gem", &["update", "fastlane"])? {
                        all_success = false;
                    } else {
                        updated_tools.push("Fastlane".to_string());
                    }
                }
            },
            "Flutter" => {
                if Command::new("flutter").arg("--version").output().is_ok() {
                    println!("Updating Flutter...");
                    if !run_update_command("flutter", &["upgrade"])? {
                        all_success = false;
                    } else {
                        updated_tools.push("Flutter".to_string());
                    }
                }
            },
            "Cargo" => {
                if Command::new("rustup").arg("--version").output().is_ok() {
                    println!("Updating Rust toolchain...");
                    if !run_update_command("rustup", &["update"])? {
                        all_success = false;
                    } else {
                        updated_tools.push("Rust".to_string());
                    }
                }
            },
            "CocoaPods" => {
                if cfg!(target_os = "macos") && Command::new("pod").arg("--version").output().is_ok() {
                    println!("Updating CocoaPods repos...");
                    if !run_update_command("pod", &["repo", "update"])? {
                        all_success = false;
                    } else {
                        updated_tools.push("CocoaPods".to_string());
                    }
                }
            },
            _ => {}
        }
    }

    if updated_tools.is_empty() {
        println!("No tools to update!");
    } else {
        println!("\n✅ Update complete!");
    }

    Ok((all_success, updated_tools))
}

fn check_tool(name: &str, cmd: &str, arg: &str) -> Result<()> {
    match Command::new(cmd).arg(arg).output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let version = stdout.lines()
                .next()
                .unwrap_or("")
                .trim();
            println!("✅ {}: {}", name, version);
        }
        _ => println!("❌ {} not found", name),
    }
    Ok(())
}

fn check_env_var(name: &str) -> Result<()> {
    match std::env::var(name) {
        Ok(_) => println!("✅ {} is set", name),
        Err(_) => println!("❌ {} is not set", name),
    }
    Ok(())
}

fn check_git_config(key: &str) -> Result<()> {
    match Command::new("git").args(["config", key]).output() {
        Ok(output) if output.status.success() => {
            println!("✅ Git {} is set", key);
        }
        _ => println!("❌ Git {} is not set", key),
    }
    Ok(())
}

fn run_update_command(cmd: &str, args: &[&str]) -> Result<bool> {
    let status = Command::new(cmd)
        .args(args)
        .status()
        .with_context(|| format!("Failed to run {} update", cmd))?;

    if !status.success() {
        println!("❌ {} update failed", cmd);
    }
    Ok(status.success())
}

fn send_notification(title: &str, message: &str, success: bool) -> Result<()> {
    if cfg!(target_os = "windows") {
        let icon = if success { "✅" } else { "❌" };
        let script = format!(
            "New-BurntToastNotification -Text '{}', '{} {}' -Silent",
            title, icon, message
        );
        Command::new("powershell")
            .arg("-Command")
            .arg(script)
            .output()?;
    } else {
        // Use notify-rust for Unix-like systems (Linux and macOS)
        let mut notification = Notification::new();
        
        notification
            .summary(title)
            .body(&format!("{} {}", if success { "✅" } else { "❌" }, message))
            .timeout(5000); // 5 seconds

        // Set appropriate icon based on the platform
        if cfg!(target_os = "macos") {
            notification.sound_name("default");
        } else {
            notification.icon(if success { "dialog-ok" } else { "dialog-error" });
        }

        notification.show()?;
    }
    Ok(())
}

fn run_script(script: &str, notify: bool) -> Result<()> {
    let start_time = std::time::Instant::now();
    
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
            &format!("Command '{}' {}", script.split_whitespace().next().unwrap_or(script), message),
            success,
        )?;
    }

    if !status.success() {
        println!("{}", "Script failed!".red());
    }

    Ok(())
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
        run_script(script, false)?;
    }

    Ok(())
}
