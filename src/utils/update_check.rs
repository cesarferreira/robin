use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};

const PKG_NAME: &str = "robin_cli_tool";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
/// How long to wait between network checks. We hit crates.io at most once a day.
const CHECK_INTERVAL_SECS: u64 = 60 * 60 * 24;
/// Keep the network check snappy so it never noticeably delays the CLI.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

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

/// Best-effort, throttled "new version available" check against crates.io.
///
/// It never fails the caller and never blocks for long: the network is queried
/// at most once per [`CHECK_INTERVAL_SECS`] (the result is cached on disk), and
/// a short notice is printed to stderr whenever the cached latest version is
/// newer than the running one. Set `ROBIN_NO_UPDATE_CHECK` to disable it.
pub async fn check_for_update() {
    if std::env::var_os("ROBIN_NO_UPDATE_CHECK").is_some() {
        return;
    }
    // Swallow every error: an update check must never break a real command.
    let _ = run_check().await;
}

async fn run_check() -> Result<()> {
    let now = unix_now();
    let path = cache_path().context("no cache directory available")?;
    let mut cache = read_cache(&path);

    // Only touch the network when the cached result is stale.
    if now.saturating_sub(cache.last_check) >= CHECK_INTERVAL_SECS {
        let latest = fetch_latest().await;
        // Record the attempt either way so we don't retry on every invocation;
        // keep any previously known version if the request failed.
        cache.last_check = now;
        if let Some(latest) = latest {
            cache.latest_version = Some(latest);
        }
        let _ = write_cache(&path, &cache);
    }

    if let Some(latest) = &cache.latest_version {
        if is_newer(latest, CURRENT_VERSION) {
            print_notice(latest);
        }
    }
    Ok(())
}

fn print_notice(latest: &str) {
    eprintln!(
        "\n{} A new version of robin is available: {} (you have {}).\n  Update with: {}",
        "➜".yellow().bold(),
        latest.green().bold(),
        CURRENT_VERSION,
        format!("cargo install {PKG_NAME}").cyan(),
    );
}

async fn fetch_latest() -> Option<String> {
    let url = format!("https://crates.io/api/v1/crates/{PKG_NAME}");
    let client = reqwest::Client::builder()
        .user_agent(concat!(
            "robin/",
            env!("CARGO_PKG_VERSION"),
            " (update-check; https://github.com/cesarferreira/robin)"
        ))
        .timeout(REQUEST_TIMEOUT)
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
}
