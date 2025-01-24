mod common;

use robin::utils::{split_command_and_args, replace_variables};
use serde_json::Value;

#[test]
fn test_split_command_and_args() {
    let args = vec!["command".to_string(), "--arg1=value1".to_string(), "--arg2=value2".to_string()];
    let (command, var_args) = split_command_and_args(&args);
    
    assert_eq!(command, "command");
    assert_eq!(var_args, vec!["--arg1=value1", "--arg2=value2"]);
}

#[test]
fn test_replace_variables() {
    let script = Value::String("echo {{VAR1}} {{VAR2}}".to_string());
    let args = vec![
        "--VAR1=hello".to_string(),
        "--VAR2=world".to_string()
    ];
    
    let result = replace_variables(&script, &args).unwrap();
    assert_eq!(result.as_str().unwrap(), "echo hello world");
}

#[test]
fn test_replace_variables_with_defaults() {
    let script = Value::String("echo {{VAR1=hello}} {{VAR2=world}}".to_string());
    let args = vec![];
    
    let result = replace_variables(&script, &args).unwrap();
    assert_eq!(result.as_str().unwrap(), "echo hello world");
}

#[test]
fn test_replace_variables_with_enum_validation() {
    let script = Value::String("deploy {{env=[staging,prod]}}".to_string());
    
    // Valid value
    let args = vec!["--env=staging".to_string()];
    let result = replace_variables(&script, &args).unwrap();
    assert_eq!(result.as_str().unwrap(), "deploy staging");
    
    // Invalid value
    let args = vec!["--env=invalid".to_string()];
    assert!(replace_variables(&script, &args).is_err());
}

#[test]
fn test_replace_variables_with_array_commands() {
    let commands = vec![
        "echo {{VAR1=hello}}".to_string(),
        "echo {{VAR2=world}}".to_string()
    ];
    let script = Value::Array(commands.into_iter().map(Value::String).collect());
    let args = vec![];
    
    let result = replace_variables(&script, &args).unwrap();
    let array = result.as_array().unwrap();
    assert_eq!(array[0].as_str().unwrap(), "echo hello");
    assert_eq!(array[1].as_str().unwrap(), "echo world");
} 