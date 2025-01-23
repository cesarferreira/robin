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
    /// Initialize a new .robin.json file
    Init {
        /// Template to use (android, ios, flutter, rails)
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

    /// Check development environment setup
    Doctor,

    /// Update development tools to their latest versions
    DoctorUpdate,

    /// Run a script
    #[command(external_subcommand)]
    Run(Vec<String>),
} 