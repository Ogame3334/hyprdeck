mod cli;
mod commands;

use clap::Parser;
use cli::Cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    commands::execute(cli.command)?;
    Ok(())
}
