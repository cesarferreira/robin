use std::path::Path;

/// Loads a `.env` file living next to the given config file into the process
/// environment, so that both robin's own `${VAR:-default}` substitution and the
/// shell see the variables. Existing environment variables always win (standard
/// dotenv precedence), and a missing file is not an error.
///
/// Set `ROBIN_NO_DOTENV` to skip loading entirely.
pub fn load_env_file(config_path: &Path) {
    if std::env::var_os("ROBIN_NO_DOTENV").is_some() {
        return;
    }

    let dir = config_path.parent().unwrap_or_else(|| Path::new("."));
    let env_path = dir.join(".env");
    if env_path.is_file() {
        // Best-effort: never let a malformed .env break a real command.
        let _ = dotenvy::from_path(&env_path);
    }
}
