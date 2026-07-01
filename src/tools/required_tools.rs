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
        serde_json::Value::String(cmd) => command_uses(cmd, pattern),
        serde_json::Value::Array(commands) => {
            commands.iter().any(|cmd| {
                cmd.as_str().is_some_and(|s| command_uses(s, pattern))
            })
        },
        _ => false,
    }
}

/// Returns true when `pattern` (an invocation prefix such as `"go "`) appears
/// in `cmd` at a command boundary — i.e. at the start of the string or right
/// after whitespace or a shell separator. This prevents false positives like
/// `"cargo build"` matching the Go pattern `"go "` because of the trailing
/// `go` in `cargo`.
fn command_uses(cmd: &str, pattern: &str) -> bool {
    cmd.match_indices(pattern).any(|(idx, _)| {
        idx == 0
            || cmd[..idx]
                .chars()
                .next_back()
                .is_some_and(|prev| prev.is_whitespace() || matches!(prev, ';' | '&' | '|' | '('))
    })
}

pub fn detect_required_tools(config: &RobinConfig) -> Vec<&'static RequiredTool> {
    KNOWN_TOOLS
        .iter()
        .filter(|tool| {
            config.scripts.values().any(|script| {
                tool.patterns.iter().any(|&pattern| check_script_contains(script, pattern))
            })
        })
        .collect()
}

pub fn check_environment(config: &RobinConfig) -> Result<(bool, usize, usize, std::time::Duration)> {
    let start_time = std::time::Instant::now();
    let mut all_checks_passed = true;
    let mut found_tools = 0;
    let mut missing_tools = 0;

    println!("🔍 Checking development environment...\n");

    // Check Required Tools
    let required_tools = detect_required_tools(config);
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

pub fn update_tools(config: &RobinConfig) -> Result<(bool, Vec<String>)> {
    let required_tools = detect_required_tools(config);
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
            "CocoaPods"
                if cfg!(target_os = "macos")
                    && Command::new("pod").arg("--version").output().is_ok() =>
            {
                println!("Updating CocoaPods repos...");
                if !run_update_command("pod", &["repo", "update"])? {
                    all_success = false;
                } else {
                    updated_tools.push("CocoaPods".to_string());
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::collections::HashMap;

    fn config_with(scripts: &[(&str, Value)]) -> RobinConfig {
        let mut map = HashMap::new();
        for (name, script) in scripts {
            map.insert((*name).to_string(), script.clone());
        }
        RobinConfig { include: vec![], scripts: map }
    }

    fn tool_names(tools: &[&'static RequiredTool]) -> Vec<&'static str> {
        let mut names: Vec<_> = tools.iter().map(|t| t.name).collect();
        names.sort_unstable();
        names
    }

    #[test]
    fn check_script_contains_matches_string() {
        assert!(check_script_contains(&json!("cargo build --release"), "cargo "));
        assert!(!check_script_contains(&json!("cargo build"), "npm "));
    }

    #[test]
    fn check_script_contains_matches_any_array_element() {
        let script = json!(["echo hi", "docker compose up"]);
        assert!(check_script_contains(&script, "docker "));
        assert!(!check_script_contains(&script, "gradle "));
    }

    #[test]
    fn check_script_contains_ignores_non_string_types() {
        assert!(!check_script_contains(&json!(42), "cargo "));
        assert!(!check_script_contains(&json!([1, 2, 3]), "cargo "));
    }

    #[test]
    fn detect_required_tools_finds_only_referenced_tools() {
        let config = config_with(&[
            ("build", json!("cargo build")),
            ("web", json!("npm run dev")),
        ]);
        assert_eq!(tool_names(&detect_required_tools(&config)), vec!["Cargo", "Node.js"]);
    }

    #[test]
    fn detect_required_tools_deduplicates_across_scripts() {
        // Two scripts both reference cargo; the tool must appear only once.
        let config = config_with(&[
            ("build", json!("cargo build")),
            ("test", json!("cargo test")),
        ]);
        let tools = detect_required_tools(&config);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "Cargo");
    }

    #[test]
    fn detect_required_tools_matches_inside_arrays() {
        let config = config_with(&[("release", json!(["cargo build", "docker push img"]))]);
        assert_eq!(tool_names(&detect_required_tools(&config)), vec!["Cargo", "Docker"]);
    }

    #[test]
    fn detect_required_tools_returns_empty_when_nothing_matches() {
        let config = config_with(&[("hello", json!("echo hello world"))]);
        assert!(detect_required_tools(&config).is_empty());
    }

    #[test]
    fn detect_required_tools_requires_trailing_space_boundary() {
        // "gocardless" must not be mistaken for the Go toolchain ("go ").
        let config = config_with(&[("pay", json!("gocardless charge"))]);
        assert!(detect_required_tools(&config).is_empty());
    }

    #[test]
    fn cargo_script_does_not_falsely_detect_go() {
        // Regression: "cargo build" contains the substring "go " but must only
        // detect Cargo, never the Go toolchain.
        let config = config_with(&[("build", json!("cargo build --release"))]);
        assert_eq!(tool_names(&detect_required_tools(&config)), vec!["Cargo"]);
    }

    #[test]
    fn command_uses_respects_boundaries() {
        assert!(command_uses("go build", "go "));
        assert!(command_uses("cd svc && go test ./...", "go ")); // after "&& "
        assert!(command_uses("npm run dev", "npm "));
        assert!(!command_uses("cargo build", "go ")); // trailing "go" in cargo
        assert!(!command_uses("gocardless pay", "go ")); // no boundary
    }
} 