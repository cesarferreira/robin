use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use colored::*;
use serde::{Deserialize, Serialize};

const PKG_NAME: &str = "robin_cli_tool";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Set on the detached worker process so it never spawns another worker.
const WORKER_ENV: &str = "ROBIN_UPDATE_CHECK_WORKER";
/// Hidden subcommand the detached worker runs (see `Commands::UpdateCheck`).
const WORKER_SUBCOMMAND: &str = "__update-check";
/// How long to wait between network checks. We hit crates.io at most once a day.
const CHECK_INTERVAL_SECS: u64 = 60 * 60 * 24;
/// Timeout for the background worker. Nobody waits on it, but it must not hang.
const BACKGROUND_TIMEOUT: Duration = Duration::from_secs(5);
/// Timeout for `robin self-update`, which the user is actively waiting on.
const INTERACTIVE_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Default, Serialize, Deserialize)]
struct UpdateCache {
    last_check: u64,
    latest_version: Option<String>,
}

#[derive(Deserialize)]
struct CratesResponse {
    #[serde(rename = "crate")]
    krate: CrateInfo,
}

#[derive(Deserialize)]
struct CrateInfo {
    max_stable_version: Option<String>,
    newest_version: Option<String>,
}

fn update_checks_disabled() -> bool {
    std::env::var_os("ROBIN_NO_UPDATE_CHECK").is_some()
}

// ---------------------------------------------------------------------------
// Background check
// ---------------------------------------------------------------------------

/// Starts a detached worker process that refreshes the update cache.
///
/// The worker is spawned only when the cached result is older than a day, and
/// the caller never waits on it, so commands never pay for network latency.
pub fn spawn_background_check() {
    let is_worker = std::env::var_os(WORKER_ENV).is_some();
    let stale = cache_path()
        .map(|p| is_stale(&read_cache(&p), unix_now()))
        .unwrap_or(false);
    if !should_spawn_background_check(update_checks_disabled(), is_worker, stale) {
        return;
    }

    let Ok(executable) = std::env::current_exe() else {
        return;
    };
    let _ = Command::new(executable)
        .arg(WORKER_SUBCOMMAND)
        .env(WORKER_ENV, "1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

/// Body of the hidden worker command: queries crates.io and updates the cache.
pub async fn run_background_check() {
    if update_checks_disabled() {
        return;
    }
    let Some(path) = cache_path() else {
        return;
    };
    let mut cache = read_cache(&path);
    if !is_stale(&cache, unix_now()) {
        return;
    }
    // Record the attempt either way so a failing network doesn't cause a retry
    // on every invocation; keep any previously known version on failure.
    cache.last_check = unix_now();
    if let Some(latest) = fetch_latest(BACKGROUND_TIMEOUT).await {
        cache.latest_version = Some(latest);
    }
    let _ = write_cache(&path, &cache);
}

fn should_spawn_background_check(disabled: bool, is_worker: bool, stale: bool) -> bool {
    !disabled && !is_worker && stale
}

fn is_stale(cache: &UpdateCache, now: u64) -> bool {
    now.saturating_sub(cache.last_check) >= CHECK_INTERVAL_SECS
}

// ---------------------------------------------------------------------------
// Notification
// ---------------------------------------------------------------------------

/// Prints a notice to stderr when the cached latest version is newer than the
/// running one. Reads the cache only; it never touches the network.
/// Set `ROBIN_NO_UPDATE_CHECK` to disable it.
pub fn show_update_notification() {
    if update_checks_disabled() {
        return;
    }
    let Some(path) = cache_path() else {
        return;
    };
    if let Some(latest) = read_cache(&path).latest_version
        && is_newer(&latest, CURRENT_VERSION)
    {
        print_notice(&latest, detect_install_method());
    }
}

fn print_notice(latest: &str, method: InstallMethod) {
    let hint = match method.upgrade_command() {
        Some(cmd) => format!(
            "{} {}",
            "robin self-update".cyan(),
            format!("(runs: {cmd})").dimmed()
        ),
        None => format!("cargo install {PKG_NAME} --locked")
            .cyan()
            .to_string(),
    };
    eprintln!(
        "\n{} A new version of robin is available: {} (you have {}).\n  Update with: {}",
        "➜".yellow().bold(),
        latest.green().bold(),
        CURRENT_VERSION,
        hint,
    );
}

// ---------------------------------------------------------------------------
// `robin self-update`
// ---------------------------------------------------------------------------

/// Upgrades robin with the command that matches how it was installed.
///
/// Unless `force` is set, it first does a live crates.io check and stops when
/// robin is already up to date.
pub async fn self_update(force: bool) -> Result<()> {
    if !force {
        match fetch_latest(INTERACTIVE_TIMEOUT).await {
            Some(latest) if !is_newer(&latest, CURRENT_VERSION) => {
                println!(
                    "{} {}",
                    "✓".green(),
                    format!("robin is already up to date (v{CURRENT_VERSION}).").dimmed()
                );
                return Ok(());
            }
            Some(_) => {}
            None => eprintln!(
                "{} could not reach crates.io; trying the upgrade anyway.",
                "Warning:".yellow().bold()
            ),
        }
    }

    let method = detect_install_method();
    let Some((program, args)) = method.upgrade_argv() else {
        bail!(
            "Could not detect how robin was installed ({}). Upgrade it manually, for example: \
             `cargo install {PKG_NAME} --locked` or `cargo binstall {PKG_NAME} --force`.",
            std::env::current_exe()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "unknown path".into())
        );
    };
    let display = method.upgrade_command().unwrap_or_default();
    println!(
        "{} {}",
        "Upgrading robin with".green().bold(),
        display.cyan()
    );

    let status = Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("Failed to run `{display}`"))?;
    if !status.success() {
        bail!("Upgrade command failed: {display}");
    }
    println!("{}", "robin upgrade complete.".green().bold());
    Ok(())
}

