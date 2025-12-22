mod db;
mod ui;
mod cli;

use clap::Parser;
use cli::{Cli, Commands};
use db::Db;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let db = Db::init()?;

    match &cli.command {
        Commands::Add { cmd, cwd, exit_code, session_id } => {
            db.add_entry(cmd, cwd.as_deref(), *exit_code, session_id.as_deref())?;
        }
        Commands::Search { query } => {
            if let Err(e) = ui::run(db, query.clone()) {
                eprintln!("Error running TUI: {}", e);
            }
        }
        Commands::Stats => {
            let stats = db.get_stats()?;
            println!("🔥 Top 10 Commands:");
            for (i, (cmd, count)) in stats.iter().enumerate() {
                println!("{}. {} ({})", i + 1, cmd, count);
            }
        }
    }

    Ok(())
}

