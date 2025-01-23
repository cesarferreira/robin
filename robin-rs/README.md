# Robin (Rust Version)

A Rust implementation of the Robin CLI tool - your own customizable CLI tool for running project-specific scripts.

## Features

- Define and run project-specific scripts via `.robin.json`
- Interactive mode with fuzzy search
- List all available commands
- Add new commands easily
- Cross-platform support
- Template initialization for different project types
- Variable substitution with default values
- Enum validation for variables

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

## Variable Substitution

### Basic Variables
Use `{{variable}}` in your scripts and pass them as `--variable=XXX` when running the command:

```json
{
    "scripts": {
        "deploy": "fastlane {{platform}} {{env}}"
    }
}
```

Then run:
```bash
robin deploy --platform=ios --env=staging
```

### Default Values
You can specify default values for variables using `{{variable=default}}` syntax:

```json
{
    "scripts": {
        "print": "echo {{env=prod}}",
        "deploy": "echo \"Deploying to {{env=staging}} with version {{version=latest}}\""
    }
}
```

Using default values:
```bash
robin print              # Will use default: prod
robin deploy            # Will use defaults: staging and latest

# Override defaults:
robin print --env=dev   # Will use: dev
robin deploy --env=prod --version=1.0.0  # Will use: prod and 1.0.0
```

### Enum Validation
You can restrict variable values to a specific set using `{{variable=[value1, value2, ...]}}` syntax:

```json
{
    "scripts": {
        "deploy": "echo \"Deploying to {{env=[staging, prod]}}\"",
        "build": "cargo build --{{mode=[debug, release]}}",
        "deploy:app": "fastlane {{platform=[ios, android]}} {{env=[dev, staging, prod]}} --track={{track=[alpha, beta, production]}}"
    }
}
```

Using enum validation:
```bash
# Simple validation
robin deploy --env=staging    # Works: 'staging' is allowed
robin deploy --env=prod      # Works: 'prod' is allowed
robin deploy --env=dev       # Fails: only 'staging' or 'prod' are allowed

# Build modes
robin build --mode=debug     # Works: 'debug' is allowed
robin build --mode=release   # Works: 'release' is allowed
robin build --mode=test      # Fails: only 'debug' or 'release' are allowed

# Multiple validations
robin deploy:app \
    --platform=ios \
    --env=staging \
    --track=beta            # Works: all values are allowed

robin deploy:app \
    --platform=web \        # Fails: 'web' is not in [ios, android]
    --env=staging \
    --track=beta
```

## Development Environment

### Doctor Command
The `doctor` command helps verify your development environment is properly set up:

```bash
robin doctor
```

This will check:
- 📦 Required Tools
  - Cargo and Rust
  - Ruby and Fastlane
  - Flutter
  - Node.js and npm
- 🔧 Environment Variables
  - ANDROID_HOME
  - JAVA_HOME
  - FLUTTER_ROOT
- 📱 Platform Tools
  - Android Debug Bridge (adb)
  - Xcode Command Line Tools
  - CocoaPods
- 🔐 Git Configuration
  - user.name
  - user.email

Example output:
```bash
🔍 Checking development environment...

📦 Required Tools:
✅ Cargo: cargo 1.75.0
✅ Rust: rustc 1.75.0
✅ Ruby: ruby 3.2.2
✅ Fastlane: fastlane 2.217.0
❌ Flutter not found
✅ Node.js: v20.10.0
✅ npm: 10.2.3

🔧 Environment Variables:
✅ ANDROID_HOME is set
✅ JAVA_HOME is set
❌ FLUTTER_ROOT is not set

📱 Platform Tools:
✅ Android Debug Bridge (adb): Android Debug Bridge version 1.0.41
✅ Xcode Command Line Tools: installed
✅ CocoaPods: 1.14.3

🔐 Git Configuration:
✅ Git user.name is set
✅ Git user.email is set
```

### Update Development Tools
To update all development tools to their latest versions:

```bash
robin doctor:update
```

This will update:
- Rust (via rustup)
- Flutter
- Fastlane (via gem)
- Global npm packages
- CocoaPods repositories

## License

MIT © [Cesar Ferreira](http://cesarferreira.com) 