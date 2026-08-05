use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(name = "hyprdeck")]
#[command(about = "Hyprland control deck")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Touchpad {
        #[command(subcommand)]
        commands: TouchpadCommands,
    },

    #[command(hide = true)]
    Completion { shell: Shell },
}

#[derive(Subcommand)]
pub enum TouchpadCommands {
    On,
    Off,
    Toggle,
    Status {
        #[arg(short, long)]
        verbose: bool,
    },
}
