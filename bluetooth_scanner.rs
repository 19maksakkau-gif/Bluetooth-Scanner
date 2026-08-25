// bluetooth_scanner.rs
use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use std::time::Duration;
use tokio::time;
use serde::{Serialize, Deserialize};
use std::fs;
use std::env;
use regex::Regex;
use chrono::Utc;

#[derive(Serialize, Deserialize, Clone)]
struct DeviceInfo {
    address: String,
    name: String,
    rssi: i16,
    timestamp: String,
    services: Vec<String>,
}

struct Scanner {
    timeout: u64,
    filter_regex: Option<Regex>,
    history_file: Option<String>,
    devices: Vec<DeviceInfo>,
    history: Vec<DeviceInfo>,
}

impl Scanner {
    fn new(timeout: u64, filter: Option<&str>, history: Option<&str>) -> Self {
        let re = filter.map(|f| Regex::new(f).unwrap());
        let mut s = Scanner {
            timeout,
            filter_regex: re,
            history_file: history.map(String::from),
            devices: Vec::new(),
            history: Vec::new(),
        };
        s.load_history();
        s
    }

    fn load_history(&mut self) {
        if let Some(ref file) = self.history_file {
            if let Ok(data) = fs::read_to_string(file) {
                if let Ok(h) = serde_json::from_str(&data) {
                    self.history = h;
                }
            }
        }
    }

    fn save_history(&self) {
        if let Some(ref file) = self.history_file {
            if let Ok(data) = serde_json::to_string_pretty(&self.history) {
                let _ = fs::write(file, data);
            }
        }
    }

    fn matches_filter(&self, name: &str) -> bool {
        if let Some(ref re) = self.filter_regex {
            re.is_match(name)
        } else {
            true
        }
    }

    async fn scan(&mut self) {
        println!("Scanning for {} seconds...", self.timeout);
        let manager = Manager::new().await.unwrap();
        let adapter = manager.adapters().await.unwrap().into_iter().next().unwrap();
        adapter.start_scan(ScanFilter::default()).await.unwrap();

        let start = std::time::Instant::now();
        while start.elapsed().as_secs() < self.timeout {
            if let Ok(peripherals) = adapter.peripherals().await {
                for p in peripherals {
                    if let Some(prop) = p.properties().await.unwrap() {
                        let name = prop.local_name.unwrap_or_else(|| "Unknown".to_string());
                        if self.matches_filter(&name) {
                            let info = DeviceInfo {
                                address: p.address().to_string(),
                                name,
                                rssi: prop.rssi.unwrap_or(0),
                                timestamp: Utc::now().to_rfc3339(),
                                services: prop.services.iter().map(|s| s.to_string()).collect(),
                            };
                            // Avoid duplicates (simplistic)
                            if !self.devices.iter().any(|d| d.address == info.address) {
                                self.devices.push(info);
                            }
                        }
                    }
                }
            }
            time::sleep(Duration::from_millis(500)).await;
        }
        adapter.stop_scan().await.unwrap();

        println!("Found {} devices:", self.devices.len());
        for d in &self.devices {
            println!("  {} ({}) RSSI: {} dBm", d.name, d.address, d.rssi);
        }
        if self.history_file.is_some() {
            self.history.extend(self.devices.clone());
            self.save_history();
            println!("History updated.");
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let mut timeout = 10;
    let mut filter = None;
    let mut history = None;
    let mut export = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--timeout" => { timeout = args[i+1].parse().unwrap(); i += 2; }
            "--filter" => { filter = Some(args[i+1].as_str()); i += 2; }
            "--history" => { history = Some(args[i+1].as_str()); i += 2; }
            "--export" => { export = Some(args[i+1].as_str()); i += 2; }
            _ => i += 1,
        }
    }

    let mut scanner = Scanner::new(timeout, filter, history);
    scanner.scan().await;

    if let (Some(exp), Some(hist)) = (export, scanner.history_file) {
        // Export CSV
        let mut csv = "timestamp,name,address,rssi,services\n".to_string();
        for e in &scanner.history {
            csv.push_str(&format!("{},{},{},{},\"{}\"\n",
                e.timestamp, e.name, e.address, e.rssi, e.services.join(";")));
        }
        let _ = fs::write(exp, csv);
        println!("Exported to {}", exp);
    }
}
