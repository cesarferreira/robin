use std::path::PathBuf;
use tempfile::TempDir;
use robin::CONFIG_FILE;

pub async fn setup() -> (TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join(CONFIG_FILE);
    (temp_dir, config_path)
} 