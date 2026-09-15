use crate::cli::WifiCommands;

use inquire::{Password, Select};
use serde::Serialize;

use std::{collections::BTreeSet, process::Command};

#[derive(Clone, PartialEq)]
pub enum AccessPointBand {
    B2_4GHz,
    B5GHz,
    B6GHz,
}

#[derive(PartialEq)]
pub enum BandSelection {
    Auto,
    Specific(AccessPointBand),
}

impl std::fmt::Display for BandSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Specific(band) => write!(f, "{}", band.as_str()),
        }
    }
}

impl AccessPointBand {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::B2_4GHz => "2.4 GHz",
            Self::B5GHz => "5 GHz",
            Self::B6GHz => "6 GHz",
        }
    }
}

impl TryFrom<&str> for AccessPointBand {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "2.4 GHz" => Ok(Self::B2_4GHz),
            "5 GHz" => Ok(Self::B5GHz),
            "6 GHz" => Ok(Self::B6GHz),
            _ => Err(format!("unknown band: {}", value)),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum AccessPointSecurity {
    FREE,
    WEP,
    WPA1,
    WPA2,
    WPA3,
    WPA2WPA3,
    OWE,
}

impl TryFrom<&str> for AccessPointSecurity {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "" => Ok(Self::FREE),
            "WEP" => Ok(Self::WEP),
            "WPA1" => Ok(Self::WPA1),
            "WPA2" => Ok(Self::WPA2),
            "WPA3" => Ok(Self::WPA3),
            "WPA2 WPA3" | "WPA1 WPA2" | "WPA1 WPA2 WPA3" => Ok(Self::WPA2WPA3),
            "OWE" => Ok(Self::OWE),
            _ => Err(format!("unknown security: {}", value)),
        }
    }
}

#[derive(Clone)]
pub struct AccessPoint {
    bssid: String,
    ssid: String,
    band: AccessPointBand,
    security: AccessPointSecurity,
}

type AccessPoints = Vec<AccessPoint>;

pub fn execute(command: WifiCommands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        WifiCommands::Connect => {
            connect()?;
        }
        WifiCommands::Status { json } => status(json)?,
        WifiCommands::Scan { rescan, json } => scan(rescan, json)?,
        WifiCommands::Saved { json } => saved(json)?,
        WifiCommands::Disconnect { device } => disconnect(device.as_deref())?,
        WifiCommands::On => radio(true)?,
        WifiCommands::Off => radio(false)?,
    }

    Ok(())
}

#[derive(Debug, Serialize)]
struct DeviceStatus {
    device: String,
    kind: String,
    state: String,
    connection: String,
}

#[derive(Debug, Serialize)]
struct NetworkEntry {
    ssid: String,
    bssid: String,
    signal: Option<u32>,
    security: String,
    channel: Option<u32>,
    band: String,
    in_use: bool,
}

#[derive(Debug, Serialize)]
struct SavedConnection {
    name: String,
    uuid: String,
    kind: String,
    device: String,
}

fn status(json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let output = nmcli(&[
        "-t",
        "-e",
        "yes",
        "-f",
        "DEVICE,TYPE,STATE,CONNECTION",
        "device",
        "status",
    ])?;
    let devices: Vec<DeviceStatus> = output
        .lines()
        .filter_map(|line| {
            let fields = split_nmcli_line(line);
            (fields.len() >= 4).then(|| DeviceStatus {
                device: fields[0].clone(),
                kind: fields[1].clone(),
                state: fields[2].clone(),
                connection: fields[3].clone(),
            })
        })
        .collect();
    if json {
        println!("{}", serde_json::to_string_pretty(&devices)?);
    } else {
        for device in devices {
            println!(
                "{}: {} ({}, {})",
                device.device, device.state, device.kind, device.connection
            );
        }
    }
    Ok(())
}

fn scan(rescan: bool, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    if rescan {
        nmcli(&["device", "wifi", "rescan"])?;
    }
    let output = nmcli(&[
        "-t",
        "-e",
        "yes",
        "-f",
        "IN-USE,SSID,BSSID,SIGNAL,SECURITY,CHAN,BAND",
        "device",
        "wifi",
        "list",
    ])?;
    let entries: Vec<NetworkEntry> = output.lines().filter_map(parse_network_entry).collect();
    if json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
    } else {
        for entry in entries {
            println!(
                "{} {}% {} {} {}",
                if entry.in_use { "*" } else { " " },
                entry.signal.unwrap_or(0),
                entry.ssid,
                entry.band,
                entry.security
            );
        }
    }
    Ok(())
}

