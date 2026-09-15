use crate::cli::{MetricsCommands, MetricsStatusArgs};
use serde::Serialize;
use std::collections::HashMap;
use std::ffi::CString;
use std::fs;
use std::thread;
use std::time::Duration;

#[derive(Debug, Serialize, Default)]
struct MetricsReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    cpu: Option<CpuMetrics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    memory: Option<MemoryMetrics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    storage: Option<Vec<StorageMetrics>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    network: Option<NetworkMetrics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperatures: Option<Vec<Temperature>>,
}

#[derive(Debug, Serialize)]
struct CpuMetrics {
    usage_percent: f64,
    load_average: [f64; 3],
    frequency_hz: Option<u64>,
}

#[derive(Debug, Serialize)]
struct MemoryMetrics {
    total_bytes: u64,
    used_bytes: u64,
    available_bytes: u64,
    usage_percent: f64,
    swap_total_bytes: u64,
    swap_used_bytes: u64,
}

#[derive(Debug, Serialize)]
struct StorageMetrics {
    mount: String,
    device: String,
    total_bytes: u64,
    used_bytes: u64,
    available_bytes: u64,
    usage_percent: f64,
}

#[derive(Debug, Serialize)]
struct NetworkMetrics {
    download_bytes_per_second: u64,
    upload_bytes_per_second: u64,
    interfaces: Vec<NetworkInterface>,
}

#[derive(Debug, Serialize)]
struct NetworkInterface {
    name: String,
    receive_bytes: u64,
    transmit_bytes: u64,
}

#[derive(Debug, Serialize)]
struct Temperature {
    name: String,
    celsius: f64,
}

#[derive(Default)]
struct Sample {
    cpu: CpuTicks,
    network: HashMap<String, (u64, u64)>,
}

#[derive(Default, Clone, Copy)]
struct CpuTicks {
    total: u64,
    idle: u64,
}

pub fn execute(commands: MetricsCommands) -> Result<(), Box<dyn std::error::Error>> {
    match commands {
        MetricsCommands::Status(args) => print_once(&args),
        MetricsCommands::Watch(args) => {
            if !args.interval.is_finite() || args.interval <= 0.0 {
                return Err("--interval must be greater than zero".into());
            }
            loop {
                print_once(&args.status)?;
                thread::sleep(Duration::from_secs_f64(args.interval));
            }
        }
    }
}

fn print_once(args: &MetricsStatusArgs) -> Result<(), Box<dyn std::error::Error>> {
    let all = !(args.cpu || args.memory || args.storage || args.network || args.temperature);
    let need_cpu = all || args.cpu;
    let need_network = all || args.network;
    let first = Sample::read(need_cpu, need_network)?;
    if need_cpu || need_network {
        thread::sleep(Duration::from_millis(100));
    }
    let second = Sample::read(need_cpu, need_network)?;
    let elapsed = Duration::from_millis(100);
    let report = collect_report(args, all, &first, &second, elapsed)?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else if args.short {
        print_short(&report);
    } else {
        print_human(&report, args.verbose);
    }
    Ok(())
}

fn collect_report(
    args: &MetricsStatusArgs,
    all: bool,
    first: &Sample,
    second: &Sample,
    elapsed: Duration,
) -> Result<MetricsReport, Box<dyn std::error::Error>> {
    Ok(MetricsReport {
        cpu: (all || args.cpu).then(|| cpu_metrics(first.cpu, second.cpu, elapsed)),
        memory: (all || args.memory).then(read_memory).transpose()?,
        storage: (all || args.storage)
            .then(|| read_storage(args.mount.as_deref()))
            .transpose()?,
        network: (all || args.network).then(|| network_metrics(first, second, elapsed)),
        temperatures: (all || args.temperature || args.cpu)
            .then(read_temperatures)
            .transpose()?,
    })
}

fn cpu_metrics(first: CpuTicks, second: CpuTicks, _elapsed: Duration) -> CpuMetrics {
    let total = second.total.saturating_sub(first.total);
    let idle = second.idle.saturating_sub(first.idle);
    let usage = if total == 0 {
        0.0
    } else {
        (total - idle) as f64 / total as f64 * 100.0
    };
    let load = fs::read_to_string("/proc/loadavg").unwrap_or_default();
    let mut values = load
        .split_whitespace()
        .filter_map(|value| value.parse().ok());
    CpuMetrics {
        usage_percent: usage,
        load_average: [
            values.next().unwrap_or(0.0),
            values.next().unwrap_or(0.0),
            values.next().unwrap_or(0.0),
        ],
        frequency_hz: read_frequency(),
    }
}

