# AGENTS.md

Guidance for AI coding agents working in the **robin** repo (the CLI itself).

## Using robin

This repo dogfoods robin via `.robin.json` (+ `.base_robin.json`, `.app_robin.json`).
To drive robin — running tasks, editing configs, understanding the config format —
read **[SKILL.md](./SKILL.md)**. Common tasks: `robin --list`, `robin build`, `robin test`.

## Keep SKILL.md up to date

**SKILL.md documents robin's user-facing surface and MUST stay in sync with the code.**
Whenever a change touches any of the following, update SKILL.md (and the README) in the
same commit:

- CLI flags or subcommands — `src/cli/commands.rs`
- Config format / task shapes / variable & env-var substitution — `src/config/`, `src/scripts/`
- Recognized environment variables (e.g. `ROBIN_NO_DOTENV`, `ROBIN_NO_UPDATE_CHECK`)
- The published JSON Schema — `schema/robin.schema.json`

Quick self-check after a surface change: does the `robin --help` output still match the
Quick reference table in SKILL.md? If not, the doc is stale.

## Building & verifying

- Build: `cargo build` · Lint: `cargo clippy -- -D warnings` · Format: `cargo fmt --all`
- Test: `cargo test` — add/adjust tests for changed behavior; don't claim done without a green run.

## Conventions

- Stacked branches via `stax`; keep PRs small and focused.
- No agent attribution in commits, PR titles/bodies, or code comments.
