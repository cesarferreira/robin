---
name: robin
description: Use when a project contains a .robin.json (or .base_robin.json / .app_robin.json) and you need to run, list, add, rename, or remove its tasks, scaffold robin in a new project, or edit the config — the robin CLI runs project-defined scripts with variables, sequences, and task references.
---

# robin

`robin` runs project-specific scripts declared in a `.robin.json` file — like `npm run`/`make`, but language-agnostic and shareable across a team. If a repo has a `.robin.json`, prefer `robin <task>` over re-deriving the raw command.

## Quick reference

| Goal | Command |
|------|---------|
| List every task (with descriptions) | `robin --list` (`-l`) |
| List Makefile targets | `robin --make --list` |
| Pick a task interactively (fuzzy) | `robin --interactive` (`-i`) |
| Pick a Makefile target interactively | `robin --make` or `robin --make -i` |
| Run a task | `robin <task>` |
| Run a Makefile target | `robin --make <target>` |
| Run with variables | `robin deploy --env=staging --platform=ios` |
| Preview without executing | `robin <task> --dry-run` |
| Run in another directory | `robin <task> --cwd ./path` |
| Desktop notification on finish | `robin <task> --notify` |
| Scaffold a config | `robin init [--template rust\|node\|python\|go\|android\|ios\|flutter\|rails\|nextjs]` |
| Add / remove / rename a task | `robin add "name" "cmd"` · `robin rm "name"` · `robin rename "old" "new"` |
| Add `desc` scaffolding to every task | `robin migrate` |
| Check the dev environment | `robin doctor` · `robin doctor-update` |

Robin searches the current directory and walks **up** to find `.robin.json`, so tasks run from anywhere inside the project. `--dry-run`, `--cwd`, `--notify`, and `--make` work before or after the task name.

## Makefile mode (`--make`)

When a project has a `Makefile` (or `makefile` / `GNUmakefile`) but no `.robin.json` — or you simply prefer Make targets — use `--make` to treat Makefile targets as robin tasks:

```bash
robin --make --list          # list targets with recipe previews / comments
robin --make                 # interactive picker over Makefile targets
robin --make build           # run `make -C <makefile-dir> build`
robin build --make           # same; robin flags also work after the target name
```

Robin walks up from the current directory to find the Makefile, mirroring `.robin.json` discovery. Each target runs via `make` so dependencies, variables, and `.PHONY` rules behave normally. Comments on the line above a target become the task description in `--list` and the interactive picker.

## Config format (`.robin.json`)

```json
{
  "$schema": "https://raw.githubusercontent.com/cesarferreira/robin/refs/heads/main/schema/robin.schema.json",
  "include": ["../common/robin.base.json"],
  "scripts": {
    "build": "cargo build --release",
    "test":  { "cmd": "cargo test", "desc": "Run the full test suite" },
    "ship":  { "cmd": ["@build", "scp target/release/app server:/srv"], "desc": "Build then deploy" }
  }
}
```

- **Task forms:** a string, an array of commands (a sequence), or an object `{ "cmd": <string|array>, "desc": "..." }`. `desc` shows in `--list` and the interactive picker.
- **Sequences** run in order; each line is echoed with `▶`; stops on first failure.
- **`@task` references** (inside a sequence) run another task by name; expanded recursively; cycles are errored.
- **`include`** merges scripts from other files; local scripts win on conflict.
- **Variables:** `{{name}}` filled from `--name=value`; `{{name=default}}` for a default; `{{name=[a,b]}}` for enum validation.
- **Env vars:** `${VAR:-default}` (unset/empty → default) and `${VAR-default}` (unset → default); bare `${VAR}` is left for the shell.
- **`.env`** next to the config is auto-loaded (real env wins; disable with `ROBIN_NO_DOTENV`).
- **`$schema`** gives editors autocomplete; robin preserves it when rewriting the file.

## Editing configs

Prefer `robin add`/`rm`/`rename`/`migrate` — they preserve `$schema` and formatting. Hand-edit JSON only for shapes those commands don't cover (adding `desc`, sequences, `include`, variables). Keep `desc` on every task so the picker and `--list` stay useful.

## Env vars

- `ROBIN_NO_DOTENV` — skip `.env` auto-loading.
- `ROBIN_NO_UPDATE_CHECK` — skip the once-a-day crates.io update check.

For exact flags: `robin --help` and `robin <subcommand> --help`.
