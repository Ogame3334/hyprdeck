use crate::cli::VolumeCommands;

use std::process::Command;

pub fn execute_volume(commands: VolumeCommands) -> Result<(), Box<dyn std::error::Error>> {
    match commands {
        VolumeCommands::Status(args) => {
            let output = wpctl(&["get-volume", &args.device])?;
            let (volume, muted) = parse_status(&output)?;

            if args.json {
                println!(
                    "{}",
                    serde_json::json!({"device": args.device, "volume": volume, "percent": (volume * 100.0).round(), "muted": muted})
                );
            } else if args.value {
                println!("{:2}", volume);
            } else if args.percent {
                println!("{}%", (volume * 100.0).round());
            } else {
                println!(
                    "Volume: {:.0}%{}",
                    volume * 100.0,
                    if muted { " (muted)" } else { "" }
                );
            }
        }
        VolumeCommands::Set { volume, device } => set_volume(&device, &volume.0.to_string())?,
        VolumeCommands::Increase { amount, device } => {
            set_volume(&device, &format!("{}+", amount.0))?
        }
        VolumeCommands::Decrease { amount, device } => {
            set_volume(&device, &format!("{}-", amount.0))?
        }
        VolumeCommands::Mute { device } => set_mute(&device, "1")?,
        VolumeCommands::Unmute { device } => set_mute(&device, "0")?,
        VolumeCommands::ToggleMute { device } => set_mute(&device, "toggle")?,
    }

    Ok(())
}

fn wpctl(args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("wpctl")
        .args(args)
        .output()
        .map_err(|error| format!("wpctl is required: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "wpctl failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

fn parse_status(output: &str) -> Result<(f32, bool), Box<dyn std::error::Error>> {
    let volume = output
        .split_whitespace()
        .find_map(|token| token.parse::<f32>().ok())
        .ok_or("volume value not found")?;
    Ok((
        volume,
        output.split_whitespace().any(|token| token == "[MUTED]"),
    ))
}

fn set_volume(device: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
    wpctl(&["set-volume", device, value])?;
    println!("Volume updated: {device} {value}");
    Ok(())
}

fn set_mute(device: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
    wpctl(&["set-mute", device, value])?;
    println!("Mute updated: {device} {value}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_volume_and_mute_state() {
        assert_eq!(
            parse_status("Volume: 0.75 [MUTED]\n").unwrap(),
            (0.75, true)
        );
        assert_eq!(parse_status("Volume: 0.25\n").unwrap(), (0.25, false));
    }

    #[test]
    fn rejects_missing_volume() {
        assert!(parse_status("Volume: [MUTED]").is_err());
    }
}
