mod cli;
mod commands;
mod completion;

use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Completion { shell } => {
            completion::generate(shell);
        }

        command => {
            commands::execute(command)?;
        }
    }

    Ok(())
}
