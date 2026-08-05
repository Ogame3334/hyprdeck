use clap::{ArgGroup, Args, Parser, Subcommand};
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

    Volume {
        #[command(subcommand)]
        commands: VolumeCommands,
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

#[derive(Subcommand)]
pub enum VolumeCommands {
    Status(VolumeStatusArgs),
}

#[derive(Args)]
#[command(group(
    ArgGroup::new("format")
        .multiple(false)
))]
pub struct VolumeStatusArgs {
    #[arg(long)]
    pub percent: bool,

    #[arg(long)]
    pub value: bool,
}
