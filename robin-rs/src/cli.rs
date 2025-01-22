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

    /// Run a script
    #[command(external_subcommand)]
    Run(Vec<String>),
} 