use crate::cli::WorkspaceCommands;

use serde::Deserialize;
use std::process::{Command, Stdio};

pub fn execute(commands: WorkspaceCommands) -> Result<(), Box<dyn std::error::Error>> {
    match commands {
        WorkspaceCommands::Move { workspace, monitor } => {
            Command::new("hyprctl")
                .args([
                    "dispatch".to_string(),
                    format!(
                        "hl.dsp.workspace.move({{ workspace = \"{}\", monitor = \"{}\" }})",
                        workspace, monitor
                    ),
                ])
                .stdout(Stdio::null())
                .status()?;

            Ok(())
        }

        WorkspaceCommands::Status { workspace } => {
            match workspace.as_deref() {
                None => {
                    Command::new("hyprctl").arg("workspaces").status()?;
                }

                Some("now") => {
                    Command::new("hyprctl").arg("activeworkspace").status()?;
                }

                Some(workspace) => {
                    let workspace_id = workspace.parse::<i64>()?;

                    let output = Command::new("hyprctl")
                        .args(["workspaces", "-j"])
                        .output()?;

                    let workspaces: Vec<Workspace> = serde_json::from_slice(&output.stdout)?;

                    let workspace = workspaces
                        .iter()
                        .find(|workspace| workspace.id == workspace_id)
                        .ok_or("Workspace is not found.")?;

                    println!(
                        "workspace ID {} ({}) on monitor {}:",
                        workspace.name, workspace.id, workspace.monitor
                    );
                    println!("\tmonitorID: {}", workspace.monitor_id);
                    println!("\twindows: {}", workspace.windows);
                    println!(
                        "\thasfullscreen: {}",
                        if workspace.hasfullscreen { 1 } else { 0 }
                    );
                    println!("\tlastwindow: {}", workspace.lastwindow);
                    println!("\tlastwindowtitle: {}", workspace.lastwindowtitle);
                    println!(
                        "\tispersistent: {}",
                        if workspace.ispersistent { 1 } else { 0 }
                    );
                    println!("\ttiledLayout: {}\n\n", workspace.tiled_layoyt);
                }
            }

            Ok(())
        }
    }
}

#[derive(Debug, Deserialize)]
struct Workspace {
    id: i64,
    name: String,
    monitor: String,
    #[serde(rename = "monitorID")]
    monitor_id: i64,
    windows: i64,
    hasfullscreen: bool,
    lastwindow: String,
    lastwindowtitle: String,
    ispersistent: bool,
    #[serde(rename = "tiledLayout")]
    tiled_layoyt: String,
}
