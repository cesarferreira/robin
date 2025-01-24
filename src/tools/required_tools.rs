use std::process::Command;
use anyhow::{Result, Context};
use crate::config::RobinConfig;

#[derive(Debug, PartialEq)]
pub struct RequiredTool {
    pub name: &'static str,
    pub command: &'static str,
    pub version_arg: &'static str,
    pub patterns: &'static [&'static str],
}

pub const KNOWN_TOOLS: &[RequiredTool] = &[
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

fn check_script_contains(script: &serde_json::Value, pattern: &str) -> bool {
    match script {
        serde_json::Value::String(cmd) => cmd.contains(pattern),
        serde_json::Value::Array(commands) => {
            commands.iter().any(|cmd| {
                cmd.as_str().map_or(false, |s| s.contains(pattern))
            })
        },
        _ => false,
    }
}

fn detect_required_tools(config: &RobinConfig) -> Vec<&'static RequiredTool> {
    KNOWN_TOOLS
        .iter()
        .filter(|tool| {
            config.scripts.values().any(|script| {
                tool.patterns.iter().any(|&pattern| check_script_contains(script, pattern))
            })
        })
        .collect()
}

pub fn check_environment() -> Result<(bool, usize, usize, std::time::Duration)> {
    let start_time = std::time::Instant::now();
    let mut all_checks_passed = true;
    let mut found_tools = 0;
    let mut missing_tools = 0;

    let config_path = std::path::PathBuf::from(crate::CONFIG_FILE);
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
    let needs_java = needs_android || config.scripts.values().any(|s| 
        check_script_contains(s, "java ") || check_script_contains(s, "gradle ")
    );
    
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
    if config.scripts.values().any(|s| check_script_contains(s, "git ")) {
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

pub fn update_tools() -> Result<(bool, Vec<String>)> {
    let config_path = std::path::PathBuf::from(crate::CONFIG_FILE);
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