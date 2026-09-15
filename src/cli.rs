use clap::{ArgGroup, Args, Parser, Subcommand};
use clap_complete::Shell;
use std::path::PathBuf;

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

    Wifi {
        #[command(subcommand)]
        commands: WifiCommands,
    },

    Workspace {
        #[command(subcommand)]
        commands: WorkspaceCommands,
    },

    Monitor {
        #[command(subcommand)]
        commands: MonitorCommands,
    },

    /// Capture a PNG screenshot from the Hyprland session.
    Screenshot(ScreenshotArgs),

    Battery {
        #[command(subcommand)]
        commands: BatteryCommands,
    },

    Metrics {
        #[command(subcommand)]
        commands: MetricsCommands,
    },

    #[command(hide = true)]
    Completion { shell: Shell },
}

#[derive(Args)]
#[command(
    group(ArgGroup::new("target").args(["fast", "window", "monitor"]).multiple(false)),
    group(ArgGroup::new("destination").args(["output", "no_save"]).multiple(false))
)]
pub struct ScreenshotArgs {
    /// Capture the whole visible desktop without showing a selection UI.
    #[arg(long)]
    pub fast: bool,

    /// Capture the currently focused window.
    #[arg(long)]
    pub window: bool,

    /// Capture a monitor by name (for example, DP-1).
    #[arg(long, value_name = "NAME")]
    pub monitor: Option<String>,

    /// Freeze the desktop while selecting a region (requires hyprpicker).
    #[arg(long)]
    pub freeze: bool,

    /// Include the pointer in the screenshot.
    #[arg(long)]
    pub cursor: bool,

    /// Copy the PNG to the Wayland clipboard.
    #[arg(long)]
    pub clipboard: bool,

    /// Do not save a file. Useful with --clipboard.
    #[arg(long)]
    pub no_save: bool,

    /// Exact output file path.
    #[arg(short, long, value_name = "PATH", conflicts_with = "directory")]
    pub output: Option<PathBuf>,

    /// Directory for generated screenshots.
    #[arg(short = 'd', long, value_name = "DIR")]
    pub directory: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum BatteryCommands {
    /// Show battery and AC power information.
    Status(BatteryStatusArgs),
}

#[derive(Args)]
pub struct BatteryStatusArgs {
    /// Read only this battery (for example, BAT0).
    #[arg(long, value_name = "NAME")]
    pub battery: Option<String>,

    /// Include per-battery energy and health details.
    #[arg(long)]
    pub verbose: bool,

    /// Print a compact status suitable for status bars.
    #[arg(long)]
    pub short: bool,

    /// Print machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum MetricsCommands {
    /// Show a metrics snapshot.
    Status(MetricsStatusArgs),
    /// Print metrics repeatedly at a fixed interval.
    Watch(MetricsWatchArgs),
}

#[derive(Args, Clone)]
pub struct MetricsStatusArgs {
    #[arg(long)]
    pub cpu: bool,
    #[arg(long)]
    pub memory: bool,
    #[arg(long)]
    pub storage: bool,
    #[arg(long)]
    pub network: bool,
    #[arg(long)]
    pub temperature: bool,
    #[arg(long)]
    pub short: bool,
    #[arg(long)]
    pub verbose: bool,
    #[arg(long)]
    pub json: bool,
    #[arg(long, value_name = "PATH")]
    pub mount: Option<String>,
}

#[derive(Args)]
pub struct MetricsWatchArgs {
    #[command(flatten)]
    pub status: MetricsStatusArgs,
    /// Sampling interval in seconds.
    #[arg(long, default_value_t = 1.0, value_name = "SECONDS")]
    pub interval: f64,
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
        #[arg(long, value_name = "ID", default_value = "@DEFAULT_AUDIO_SINK@")]
        device: String,
    },
    Increase {
        #[arg(value_parser = parse_volume, default_value = "5%")]
        amount: Volume,
        #[arg(long, value_name = "ID", default_value = "@DEFAULT_AUDIO_SINK@")]
        device: String,
    },
    Decrease {
        #[arg(value_parser = parse_volume, default_value = "5%")]
        amount: Volume,
        #[arg(long, value_name = "ID", default_value = "@DEFAULT_AUDIO_SINK@")]
        device: String,
    },
    Mute {
        #[arg(long, value_name = "ID", default_value = "@DEFAULT_AUDIO_SINK@")]
        device: String,
    },
    Unmute {
        #[arg(long, value_name = "ID", default_value = "@DEFAULT_AUDIO_SINK@")]
        device: String,
    },
    ToggleMute {
        #[arg(long, value_name = "ID", default_value = "@DEFAULT_AUDIO_SINK@")]
        device: String,
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

    #[arg(long)]
    pub json: bool,

    #[arg(long, value_name = "ID", default_value = "@DEFAULT_AUDIO_SINK@")]
    pub device: String,
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

#[derive(Subcommand)]
pub enum WifiCommands {
    Connect,
}

#[derive(Subcommand)]
pub enum WorkspaceCommands {
    Move { workspace: String, monitor: String },
    Status { workspace: Option<String> },
}

#[derive(Subcommand)]
pub enum MonitorCommands {
    Status,
    Extend,
    Mirror,
}
