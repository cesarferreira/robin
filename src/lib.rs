pub const CONFIG_FILE: &str = ".robin.json";

pub mod cli;
pub mod config;
pub mod tools;
pub mod utils;
pub mod scripts;

pub use cli::{Cli, Commands};
pub use config::RobinConfig;
pub use tools::{check_environment, update_tools};
pub use utils::{send_notification, split_command_and_args, replace_variables};
pub use scripts::{run_script, list_commands, interactive_mode}; 