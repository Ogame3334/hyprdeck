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

    Audio {
        #[command(subcommand)]
        commands: AudioCommands,
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
pub enum AudioCommands {
    Volume {
        #[command(subcommand)]
        commands: VolumeCommands,
    },
}

#[derive(Debug, Clone)]
pub struct Volume(pub f32);

#[derive(Subcommand)]
pub enum VolumeCommands {
    Status(VolumeStatusArgs),
    Set {
        #[arg(value_parser = parse_volume)]
        volume: Volume,
    },
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

fn parse_volume(s: &str) -> Result<Volume, String> {
    if let Some(percent) = s.strip_suffix('%') {
        let value: f32 = percent
            .parse()
            .map_err(|_| "invalid percentage".to_string())?;

        if !(0.0..=100.0).contains(&value) {
            return Err("percentage must be between 0 and 100".into());
        }

        return Ok(Volume(value / 100.0));
    }

    let value: f32 = s.parse().map_err(|_| "invalid volume".to_string())?;

    if !(0.0..=1.0).contains(&value) {
        return Err("volume must be between 0.0 and 1.0".into());
    }

    Ok(Volume(value))
}
