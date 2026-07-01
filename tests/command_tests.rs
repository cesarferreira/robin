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

// --- split_command_and_args edge cases ---

#[test]
fn split_empty_args() {
    let (command, var_args) = split_command_and_args(&[]);
    assert_eq!(command, "");
    assert!(var_args.is_empty());
}

#[test]
fn split_multiword_command_name() {
    // "deploy production" is a single command name with two words.
    let args = vec!["deploy".to_string(), "production".to_string()];
    let (command, var_args) = split_command_and_args(&args);
    assert_eq!(command, "deploy production");
    assert!(var_args.is_empty());
}

#[test]
fn split_multiword_command_with_flags() {
    let args = vec![
        "deploy".to_string(),
        "production".to_string(),
        "--env=prod".to_string(),
    ];
    let (command, var_args) = split_command_and_args(&args);
    assert_eq!(command, "deploy production");
    assert_eq!(var_args, vec!["--env=prod"]);
}

#[test]
fn split_flags_only() {
    let args = vec!["--only=flags".to_string()];
    let (command, var_args) = split_command_and_args(&args);
    assert_eq!(command, "");
    assert_eq!(var_args, vec!["--only=flags"]);
}

#[test]
fn split_positional_after_first_flag_stays_with_args() {
    // Once a flag appears, everything after it is treated as an argument,
    // even bare words — they must not rejoin the command name.
    let args = vec![
        "run".to_string(),
        "--flag=1".to_string(),
        "trailing".to_string(),
    ];
    let (command, var_args) = split_command_and_args(&args);
    assert_eq!(command, "run");
    assert_eq!(var_args, vec!["--flag=1", "trailing"]);
}

// --- replace_variables error / override cases ---

#[test]
fn missing_required_variable_errors() {
    let script = Value::String("echo {{NAME}}".to_string());
    let err = replace_variables(&script, &[]).unwrap_err();
    assert!(err.to_string().contains("Missing required variable"), "{err}");
}

#[test]
fn missing_enum_variable_errors() {
    let script = Value::String("deploy {{env=[staging,prod]}}".to_string());
    let err = replace_variables(&script, &[]).unwrap_err();
    assert!(err.to_string().contains("Missing required variable"), "{err}");
}

#[test]
fn provided_arg_overrides_default() {
    let script = Value::String("build {{mode=debug}}".to_string());
    let args = vec!["--mode=release".to_string()];
    let result = replace_variables(&script, &args).unwrap();
    assert_eq!(result.as_str().unwrap(), "build release");
}

#[test]
fn enum_tolerates_spaces_in_definition() {
    // Robin's own templates use "{{env=[staging, prod]}}" with a space.
    let script = Value::String("deploy {{env=[staging, prod]}}".to_string());
    let args = vec!["--env=prod".to_string()];
    let result = replace_variables(&script, &args).unwrap();
    assert_eq!(result.as_str().unwrap(), "deploy prod");
}

#[test]
fn repeated_variable_is_replaced_everywhere() {
    let script = Value::String("{{x}} and {{x}}".to_string());
    let args = vec!["--x=hi".to_string()];
    let result = replace_variables(&script, &args).unwrap();
    assert_eq!(result.as_str().unwrap(), "hi and hi");
}

#[test]
fn non_string_script_is_returned_unchanged() {
    // Numbers/objects aren't templated; they pass through as-is.
    let script = Value::from(42);
    let result = replace_variables(&script, &[]).unwrap();
    assert_eq!(result, Value::from(42));
}
