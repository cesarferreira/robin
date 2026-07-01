use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// List all available commands
    #[arg(short, long)]
    pub list: bool,

    /// Interactive mode
    #[arg(short, long)]
    pub interactive: bool,

    /// Send system notification when command completes
    #[arg(long)]
    pub notify: bool,

    /// Print the fully-resolved command(s) without executing anything
    #[arg(long)]
    pub dry_run: bool,

    /// Run the task's commands in this directory instead of the current one
    #[arg(long, value_name = "DIR")]
    pub cwd: Option<std::path::PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new .robin.json file
    Init {
        /// Template to use (android, ios, flutter, rails, node, nextjs, python, rust, go)
        #[arg(long)]
        template: Option<String>,
    },

    /// Add a new command
    Add {
        /// Command name
        name: String,
        /// Command script
        script: String,
    },

    /// Remove a command
    #[command(alias = "rm")]
    Remove {
        /// Command name to remove
        name: String,
    },

    /// Rename a command
    Rename {
        /// Current command name
        from: String,
        /// New command name
        to: String,
    },

    /// Rewrite .robin.json so every task uses the object form with a `desc`
    /// field ready to be filled in (existing string/array tasks keep working)
    Migrate,

    /// Check development environment setup
    Doctor,

    /// Update development tools to their latest versions
    DoctorUpdate,

    /// Run a script
    #[command(external_subcommand)]
    Run(Vec<String>),
}
