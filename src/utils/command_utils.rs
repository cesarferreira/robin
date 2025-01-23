use anyhow::{Result, anyhow};
use regex::Regex;
use serde_json;

pub fn split_command_and_args(args: &[String]) -> (String, Vec<String>) {
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

pub fn replace_variables(script: &serde_json::Value, args: &[String]) -> Result<serde_json::Value> {
    match script {
        serde_json::Value::String(cmd) => {
            let replaced = replace_variables_in_string(cmd, args)?;
            Ok(serde_json::Value::String(replaced))
        },
        serde_json::Value::Array(commands) => {
            let mut replaced_commands = Vec::new();
            for cmd in commands {
                if let Some(cmd_str) = cmd.as_str() {
                    let replaced = replace_variables_in_string(cmd_str, args)?;
                    replaced_commands.push(serde_json::Value::String(replaced));
                } else {
                    replaced_commands.push(cmd.clone());
                }
            }
            Ok(serde_json::Value::Array(replaced_commands))
        },
        _ => Ok(script.clone()),
    }
}

fn replace_variables_in_string(script: &str, args: &[String]) -> Result<String> {
    let var_regex = Regex::new(r"\{\{(\w+)(?:=([^}]+|\[[^\]]+\]))\}\}").unwrap();
    let mut result = script.to_string();
    
    for capture in var_regex.captures_iter(script) {
        let full_match = &capture[0];
        let var_name = &capture[1];
        let default_or_enum = capture.get(2).map(|m| m.as_str()).unwrap_or("");
        let var_pattern = format!("--{}=", var_name);

        if default_or_enum.starts_with('[') && default_or_enum.ends_with(']') {
            let allowed_values: Vec<&str> = default_or_enum[1..default_or_enum.len()-1]
                .split(',')
                .map(|s| s.trim())
                .collect();
            
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