fn saved(json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let output = nmcli(&[
        "-t",
        "-e",
        "yes",
        "-f",
        "NAME,UUID,TYPE,DEVICE",
        "connection",
        "show",
    ])?;
    let entries: Vec<SavedConnection> = output
        .lines()
        .filter_map(|line| {
            let fields = split_nmcli_line(line);
            (fields.len() >= 4).then(|| SavedConnection {
                name: fields[0].clone(),
                uuid: fields[1].clone(),
                kind: fields[2].clone(),
                device: fields[3].clone(),
            })
        })
        .collect();
    if json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
    } else {
        for entry in entries {
            println!(
                "{} ({}, {}) [{}]",
                entry.name, entry.kind, entry.device, entry.uuid
            );
        }
    }
    Ok(())
}

fn disconnect(device: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let device = match device {
        Some(device) => device.to_owned(),
        None => active_wifi_device()?.ok_or("no active Wi-Fi device")?,
    };
    nmcli(&["device", "disconnect", &device])?;
    println!("Disconnected: {device}");
    Ok(())
}

fn radio(enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
    nmcli(&["radio", "wifi", if enabled { "on" } else { "off" }])?;
    println!("Wi-Fi: {}", if enabled { "on" } else { "off" });
    Ok(())
}

fn active_wifi_device() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let output = nmcli(&[
        "-t",
        "-e",
        "yes",
        "-f",
        "DEVICE,TYPE,STATE",
        "device",
        "status",
    ])?;
    Ok(output
        .lines()
        .filter_map(|line| {
            let fields = split_nmcli_line(line);
            (fields.len() >= 3 && fields[1] == "wifi" && fields[2].starts_with("connected"))
                .then(|| fields[0].clone())
        })
        .next())
}

