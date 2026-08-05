use crate::cli::TouchpadCommands;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn execute_touchpad(
    device_id: String,
    command: TouchpadCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    let state: PathBuf = state_file()?;

    match command {
        TouchpadCommands::On => {
            set_touchpad(device_id, true)?;

            if fs::exists(&state)? {
                fs::remove_file(&state)?;
            }

            println!("Touchpad: ON");
        }

        TouchpadCommands::Off => {
            set_touchpad(device_id, false)?;

            if !fs::exists(&state)? {
                fs::File::create(&state)?;
            }

            println!("Touchpad: OFF");
        }

        TouchpadCommands::Toggle => {
            if fs::exists(&state)? {
                set_touchpad(device_id, true)?;
                fs::remove_file(&state)?;

                println!("Touchpad: ON");
            } else {
                set_touchpad(device_id, false)?;
                fs::File::create(&state)?;

                println!("Touchpad: OFF");
            }
        }
        TouchpadCommands::Status { verbose } => {
            status_touchpad(device_id, verbose)?;
        }
    }

    Ok(())
}

fn set_touchpad(device_id: String, enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
    let value = if enabled { "true" } else { "false" };

    let lua = format!(
        r#"hl.device({{ name = "{}", enabled = {} }})"#,
        device_id, value
    );

    let status = Command::new("hyprctl")
        .args(["eval", &lua])
        .stdout(Stdio::null())
        .status()?;

    if !status.success() {
        return Err(format!("hyprctl failed with status: {status}").into());
    }

    Ok(())
}

fn state_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = dirs::state_dir()
        .expect("Failed to get state directory")
        .join("hyprdeck");

    fs::create_dir_all(&dir)?;

    Ok(dir.join("touchpad-disabled"))
}

fn status_touchpad(device_id: String, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let state = state_file()?;

    let enabled = !state.exists();

    if verbose {
        println!("Touchpad: {}", if enabled { "ON" } else { "OFF" });

        println!("Device: {}", device_id);
        println!("State: {}", state.display());
    } else {
        println!("{}", if enabled { "ON" } else { "OFF" });
    }

    Ok(())
}

pub fn get_touchpad_device_id() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("hyprctl").arg("devices").output()?;

    let stdout = String::from_utf8(output.stdout)?;

    for line in stdout.lines() {
        let line = line.trim();

        if line.ends_with("-touchpad") {
            return Ok(line.to_string());
        }
    }

    Err("Touchpad not found".into())
}
