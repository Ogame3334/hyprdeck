use crate::cli::ClipboardCommands;
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};

pub fn execute(command: ClipboardCommands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ClipboardCommands::Copy {
            mime_type,
            primary,
            sensitive,
        } => copy(mime_type.as_deref(), primary, sensitive),
        ClipboardCommands::Paste {
            mime_type,
            primary,
            no_newline,
        } => paste(mime_type.as_deref(), primary, no_newline),
        ClipboardCommands::Show {
            primary,
            no_newline,
        } => show(primary, no_newline),
        ClipboardCommands::Types { primary } => types(primary),
        ClipboardCommands::Clear { primary } => clear(primary),
    }
}

fn copy(
    mime_type: Option<&str>,
    primary: bool,
    sensitive: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut data = Vec::new();
    io::stdin().read_to_end(&mut data)?;
    if data.is_empty() {
        return Err("clipboard input is empty".into());
    }

    let mut args = Vec::new();
    if let Some(mime_type) = mime_type {
        args.extend(["--type", mime_type]);
    }
    if primary {
        args.push("--primary");
    }
    if sensitive {
        args.push("--sensitive");
    }

    let mut process = Command::new("wl-copy")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        // wl-copy keeps a child process alive to own the clipboard. Inherit
        // stderr so that child does not keep a Rust pipe open indefinitely.
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| tool_error("wl-copy", error))?;
    process
        .stdin
        .take()
        .expect("wl-copy stdin was requested")
        .write_all(&data)?;
    let status = process.wait()?;
    if !status.success() {
        return Err(format!("wl-copy failed with status: {status}").into());
    }
    Ok(())
}

fn paste(
    mime_type: Option<&str>,
    primary: bool,
    no_newline: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut args = Vec::new();
    if let Some(mime_type) = mime_type {
        args.extend(["--type", mime_type]);
    }
    if primary {
        args.push("--primary");
    }
    if no_newline {
        args.push("--no-newline");
    }
    let output = Command::new("wl-paste")
        .args(args)
        .output()
        .map_err(|error| tool_error("wl-paste", error))?;
    if !output.status.success() {
        return Err(format!(
            "wl-paste failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    io::stdout().write_all(&output.stdout)?;
    Ok(())
}

fn types(primary: bool) -> Result<(), Box<dyn std::error::Error>> {
    let output = list_types(primary)?;
    io::stdout().write_all(&output)?;
    Ok(())
}

fn show(primary: bool, no_newline: bool) -> Result<(), Box<dyn std::error::Error>> {
    let output = list_types(primary)?;
    let type_text = String::from_utf8_lossy(&output);
    let mime_types: Vec<&str> = type_text.lines().collect();
    if let Some(text_type) = mime_types
        .iter()
        .copied()
        .find(|mime_type| mime_type.starts_with("text/"))
    {
        return paste(Some(text_type), primary, no_newline);
    }

    if mime_types.is_empty() {
        println!("Clipboard is empty.");
    } else {
        println!(
            "Clipboard contains non-text data (MIME types: {}).",
            mime_types.join(", ")
        );
    }
    Ok(())
}

fn list_types(primary: bool) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut args = vec!["--list-types"];
    if primary {
        args.push("--primary");
    }
    let output = Command::new("wl-paste")
        .args(args)
        .output()
        .map_err(|error| tool_error("wl-paste", error))?;
    if !output.status.success() {
        return Err(format!(
            "wl-paste failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    Ok(output.stdout)
}

fn clear(primary: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut args = vec!["--clear"];
    if primary {
        args.push("--primary");
    }
    let status = Command::new("wl-copy")
        .args(args)
        .status()
        .map_err(|error| tool_error("wl-copy", error))?;
    if !status.success() {
        return Err(format!("wl-copy failed with status: {status}").into());
    }
    Ok(())
}

fn tool_error(tool: &str, error: io::Error) -> Box<dyn std::error::Error> {
    format!("{tool} is required: {error}").into()
}
