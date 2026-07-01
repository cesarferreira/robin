use robin::config::RobinConfig;
use robin::tools::{check_environment, update_tools};
use serde_json::{Value, json};
use std::collections::HashMap;

fn config_with(scripts: &[(&str, Value)]) -> RobinConfig {
    let mut map = HashMap::new();
    for (name, script) in scripts {
        map.insert((*name).to_string(), script.clone());
    }
    RobinConfig {
        include: vec![],
        scripts: map,
    }
}

#[test]
fn check_environment_reports_nothing_for_toolless_config() {
    // A config that references no known tool must short-circuit to an
    // all-clear with zero checks — no shelling out, fully hermetic.
    let config = config_with(&[("hello", json!("echo hello world"))]);
    let (passed, found, missing, duration) = check_environment(&config).unwrap();

    assert!(passed);
    assert_eq!(found, 0);
    assert_eq!(missing, 0);
    assert!(duration.as_secs_f32() >= 0.0);
}

#[test]
fn check_environment_probes_detected_tools() {
    // git is detected and probed (read-only `git --version` / `git config`).
    // Whether or not git is installed, at least one check must be recorded.
    let config = config_with(&[("status", json!("git status"))]);
    let (_passed, found, missing, _duration) = check_environment(&config).unwrap();

    assert!(
        found + missing >= 1,
        "git should trigger at least one check"
    );
}

#[test]
fn update_tools_is_a_noop_without_updatable_tools() {
    // echo and git are not in the updatable set, so this must NOT execute any
    // real update command (no rustup/npm/gem/flutter/pod side effects).
    let config = config_with(&[("hello", json!("echo hi")), ("status", json!("git status"))]);
    let (success, updated) = update_tools(&config).unwrap();

    assert!(success);
    assert!(updated.is_empty(), "no updatable tools => nothing updated");
}
