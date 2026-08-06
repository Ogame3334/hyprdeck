use crate::cli::VolumeCommands;

use std::process::Command;

pub fn execute_volume(commands: VolumeCommands) -> Result<(), Box<dyn std::error::Error>> {
    match commands {
        VolumeCommands::Status(args) => {
            let output = Command::new("wpctl")
                .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
                .output()?;

            let stdout = String::from_utf8(output.stdout)?;

            if args.value {
                let volume = get_volume(stdout)?;
                println!("{:2}", volume);
            } else if args.percent {
                let volume = get_volume(stdout)?;
                println!("{}%", (volume * 100.0).round());
            } else {
                print!("{}", stdout);
            }
        }
        VolumeCommands::Set { volume } => {
            set_volume(volume.0)?;
        }
    }

    Ok(())
}

fn get_volume(out: String) -> Result<f32, Box<dyn std::error::Error>> {
    let volume = out
        .split_whitespace()
        .nth(1)
        .ok_or("volume value not found")?
        .parse::<f32>()?;

    Ok(volume)
}

fn set_volume(value: f32) -> Result<(), Box<dyn std::error::Error>> {
    Command::new("wpctl")
        .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &value.to_string()])
        .status()?;

    println!("Volume set to: {}", value);

    Ok(())
}
