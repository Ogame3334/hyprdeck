use crate::cli::{BatteryCommands, BatteryStatusArgs};
use serde::Serialize;
use std::fs;
use std::path::Path;

const POWER_SUPPLY: &str = "/sys/class/power_supply";

#[derive(Debug, Clone, Serialize)]
pub struct BatteryInfo {
    pub name: String,
    pub percentage: f64,
    pub status: String,
    pub energy_now_wh: Option<f64>,
    pub energy_full_wh: Option<f64>,
    pub power_watts: Option<f64>,
    pub health_percent: Option<f64>,
    pub cycle_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_remaining_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_to_full_seconds: Option<u64>,
}

#[derive(Debug, Serialize)]
struct BatteryReport {
    percentage: f64,
    status: String,
    power_watts: Option<f64>,
    time_remaining_seconds: Option<u64>,
    time_to_full_seconds: Option<u64>,
    ac_online: bool,
    batteries: Vec<BatteryInfo>,
}

pub fn execute(commands: BatteryCommands) -> Result<(), Box<dyn std::error::Error>> {
    match commands {
        BatteryCommands::Status(args) => status(args),
    }
}

fn status(args: BatteryStatusArgs) -> Result<(), Box<dyn std::error::Error>> {
    let report = read_report(Path::new(POWER_SUPPLY), args.battery.as_deref())?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else if args.short {
        println!("{:.0}% {}", report.percentage, report.status);
    } else {
        println!("Battery: {:.0}%", report.percentage);
        println!("Status: {}", report.status);
        if let Some(power) = report.power_watts {
            println!("Power: {power:.2} W");
        }
        if let Some(seconds) = report.time_remaining_seconds {
            println!("Time remaining: {}", format_duration(seconds));
        }
        if let Some(seconds) = report.time_to_full_seconds {
            println!("Time to full: {}", format_duration(seconds));
        }
        println!(
            "AC: {}",
            if report.ac_online {
                "online"
            } else {
                "offline"
            }
        );
        if args.verbose {
            for battery in &report.batteries {
                println!("\n{}:", battery.name);
                println!("  Capacity: {:.0}%", battery.percentage);
                println!("  Status: {}", battery.status);
                if let (Some(now), Some(full)) = (battery.energy_now_wh, battery.energy_full_wh) {
                    println!("  Energy: {now:.2} Wh / {full:.2} Wh");
                }
                if let Some(power) = battery.power_watts {
                    println!("  Power: {power:.2} W");
                }
                if let Some(health) = battery.health_percent {
                    println!("  Health: {health:.0}%");
                }
                if let Some(cycles) = battery.cycle_count {
                    println!("  Cycle count: {cycles}");
                }
            }
        }
    }
    Ok(())
}

fn read_report(
    root: &Path,
    selected: Option<&str>,
) -> Result<BatteryReport, Box<dyn std::error::Error>> {
    let mut batteries = Vec::new();
    let mut ac_online = false;
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let kind = read_value(&path, "type")
            .unwrap_or_default()
            .to_ascii_lowercase();
        if kind == "battery" && selected.is_none_or(|wanted| wanted == name) {
            batteries.push(read_battery(&path, name)?);
        } else if matches!(kind.as_str(), "mains" | "usb" | "usb-c") {
            ac_online |= read_value(&path, "online").as_deref() == Some("1");
        }
    }
    if batteries.is_empty() {
        return Err(match selected {
            Some(name) => format!("battery not found: {name}"),
            None => "no battery found".to_owned(),
        }
        .into());
    }

    let total_full: f64 = batteries
        .iter()
        .filter_map(|battery| battery.energy_full_wh)
        .sum();
    let total_now: f64 = batteries
        .iter()
        .filter_map(|battery| battery.energy_now_wh)
        .sum();
    let percentage = if total_full > 0.0 {
        (total_now / total_full * 100.0).clamp(0.0, 100.0)
    } else {
        batteries
            .iter()
            .map(|battery| battery.percentage)
            .sum::<f64>()
            / batteries.len() as f64
    };
    let status = aggregate_status(&batteries);
    let power_watts = sum_optional(batteries.iter().filter_map(|battery| battery.power_watts));
    let time_remaining_seconds = if status == "Discharging" {
        estimate_time(total_now, power_watts)
    } else {
        None
    };
    let time_to_full_seconds = if status == "Charging" {
        estimate_time((total_full - total_now).max(0.0), power_watts)
    } else {
        None
    };
    Ok(BatteryReport {
        percentage,
        status,
        power_watts,
        time_remaining_seconds,
        time_to_full_seconds,
        ac_online,
        batteries,
    })
}

