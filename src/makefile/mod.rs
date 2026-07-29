use anyhow::{Context, Result, anyhow};
use regex::Regex;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const MAKEFILE_NAMES: &[&str] = &["Makefile", "makefile", "GNUmakefile"];

/// Walks up from `start` (inclusive) looking for a Makefile.
pub fn find_makefile_from(start: &Path) -> Option<PathBuf> {
    let mut dir = Some(start);
    while let Some(current) = dir {
        for name in MAKEFILE_NAMES {
            let candidate = current.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        dir = current.parent();
    }
    None
}

/// Resolves the Makefile path for read commands: the nearest Makefile found by
/// walking up from the current directory, falling back to `./Makefile`.
pub fn find_makefile_path() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    find_makefile_from(&cwd).unwrap_or_else(|| cwd.join("Makefile"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MakefileTarget {
    name: String,
    description: Option<String>,
}

/// Parses a Makefile and returns robin-compatible script entries keyed by target
/// name. Each entry runs the target via `make` so dependencies and variables are
/// handled correctly.
pub fn parse_makefile(content: &str, makefile_dir: &Path) -> Result<HashMap<String, Value>> {
    let targets = parse_targets(content)?;
    let dir = makefile_dir
        .canonicalize()
        .unwrap_or_else(|_| makefile_dir.to_path_buf());
    let make_cmd = format!("make -C {}", shell_quote(&dir.display().to_string()));

    let mut scripts = HashMap::new();
    for target in targets {
        let cmd = format!("{} {}", make_cmd, target.name);
        let entry = if let Some(desc) = target.description.filter(|s| !s.is_empty()) {
            json!({ "cmd": cmd, "desc": desc })
        } else {
            Value::String(cmd)
        };
        scripts.insert(target.name, entry);
    }

    Ok(scripts)
}

pub fn load_makefile_scripts(path: &Path) -> Result<HashMap<String, Value>> {
    if !path.exists() {
        return Err(anyhow!(
            "No Makefile found. Looked for Makefile, makefile, or GNUmakefile"
        ));
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read Makefile: {}", path.display()))?;

    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    parse_makefile(&content, dir)
}

fn shell_quote(s: &str) -> String {
    if s.chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '-' | '_' | '.'))
    {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

fn parse_targets(content: &str) -> Result<Vec<MakefileTarget>> {
    let target_line = Regex::new(
        r"^([A-Za-z0-9_.][A-Za-z0-9_.$-]*(?:\s+[A-Za-z0-9_.][A-Za-z0-9_.$-]*)*)\s*:([^=].*)?$",
    )
    .unwrap();

    let mut phony = HashSet::new();
    let mut pending_comment: Option<String> = None;
    let mut targets = Vec::new();
    let mut i = 0;
    let lines: Vec<&str> = content.lines().collect();

    while i < lines.len() {
        let raw = lines[i];
        let line = raw.trim_end();

        if let Some(rest) = line.strip_prefix(".PHONY:") {
            for name in rest.split_whitespace() {
                phony.insert(name.to_string());
            }
            pending_comment = None;
            i += 1;
            continue;
        }

        if line.starts_with('#') {
            pending_comment = Some(line.trim_start_matches('#').trim().to_string());
            i += 1;
            continue;
        }

        if line.is_empty() {
            i += 1;
            continue;
        }

        if let Some(caps) = target_line.captures(line) {
            let names = caps.get(1).unwrap().as_str();
            i += 1;

            while i < lines.len() && lines[i].starts_with('\t') {
                i += 1;
            }

            for name in names.split_whitespace() {
                if should_skip_target(name, &phony) {
                    continue;
                }
                targets.push(MakefileTarget {
                    name: name.to_string(),
                    description: pending_comment.clone(),
                });
            }
            pending_comment = None;
            continue;
        }

        pending_comment = None;
        i += 1;
    }

    if targets.is_empty() {
        return Err(anyhow!("No runnable targets found in Makefile"));
    }

    Ok(targets)
}

fn should_skip_target(name: &str, phony: &HashSet<String>) -> bool {
    if name.contains('%') {
        return true;
    }
    name.starts_with('.') && !phony.contains(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_targets_with_comments() {
        let content = "\
# Build the project
build:
\tcargo build

# Run tests
test: build
\tcargo test
";
        let scripts = parse_makefile(content, Path::new(".")).unwrap();
        assert!(scripts.contains_key("build"));
        assert!(scripts.contains_key("test"));
        assert_eq!(
            scripts["build"]["desc"].as_str().unwrap(),
            "Build the project"
        );
        assert!(
            scripts["build"]["cmd"]
                .as_str()
                .unwrap()
                .ends_with(" build")
        );
    }

    #[test]
    fn parse_phony_and_skip_internal_targets() {
        let content = "\
.PHONY: clean
clean:
\trm -rf target

.internal:
\techo hidden
";
        let scripts = parse_makefile(content, Path::new(".")).unwrap();
        assert!(scripts.contains_key("clean"));
        assert!(!scripts.contains_key(".internal"));
    }

    #[test]
    fn targets_without_comments_have_no_description() {
        let content = "\
demo:
\techo one
\techo two
";
        let scripts = parse_makefile(content, Path::new(".")).unwrap();
        assert!(scripts["demo"].is_string());
        assert!(scripts["demo"].as_str().unwrap().ends_with(" demo"));
    }

    #[test]
    fn find_makefile_from_walks_up() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("a/b");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(dir.path().join("Makefile"), "build:\n\ttrue\n").unwrap();

        assert_eq!(
            find_makefile_from(&nested),
            Some(dir.path().join("Makefile"))
        );
    }
}