fn read_memory() -> Result<MemoryMetrics, Box<dyn std::error::Error>> {
    let values = proc_values("/proc/meminfo")?;
    let total = values.get("MemTotal").copied().unwrap_or(0) * 1024;
    let available = values.get("MemAvailable").copied().unwrap_or(0) * 1024;
    let swap_total = values.get("SwapTotal").copied().unwrap_or(0) * 1024;
    let swap_free = values.get("SwapFree").copied().unwrap_or(0) * 1024;
    let used = total.saturating_sub(available);
    Ok(MemoryMetrics {
        total_bytes: total,
        used_bytes: used,
        available_bytes: available,
        usage_percent: percent(used, total),
        swap_total_bytes: swap_total,
        swap_used_bytes: swap_total.saturating_sub(swap_free),
    })
}

fn read_storage(mount: Option<&str>) -> Result<Vec<StorageMetrics>, Box<dyn std::error::Error>> {
    let mounts = fs::read_to_string("/proc/mounts")?;
    let mut result = Vec::new();
    for line in mounts.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() < 3
            || (!fields[0].starts_with("/dev/") && !fields[0].starts_with("/run/media/"))
        {
            continue;
        }
        let path = fields[1].replace("\\040", " ");
        if mount.is_some_and(|wanted| wanted != path)
            || result
                .iter()
                .any(|entry: &StorageMetrics| entry.mount == path)
        {
            continue;
        }
        if let Some(stat) = statvfs(&path) {
            result.push(StorageMetrics {
                mount: path,
                device: fields[0].to_owned(),
                total_bytes: stat.0,
                used_bytes: stat.1,
                available_bytes: stat.2,
                usage_percent: percent(stat.1, stat.0),
            });
        }
    }
    if let Some(wanted) = mount {
        if result.is_empty() {
            return Err(format!("mount not found or unavailable: {wanted}").into());
        }
    }
    Ok(result)
}

fn statvfs(path: &str) -> Option<(u64, u64, u64)> {
    let path = CString::new(path).ok()?;
    let mut stat = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    if unsafe { libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) } != 0 {
        return None;
    }
    let stat = unsafe { stat.assume_init() };
    let block = stat.f_frsize as u64;
    let total = stat.f_blocks as u64 * block;
    let available = stat.f_bavail as u64 * block;
    Some((total, total.saturating_sub(available), available))
}

fn read_temperatures() -> Result<Vec<Temperature>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    let entries = match fs::read_dir("/sys/class/thermal") {
        Ok(entries) => entries,
        Err(_) => return Ok(result),
    };
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !entry
            .file_name()
            .to_string_lossy()
            .starts_with("thermal_zone")
        {
            continue;
        }
        let value = fs::read_to_string(path.join("temp"))
            .ok()
            .and_then(|value| value.trim().parse::<f64>().ok())
            .map(|value| value / 1000.0);
        if let Some(celsius) = value {
            let name = fs::read_to_string(path.join("type"))
                .unwrap_or_else(|_| entry.file_name().to_string_lossy().into_owned());
            result.push(Temperature {
                name: name.trim().into(),
                celsius,
            });
        }
    }
    Ok(result)
}

fn network_metrics(first: &Sample, second: &Sample, elapsed: Duration) -> NetworkMetrics {
    let seconds = elapsed.as_secs_f64();
    let mut interfaces = Vec::new();
    let mut download = 0.0;
    let mut upload = 0.0;
    for (name, &(rx, tx)) in &second.network {
        let (old_rx, old_tx) = first.network.get(name).copied().unwrap_or((rx, tx));
        let rx_rate = rx.saturating_sub(old_rx) as f64 / seconds;
        let tx_rate = tx.saturating_sub(old_tx) as f64 / seconds;
        download += rx_rate;
        upload += tx_rate;
        interfaces.push(NetworkInterface {
            name: name.clone(),
            receive_bytes: rx,
            transmit_bytes: tx,
        });
    }
    NetworkMetrics {
        download_bytes_per_second: download.round() as u64,
        upload_bytes_per_second: upload.round() as u64,
        interfaces,
    }
}

impl Sample {
    fn read(cpu: bool, network: bool) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            cpu: if cpu {
                read_cpu_ticks()?
            } else {
                CpuTicks::default()
            },
            network: if network {
                read_network()?
            } else {
                HashMap::new()
            },
        })
    }
}

fn read_cpu_ticks() -> Result<CpuTicks, Box<dyn std::error::Error>> {
    let line = fs::read_to_string("/proc/stat")?
        .lines()
        .next()
        .ok_or("/proc/stat is empty")?
        .to_owned();
    let values: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|value| value.parse().ok())
        .collect();
    Ok(CpuTicks {
        total: values.iter().sum(),
        idle: values.get(3).copied().unwrap_or(0) + values.get(4).copied().unwrap_or(0),
    })
}