// ---------------------------------------------------------------------------
// Install method detection
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InstallMethod {
    Cargo,
    CargoBinstall,
    Unknown,
}

impl InstallMethod {
    fn upgrade_argv(self) -> Option<(&'static str, &'static [&'static str])> {
        match self {
            InstallMethod::Cargo => Some(("cargo", &["install", PKG_NAME, "--locked"])),
            InstallMethod::CargoBinstall => Some(("cargo", &["binstall", PKG_NAME, "--force"])),
            InstallMethod::Unknown => None,
        }
    }

    fn upgrade_command(self) -> Option<String> {
        self.upgrade_argv()
            .map(|(program, args)| format!("{program} {}", args.join(" ")))
    }
}

#[derive(Deserialize)]
struct BinstallRecord {
    name: String,
}

fn detect_install_method() -> InstallMethod {
    match std::env::current_exe() {
        Ok(path) => {
            let cargo_home = std::env::var_os("CARGO_HOME")
                .map(PathBuf::from)
                .or_else(|| dirs::home_dir().map(|h| h.join(".cargo")));
            install_method_from_path(&path, cargo_home.as_deref())
        }
        Err(_) => InstallMethod::Unknown,
    }
}

fn install_method_from_path(path: &Path, cargo_home: Option<&Path>) -> InstallMethod {
    let in_cargo_bin = is_dot_cargo_bin_path(path)
        || cargo_home.is_some_and(|home| path.parent() == Some(home.join("bin").as_path()));
    if !in_cargo_bin {
        return InstallMethod::Unknown;
    }
    // Prefer the cargo home the binary actually lives in.
    let home = path.parent().and_then(Path::parent).or(cargo_home);
    if binstall_metadata_contains_robin(home) {
        InstallMethod::CargoBinstall
    } else {
        InstallMethod::Cargo
    }
}

fn is_dot_cargo_bin_path(path: &Path) -> bool {
    let path = path.to_string_lossy();
    let parts: Vec<&str> = path.split(['/', '\\']).filter(|p| !p.is_empty()).collect();
    matches!(parts.as_slice(), [.., ".cargo", "bin", binary] if !binary.is_empty())
}

fn binstall_metadata_contains_robin(cargo_home: Option<&Path>) -> bool {
    let Some(cargo_home) = cargo_home else {
        return false;
    };
    let Ok(metadata) = fs::read_to_string(cargo_home.join("binstall").join("crates-v1.json"))
    else {
        return false;
    };
    // cargo-binstall stores concatenated JSON objects rather than a JSON array.
    serde_json::Deserializer::from_str(&metadata)
        .into_iter::<BinstallRecord>()
        .filter_map(Result::ok)
        .any(|record| record.name == PKG_NAME)
}

// ---------------------------------------------------------------------------
// crates.io + cache helpers
// ---------------------------------------------------------------------------

async fn fetch_latest(timeout: Duration) -> Option<String> {
    let url = format!("https://crates.io/api/v1/crates/{PKG_NAME}");
    let client = reqwest::Client::builder()
        .user_agent(concat!(
            "robin/",
            env!("CARGO_PKG_VERSION"),
            " (update-check; https://github.com/cesarferreira/robin)"
        ))
        .timeout(timeout)
        .build()
        .ok()?;

    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }

    let body: CratesResponse = resp.json().await.ok()?;
    body.krate.max_stable_version.or(body.krate.newest_version)
}

fn cache_path() -> Option<PathBuf> {
    Some(dirs::cache_dir()?.join("robin").join("update_check.json"))
}

