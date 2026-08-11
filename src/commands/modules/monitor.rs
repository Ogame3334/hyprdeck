use crate::cli::MonitorCommands;

use std::process::{Command, Stdio};

pub fn execute(commands: MonitorCommands) -> Result<(), Box<dyn std::error::Error>> {
    match commands {
        MonitorCommands::Status => {
            let output = Command::new("hyprctl").arg("monitors").output()?;

            let stdout = String::from_utf8(output.stdout)?;

            println!("{}", stdout.trim());
        }

        MonitorCommands::Extend => {
            let monitor_names = get_monitor_names()?;

            println!("{}", monitor_names.len());

            if monitor_names.len() < 2 {
                return Ok(());
            }

            Command::new("hyprctl")
                .args([
                    "eval".to_string(),
                    format!(
                        "'hl.monitor({{ output = \"{}\", mode = \"1920x1080@60\", position = \"0x0\", scale = 1 }})'",
                        monitor_names[0]
                    ),
                ])
                .stdout(Stdio::null())
                .status()?;
            Command::new("hyprctl")
                .args([
                    "eval".to_string(),
                    format!(
                        "'hl.monitor({{ output = \"{}\", mode = \"1920x1080@60\", position = \"0x0\", scale = 1, mirror = \"\" }})'",
                        monitor_names[1]
                    ),
                ])
                .stdout(Stdio::null())
                .status()?;
        }
        MonitorCommands::Mirror => {
            let monitor_names = get_monitor_names()?;

            if monitor_names.len() < 2 {
                return Ok(());
            }

            Command::new("hyprctl")
                .args([
                    "eval".to_string(),
                    format!(
                        "'hl.monitor({{ output = \"{}\", mirror = \"{}\" }})'",
                        monitor_names[1], monitor_names[0],
                    ),
                ])
                .stdout(Stdio::null())
                .status()?;
        }
    }
    Ok(())
}

fn get_monitor_names() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("hyprctl").args(["monitors", "-j"]).output()?;

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;

    let names = json
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|monitor| monitor["name"].as_str())
        .map(String::from)
        .collect();

    Ok(names)
}
