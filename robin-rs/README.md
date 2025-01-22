# Robin (Rust Version)

A Rust implementation of the Robin CLI tool - your own customizable CLI tool for running project-specific scripts.

## Features

- Define and run project-specific scripts via `.robin.json`
- Interactive mode with fuzzy search
- List all available commands
- Add new commands easily
- Cross-platform support
- Template initialization for different project types

## Installation

```bash
# From source
cargo install --path .

# Once published to crates.io (coming soon)
cargo install robin
```

## Usage

### Initialize a new project

```bash
robin init
```

This creates a `.robin.json` file in your current directory with some template scripts.

### Using templates

```bash
robin init --template android
robin init --template ios
robin init --template flutter
robin init --template rails
```

### List all commands

```bash
robin --list
```

### Interactive mode

```bash
robin --interactive  # or -i
```

### Add a new command

```bash
robin add "deploy" "fastlane deliver --submit-to-review"
```

### Run a command

```bash
robin deploy staging
robin release beta
```

## Configuration

The `.robin.json` file structure:

```json
{
    "scripts": {
        "clean": "...",
        "deploy staging": "echo 'ruby deploy tool --staging'",
        "deploy production": "...",
        "release beta": "...",
        "release alpha": "...",
        "release dev": "..."
    }
}
```

## Parameter Passing

Use `{{variable}}` in your scripts and pass them as `--variable=XXX` when running the command:

```json
{
    "scripts": {
        "release": "ruby deploy_tool --{{env}}"
    }
}
```

Then run:

```bash
robin release --env=staging
robin release --env=production
```

## License

MIT © [Cesar Ferreira](http://cesarferreira.com) 