fn nmcli(args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("nmcli")
        .args(args)
        .output()
        .map_err(|error| format!("nmcli is required: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "nmcli failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

fn parse_network_entry(line: &str) -> Option<NetworkEntry> {
    let fields = split_nmcli_line(line);
    if fields.len() < 7 {
        return None;
    }
    Some(NetworkEntry {
        in_use: fields[0] == "*",
        ssid: fields[1].clone(),
        bssid: fields[2].clone(),
        signal: fields[3].parse().ok(),
        security: fields[4].clone(),
        channel: fields[5].parse().ok(),
        band: fields[6].clone(),
    })
}

pub fn connect() -> Result<(), Box<dyn std::error::Error>> {
    let ssid = select_ssid()?;

    let access_points = get_access_points(&ssid)?;

    let (band_selection, access_points) = select_band(access_points)?;

    let access_point = select_access_point(&band_selection, &access_points)?;

    let mut _target: String;

    match access_point {
        Some(access_point) => {
            println!(
                "Selected: {} (band: {})",
                access_point.bssid, band_selection
            );

            match access_point.security {
                AccessPointSecurity::FREE | AccessPointSecurity::OWE => {
                    Command::new("sudo")
                        .args(["nmcli", "device", "wifi", "connect", &access_point.bssid])
                        .status()?;
                }
                _ => {
                    let password = Password::new("Password:")
                        .with_display_mode(inquire::PasswordDisplayMode::Masked)
                        .without_confirmation()
                        .prompt()
                        .unwrap();

                    remove_existing_connection(&access_point.ssid)?;

                    Command::new("sudo")
                        .args([
                            "nmcli",
                            "device",
                            "wifi",
                            "connect",
                            &access_point.bssid,
                            "password",
                            &password,
                        ])
                        .status()?;
                }
            }
        }
        None => match access_points[0].security {
            AccessPointSecurity::FREE | AccessPointSecurity::OWE => {
                Command::new("sudo")
                    .args(["nmcli", "device", "wifi", "connect", &access_points[0].ssid])
                    .status()?;
            }
            _ => {
                let password = Password::new("Password:")
                    .with_display_mode(inquire::PasswordDisplayMode::Masked)
                    .without_confirmation()
                    .prompt()
                    .unwrap();

                remove_existing_connection(&access_points[0].ssid)?;

                Command::new("sudo")
                    .args([
                        "nmcli",
                        "device",
                        "wifi",
                        "connect",
                        &access_points[0].ssid,
                        "password",
                        &password,
                    ])
                    .status()?;
            }
        },
    }

    Ok(())
}

fn select_band(
    mut access_points: AccessPoints,
) -> Result<(BandSelection, AccessPoints), Box<dyn std::error::Error>> {
    if access_points.len() <= 1 {
        return Ok((BandSelection::Auto, access_points));
    }

    let mut options: Vec<String> = vec!["auto".to_string()];

    for band in [
        AccessPointBand::B2_4GHz,
        AccessPointBand::B5GHz,
        AccessPointBand::B6GHz,
    ] {
        if access_points.iter().any(|ap| ap.band == band) {
            options.push(band.as_str().to_string());
        }
    }

    let selected = Select::new("Select band", options).prompt()?;

    if selected == "auto" {
        return Ok((BandSelection::Auto, access_points));
    }

    let band = AccessPointBand::try_from(selected.as_str())?;

    access_points.retain(|ap| ap.band == band);

    Ok((BandSelection::Specific(band), access_points))
}

fn select_access_point(
    band_selection: &BandSelection,
    access_points: &AccessPoints,
) -> Result<Option<AccessPoint>, Box<dyn std::error::Error>> {
    if matches!(band_selection, BandSelection::Auto) {
        return Ok(None);
    }

    let access_point = if access_points.len() <= 1 {
        access_points
            .first()
            .cloned()
            .ok_or("no access point found")?
    } else {
        let bssids: Vec<String> = access_points.iter().map(|ap| ap.bssid.clone()).collect();

        let selected = Select::new("Select BSSID", bssids).prompt()?;

        access_points
            .iter()
            .find(|ap| ap.bssid == selected)
            .cloned()
            .ok_or("selected access point not found")?
    };

    Ok(Some(access_point))
}

fn select_ssid() -> Result<String, Box<dyn std::error::Error>> {
    let stdout = nmcli(&["-t", "-f", "SSID", "device", "wifi", "list"])?;

    let ssids: BTreeSet<String> = stdout
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    let ssids: Vec<String> = ssids.into_iter().collect();

    if ssids.is_empty() {
        return Err("no Wi-Fi networks found".into());
    }
    let selected = Select::new("Select Wi-Fi", ssids).prompt()?;

    Ok(selected)
}

fn get_access_points(ssid: &str) -> Result<AccessPoints, Box<dyn std::error::Error>> {
    let output = Command::new("nmcli")
        .args([
            "-t",
            "-e",
            "yes",
            "-f",
            "SSID,BSSID,BAND,SECURITY",
            "device",
            "wifi",
            "list",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;

    let mut access_points = AccessPoints::new();

    for line in stdout.lines() {
        let fields = split_nmcli_line(line);

        if fields.len() < 4 {
            continue;
        }

        if fields[0].trim() != ssid.trim() {
            continue;
        }

        let access_point = AccessPoint {
            ssid: String::from(&fields[0]),
            bssid: String::from(&fields[1]),
            band: AccessPointBand::try_from(fields[2].trim())?,
            security: AccessPointSecurity::try_from(fields[3].trim())?,
        };

        access_points.push(access_point);
    }

    Ok(access_points)
}

fn remove_existing_connection(ssid: &str) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("nmcli")
        .args(["-t", "-f", "NAME", "connection", "show"])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;

    let existing = stdout.lines().map(str::trim).find(|name| *name == ssid);

    if let Some(name) = existing {
        Command::new("sudo")
            .args(["nmcli", "connection", "delete", name])
            .status()?;
    }

    Ok(())
}

fn split_nmcli_line(line: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut escaped = false;

    for c in line.chars() {
        if escaped {
            current.push(c);
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == ':' {
            result.push(current);
            current = String::new();
        } else {
            current.push(c);
        }
    }

    result.push(current);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_escaped_nmcli_fields() {
        assert_eq!(
            split_nmcli_line("*:Cafe\\:Net:AA\\:BB:80:WPA2"),
            vec!["*", "Cafe:Net", "AA:BB", "80", "WPA2"]
        );
    }

    #[test]
    fn parses_scan_entry() {
        let entry = parse_network_entry("*:Cafe\\:Net:AA\\:BB:80:WPA2:36:5 GHz").unwrap();
        assert!(entry.in_use);
        assert_eq!(entry.ssid, "Cafe:Net");
        assert_eq!(entry.signal, Some(80));
        assert_eq!(entry.channel, Some(36));
    }

    #[test]
    fn ignores_malformed_scan_entry() {
        assert!(parse_network_entry("only:two:fields").is_none());
    }
}
