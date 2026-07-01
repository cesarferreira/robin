use robin::load_env_file;
use std::fs;
use tempfile::tempdir;

#[test]
fn loads_vars_from_dotenv_next_to_config() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join(".env"),
        "ROBIN_TEST_DOTENV_LOADED=from_dotenv\n",
    )
    .unwrap();

    // load_env_file only uses the config path's parent directory.
    load_env_file(&dir.path().join(".robin.json"));

    assert_eq!(
        std::env::var("ROBIN_TEST_DOTENV_LOADED").unwrap(),
        "from_dotenv"
    );
}

#[test]
fn missing_dotenv_is_not_an_error() {
    let dir = tempdir().unwrap();
    // No .env in the directory — must simply be a no-op.
    load_env_file(&dir.path().join(".robin.json"));
}
