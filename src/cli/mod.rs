use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hun")]
#[command(about = "The History Unification Node", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new entry to the history
    Add {
        /// The command that was executed
        #[arg(short, long)]
        cmd: String,

        /// The current working directory
        #[arg(long)]
        cwd: Option<String>,

        /// The exit code of the command
        #[arg(long)]
        exit_code: Option<i32>,
        
        /// The session ID
        #[arg(long)]
        session_id: Option<String>,
    },
    /// Search the history (opens TUI)
    Search {
        /// Optional initial query
        #[arg(short, long)]
        query: Option<String>,
    },
    /// Show stats (placeholder)
    Stats,
}
