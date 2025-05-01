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
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new config file
    Init {
        /// Template to use (android, ios, flutter, rails, node, python, rust, go)
        #[arg(long)]
        template: Option<String>,
    },
    
    /// Add a new task
    Add {
        /// Task name
        name: String,
        /// Command script
        script: String,
        /// Task description
        #[arg(long)]
        description: Option<String>,
    },

    /// Check development environment setup
    Doctor,

    /// Update development tools to their latest versions
    DoctorUpdate,
    
    /// Migrate from v1 (scripts) to v2 (tasks) format
    Migrate {
        /// Path to the legacy format config file
        #[arg(long, default_value = ".robin.json")]
        input: String,
        /// Path to save the new format config file
        #[arg(long, default_value = ".robin.json")]
        output: String,
        /// Force overwrite without confirmation
        #[arg(long)]
        force: bool,
    },

    /// Run a script
    #[command(external_subcommand)]
    Run(Vec<String>),
} 