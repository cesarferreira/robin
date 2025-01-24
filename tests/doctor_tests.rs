mod common;

use robin::tools::{check_environment, update_tools};

#[test]
fn test_doctor_command() {
    let result = check_environment();
    assert!(result.is_ok());
    
    let (success, found, missing, duration) = result.unwrap();
    assert!(duration.as_secs_f32() >= 0.0);
    assert!(found + missing > 0);
}

#[test]
fn test_doctor_update() {
    let result = update_tools();
    assert!(result.is_ok());
    
    let (success, updated_tools) = result.unwrap();
    if success {
        assert!(!updated_tools.is_empty(), "Should have updated at least one tool");
    }
} 