fn read_network() -> Result<HashMap<String, (u64, u64)>, Box<dyn std::error::Error>> {
    let mut result = HashMap::new();
    for line in fs::read_to_string("/proc/net/dev")?.lines().skip(2) {
        let (name, values) = match line.split_once(':') {
            Some(value) => value,
            None => continue,
        };
        let name = name.trim();
        if name == "lo" || name.starts_with("docker") || name.starts_with("veth") {
            continue;
        }
        let values: Vec<u64> = values
            .split_whitespace()
            .filter_map(|value| value.parse().ok())
            .collect();
        if values.len() >= 9 {
            result.insert(name.to_owned(), (values[0], values[8]));
        }
    }
    Ok(result)
}

fn proc_values(path: &str) -> Result<HashMap<String, u64>, Box<dyn std::error::Error>> {
    let mut result = HashMap::new();
    for line in fs::read_to_string(path)?.lines() {
        if let Some((key, value)) = line.split_once(':') {
            if let Some(value) = value
                .split_whitespace()
                .next()
                .and_then(|value| value.parse().ok())
            {
                result.insert(key.to_owned(), value);
            }
        }
    }
    Ok(result)
}

fn read_frequency() -> Option<u64> {
    let path = fs::read_dir("/sys/devices/system/cpu")
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("cpufreq/scaling_cur_freq"))
        .find(|path| path.exists())?;
    fs::read_to_string(path)
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()
        .map(|value| value * 1000)
}

fn percent(value: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        value as f64 / total as f64 * 100.0
    }
}

fn print_short(report: &MetricsReport) {
    let mut values = Vec::new();
    if let Some(cpu) = &report.cpu {
        values.push(format!("CPU {:.0}%", cpu.usage_percent));
    }
    if let Some(memory) = &report.memory {
        values.push(format!("MEM {:.0}%", memory.usage_percent));
    }
    if let Some(storage) = &report.storage {
        if let Some(root) = storage.iter().find(|entry| entry.mount == "/") {
            values.push(format!("SSD {:.0}%", root.usage_percent));
        }
    }
    if let Some(temperatures) = &report.temperatures {
        if let Some(temp) = temperatures.first() {
            values.push(format!("TEMP {:.0}°C", temp.celsius));
        }
    }
    println!("{}", values.join(" | "));
}

fn print_human(report: &MetricsReport, verbose: bool) {
    if let Some(cpu) = &report.cpu {
        println!(
            "CPU:\n  Usage: {:.1}%\n  Load: {:.2} {:.2} {:.2}",
            cpu.usage_percent, cpu.load_average[0], cpu.load_average[1], cpu.load_average[2]
        );
        if let Some(freq) = cpu.frequency_hz {
            println!("  Frequency: {:.2} GHz", freq as f64 / 1e9);
        }
    }
    if let Some(temperatures) = &report.temperatures {
        if !temperatures.is_empty() {
            println!("Temperature:");
            for temp in temperatures {
                println!("  {}: {:.1}°C", temp.name, temp.celsius);
            }
        }
    }
    if let Some(memory) = &report.memory {
        println!(
            "Memory:\n  Usage: {} / {} ({:.1}%)",
            human_bytes(memory.used_bytes),
            human_bytes(memory.total_bytes),
            memory.usage_percent
        );
        println!(
            "  Swap: {} / {} ({:.1}%)",
            human_bytes(memory.swap_used_bytes),
            human_bytes(memory.swap_total_bytes),
            percent(memory.swap_used_bytes, memory.swap_total_bytes)
        );
    }
    if let Some(storage) = &report.storage {
        println!("Storage:");
        for entry in storage {
            println!(
                "  {}: {} / {} ({:.1}%)",
                entry.mount,
                human_bytes(entry.used_bytes),
                human_bytes(entry.total_bytes),
                entry.usage_percent
            );
            if verbose {
                println!(
                    "    Device: {} (free {})",
                    entry.device,
                    human_bytes(entry.available_bytes)
                );
            }
        }
    }
    if let Some(network) = &report.network {
        println!(
            "Network:\n  Download: {}/s\n  Upload: {}/s",
            human_bytes(network.download_bytes_per_second),
            human_bytes(network.upload_bytes_per_second)
        );
        if verbose {
            for interface in &network.interfaces {
                println!(
                    "  {}: RX {}, TX {}",
                    interface.name,
                    human_bytes(interface.receive_bytes),
                    human_bytes(interface.transmit_bytes)
                );
            }
        }
    }
}

fn human_bytes(bytes: u64) -> String {
    let units = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut index = 0;
    while value >= 1024.0 && index < units.len() - 1 {
        value /= 1024.0;
        index += 1;
    }
    if index == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", units[index])
    }
}