fn read_battery(path: &Path, name: String) -> Result<BatteryInfo, Box<dyn std::error::Error>> {
    let percentage = read_value(path, "capacity")
        .ok_or("battery capacity is unavailable")?
        .parse::<f64>()?;
    let status = read_value(path, "status").unwrap_or_else(|| "Unknown".into());
    let now = read_energy(path, "energy_now").or_else(|| read_energy(path, "charge_now"));
    let full = read_energy(path, "energy_full").or_else(|| read_energy(path, "charge_full"));
    let design =
        read_energy(path, "energy_full_design").or_else(|| read_energy(path, "charge_full_design"));
    let power_watts = read_power(path);
    let health_percent = design
        .zip(full)
        .map(|(design, full)| (full / design * 100.0).clamp(0.0, 100.0));
    let time_remaining_seconds = if status == "Discharging" {
        estimate_time(now.unwrap_or_default(), power_watts)
    } else {
        None
    };
    let time_to_full_seconds = if status == "Charging" {
        estimate_time(
            (full.unwrap_or_default() - now.unwrap_or_default()).max(0.0),
            power_watts,
        )
    } else {
        None
    };
    Ok(BatteryInfo {
        name,
        percentage,
        status,
        energy_now_wh: now,
        energy_full_wh: full,
        power_watts,
        health_percent,
        cycle_count: read_value(path, "cycle_count").and_then(|value| value.parse().ok()),
        time_remaining_seconds,
        time_to_full_seconds,
    })
}

fn read_energy(path: &Path, key: &str) -> Option<f64> {
    let value = read_value(path, key)?.parse::<f64>().ok()?;
    if key.starts_with("energy") {
        Some(value / 1_000_000.0)
    } else {
        let voltage = read_value(path, "voltage_now")?.parse::<f64>().ok()?;
        Some(value * voltage / 1_000_000_000_000.0)
    }
}

fn read_power(path: &Path) -> Option<f64> {
    if let Some(value) = read_value(path, "power_now").and_then(|value| value.parse::<f64>().ok()) {
        return Some(value / 1_000_000.0);
    }
    let current = read_value(path, "current_now")?.parse::<f64>().ok()?;
    let voltage = read_value(path, "voltage_now")?.parse::<f64>().ok()?;
    Some(current * voltage / 1_000_000_000_000.0)
}

fn read_value(path: &Path, key: &str) -> Option<String> {
    fs::read_to_string(path.join(key))
        .ok()
        .map(|value| value.trim().to_owned())
}

fn aggregate_status(batteries: &[BatteryInfo]) -> String {
    if batteries.iter().any(|battery| battery.status == "Charging") {
        "Charging".into()
    } else if batteries
        .iter()
        .any(|battery| battery.status == "Discharging")
    {
        "Discharging".into()
    } else if batteries.iter().all(|battery| battery.status == "Full") {
        "Full".into()
    } else {
        batteries
            .first()
            .map(|battery| battery.status.clone())
            .unwrap_or_else(|| "Unknown".into())
    }
}

fn sum_optional(values: impl Iterator<Item = f64>) -> Option<f64> {
    let values: Vec<_> = values.collect();
    (!values.is_empty()).then(|| values.iter().sum())
}

fn estimate_time(energy_wh: f64, power_watts: Option<f64>) -> Option<u64> {
    power_watts
        .filter(|power| *power > 0.0)
        .map(|power| (energy_wh / power * 3600.0).round() as u64)
}

fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    if hours > 0 {
        format!("{hours} h {minutes:02} min")
    } else {
        format!("{minutes} min")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn fixture() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "hyprdeck-battery-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(path.join("BAT0")).unwrap();
        fs::create_dir_all(path.join("AC")).unwrap();
        for (key, value) in [
            ("type", "Battery"),
            ("capacity", "75"),
            ("status", "Discharging"),
            ("energy_now", "30000000"),
            ("energy_full", "40000000"),
            ("energy_full_design", "45000000"),
            ("power_now", "10000000"),
            ("cycle_count", "12"),
        ] {
            fs::write(path.join("BAT0").join(key), value).unwrap();
        }
        fs::write(path.join("AC/type"), "Mains").unwrap();
        fs::write(path.join("AC/online"), "1").unwrap();
        path
    }

    #[test]
    fn reads_and_aggregates_sysfs_values() {
        let path = fixture();
        let report = read_report(&path, None).unwrap();
        assert_eq!(report.percentage, 75.0);
        assert_eq!(report.status, "Discharging");
        assert_eq!(report.power_watts, Some(10.0));
        assert_eq!(report.time_remaining_seconds, Some(10_800));
        assert!(report.ac_online);
        assert_eq!(report.batteries[0].health_percent, Some(88.88888888888889));
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn rejects_unknown_battery() {
        let path = fixture();
        assert!(read_report(&path, Some("BAT9")).is_err());
        fs::remove_dir_all(path).unwrap();
    }
}