fn read_cache(path: &Path) -> UpdateCache {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write_cache(path: &Path, cache: &UpdateCache) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string(cache)?)?;
    Ok(())
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Compares dotted numeric versions numerically (so `1.0.10` > `1.0.9`). Any
/// parse failure yields `false`, so an unexpected version string never nags.
fn is_newer(candidate: &str, current: &str) -> bool {
    match (parse_version(candidate), parse_version(current)) {
        (Some(a), Some(b)) => a > b,
        _ => false,
    }
}

fn parse_version(v: &str) -> Option<(u64, u64, u64)> {
    // Drop any pre-release/build suffix (e.g. "1.2.3-beta.1" -> "1.2.3").
    let core = v.split(['-', '+']).next()?;
    let mut parts = core.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_patch_minor_major() {
        assert!(is_newer("1.0.3", "1.0.2"));
        assert!(is_newer("1.1.0", "1.0.9"));
        assert!(is_newer("2.0.0", "1.9.9"));
    }

    #[test]
    fn same_or_older_is_not_newer() {
        assert!(!is_newer("1.0.2", "1.0.2"));
        assert!(!is_newer("1.0.1", "1.0.2"));
        assert!(!is_newer("0.9.9", "1.0.0"));
    }

    #[test]
    fn comparison_is_numeric_not_lexical() {
        // "1.0.10" must beat "1.0.9" — a string compare would get this wrong.
        assert!(is_newer("1.0.10", "1.0.9"));
        assert!(!is_newer("1.0.9", "1.0.10"));
    }

    #[test]
    fn prerelease_suffix_is_ignored() {
        assert!(is_newer("1.0.3-beta.1", "1.0.2"));
        assert!(!is_newer("1.0.2-rc.1", "1.0.2"));
    }

    #[test]
    fn unparseable_versions_never_nag() {
        assert!(!is_newer("not-a-version", "1.0.2"));
        assert!(!is_newer("1.0", "1.0.2"));
        assert!(!is_newer("1.0.2.3", "1.0.2"));
    }

    #[test]
    fn background_check_spawns_only_when_enabled_stale_and_not_worker() {
        assert!(should_spawn_background_check(false, false, true));
        assert!(!should_spawn_background_check(false, false, false));
        assert!(!should_spawn_background_check(true, false, true));
        assert!(!should_spawn_background_check(false, true, true));
    }

    #[test]
    fn empty_cache_is_stale_and_recent_cache_is_not() {
        let now = 10 * CHECK_INTERVAL_SECS;
        assert!(is_stale(&UpdateCache::default(), now));
        let fresh = UpdateCache {
            last_check: now - 60,
            latest_version: None,
        };
        assert!(!is_stale(&fresh, now));
        let old = UpdateCache {
            last_check: now - CHECK_INTERVAL_SECS,
            latest_version: None,
        };
        assert!(is_stale(&old, now));
    }

    #[test]
    fn detects_cargo_install_in_dot_cargo_bin() {
        for path in [
            "/home/user/.cargo/bin/robin",
            "/Users/cesar/.cargo/bin/robin",
            r"C:\Users\user\.cargo\bin\robin.exe",
        ] {
            assert_eq!(
                install_method_from_path(Path::new(path), None),
                InstallMethod::Cargo,
                "{path}"
            );
        }
    }

    #[test]
    fn detects_cargo_install_in_custom_cargo_home() {
        let home = Path::new("/opt/cargo");
        assert_eq!(
            install_method_from_path(Path::new("/opt/cargo/bin/robin"), Some(home)),
            InstallMethod::Cargo
        );
    }

    #[test]
    fn detects_cargo_binstall_from_metadata() {
        let temp = tempfile::tempdir().unwrap();
        let bin = temp.path().join("bin");
        let meta = temp.path().join("binstall");
        fs::create_dir_all(&bin).unwrap();
        fs::create_dir_all(&meta).unwrap();
        fs::write(
            meta.join("crates-v1.json"),
            r#"{"name":"other"}{"name":"robin_cli_tool"}"#,
        )
        .unwrap();
        assert_eq!(
            install_method_from_path(&bin.join("robin"), Some(temp.path())),
            InstallMethod::CargoBinstall
        );
    }

    #[test]
    fn other_paths_are_unknown() {
        for path in [
            "/usr/local/bin/robin",
            "/opt/homebrew/bin/robin",
            "/repo/target/debug/robin",
        ] {
            assert_eq!(
                install_method_from_path(Path::new(path), Some(Path::new("/home/u/.cargo"))),
                InstallMethod::Unknown,
                "{path}"
            );
        }
    }

    #[test]
    fn upgrade_commands_match_install_method() {
        assert_eq!(
            InstallMethod::Cargo.upgrade_command().as_deref(),
            Some("cargo install robin_cli_tool --locked")
        );
        assert_eq!(
            InstallMethod::CargoBinstall.upgrade_command().as_deref(),
            Some("cargo binstall robin_cli_tool --force")
        );
        assert_eq!(InstallMethod::Unknown.upgrade_command(), None);
    }
}
