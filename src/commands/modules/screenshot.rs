use crate::cli::ScreenshotArgs;
use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize)]
struct ActiveWindow {
    at: [i64; 2],
    size: [i64; 2],
}

#[derive(Debug, Deserialize)]
struct Monitor {
    name: String,
    x: i64,
    y: i64,
    width: i64,
    height: i64,
}

pub fn execute(args: ScreenshotArgs) -> Result<(), Box<dyn std::error::Error>> {
    if args.no_save && !args.clipboard {
        return Err("--no-save requires --clipboard".into());
    }
    if args.freeze && (args.fast || args.window || args.monitor.is_some()) {
        return Err("--freeze can only be used with interactive region selection".into());
    }

    let geometry = if args.window {
        Some(active_window_geometry()?)
    } else if let Some(name) = args.monitor.as_deref() {
        Some(monitor_geometry(name)?)
    } else if args.fast {
        None
    } else {
        Some(select_region(args.freeze)?)
    };

    let output_path = if args.no_save {
        None
    } else {
        Some(output_path(
            args.output.as_deref(),
            args.directory.as_deref(),
        )?)
    };

    let mut grim = Command::new("grim");
    if args.cursor {
        grim.arg("-c");
    }
    if let Some(ref geometry) = geometry {
        grim.args(["-g", geometry]);
    }
    grim.arg("-").stdout(Stdio::piped()).stderr(Stdio::piped());

    let output = grim.output().map_err(|error| tool_error("grim", error))?;
    if !output.status.success() {
        return Err(format!("grim failed: {}", stderr(&output.stderr)).into());
    }
    if output.stdout.is_empty() {
        return Err("grim returned an empty screenshot".into());
    }

    if let Some(path) = output_path {
        fs::write(&path, &output.stdout)?;
        println!("{}", path.display());
    }

    if args.clipboard {
        let mut clipboard = Command::new("wl-copy")
            .args(["--type", "image/png"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            // wl-copy daemonizes by default. Do not pipe stderr: its child
            // keeps inherited descriptors open while it owns the clipboard,
            // which would make wait_with_output wait forever for EOF.
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|error| tool_error("wl-copy", error))?;
        clipboard
            .stdin
            .take()
            .expect("wl-copy stdin was requested")
            .write_all(&output.stdout)?;
        let status = clipboard.wait()?;
        if !status.success() {
            return Err(format!("wl-copy failed with status: {status}").into());
        }
    }

    Ok(())
}

fn select_region(freeze: bool) -> Result<String, Box<dyn std::error::Error>> {
    let mut frozen = None;
    if freeze {
        let child = Command::new("hyprpicker")
            .args(["-rz"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| tool_error("hyprpicker", error))?;
        thread::sleep(Duration::from_millis(200));
        frozen = Some(child);
    }

    let selected = Command::new("slurp")
        .output()
        .map_err(|error| tool_error("slurp", error));
    if let Some(mut child) = frozen {
        let _ = child.kill();
        let _ = child.wait();
    }
    let selected = selected?;
    if !selected.status.success() {
        return Err("region selection was cancelled".into());
    }
    let geometry = String::from_utf8(selected.stdout)?.trim().to_owned();
    if geometry.is_empty() {
        return Err("region selection was cancelled".into());
    }
    Ok(geometry)
}

fn active_window_geometry() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("hyprctl")
        .args(["activewindow", "-j"])
        .output()
        .map_err(|error| tool_error("hyprctl", error))?;
    if !output.status.success() {
        return Err(format!("hyprctl failed: {}", stderr(&output.stderr)).into());
    }
    let window: ActiveWindow = serde_json::from_slice(&output.stdout)?;
    Ok(format!(
        "{},{} {}x{}",
        window.at[0], window.at[1], window.size[0], window.size[1]
    ))
}

fn monitor_geometry(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()
        .map_err(|error| tool_error("hyprctl", error))?;
    if !output.status.success() {
        return Err(format!("hyprctl failed: {}", stderr(&output.stderr)).into());
    }
    let monitors: Vec<Monitor> = serde_json::from_slice(&output.stdout)?;
    let monitor = monitors
        .iter()
        .find(|monitor| monitor.name == name)
        .ok_or_else(|| format!("monitor not found: {name}"))?;
    Ok(format!(
        "{},{} {}x{}",
        monitor.x, monitor.y, monitor.width, monitor.height
    ))
}

fn output_path(
    explicit: Option<&Path>,
    directory: Option<&Path>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(path) = explicit {
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }
        return Ok(path.to_path_buf());
    }
    let directory = directory
        .map(PathBuf::from)
        .or_else(|| dirs::picture_dir().map(|path| path.join("Screenshots")))
        .ok_or("could not determine a pictures directory; use --output or --directory")?;
    fs::create_dir_all(&directory)?;
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let base = directory.join(format!("screenshot-{timestamp}"));
    let mut path = base.with_extension("png");
    let mut suffix = 1;
    while path.exists() {
        path = directory.join(format!("screenshot-{timestamp}-{suffix}.png"));
        suffix += 1;
    }
    Ok(path)
}

fn stderr(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim().to_owned()
}

fn tool_error(tool: &str, error: std::io::Error) -> Box<dyn std::error::Error> {
    format!("{tool} is required: {error}").into()
}
