use serde::{Deserialize, Serialize};
use std::ffi::CStr;
use std::ptr;
use std::time::Instant;

#[derive(Debug, Clone, Copy, Default)]
pub struct NetworkCounters {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkTelemetry {
    pub rx_bytes_per_sec: f64,
    pub tx_bytes_per_sec: f64,
    pub total_rx_bytes: u64,
    pub total_tx_bytes: u64,
    pub active_interfaces: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NetworkSampler {
    previous: Option<(NetworkCounters, Instant)>,
}

impl Default for NetworkSampler {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkSampler {
    pub fn new() -> Self {
        let previous = read_network_counters()
            .ok()
            .map(|(counters, _)| (counters, Instant::now()));
        Self { previous }
    }

    pub fn sample(&mut self) -> Result<NetworkTelemetry, String> {
        let now = Instant::now();
        let (counters, active_interfaces) = read_network_counters()?;
        let prev = self.previous.replace((counters, now));

        let (rx_rate, tx_rate) = match prev {
            Some((prev_counters, prev_time)) => {
                let elapsed = now.duration_since(prev_time).as_secs_f64();
                compute_rates(prev_counters, counters, elapsed)
            }
            None => (0.0, 0.0),
        };

        Ok(NetworkTelemetry {
            rx_bytes_per_sec: rx_rate,
            tx_bytes_per_sec: tx_rate,
            total_rx_bytes: counters.rx_bytes,
            total_tx_bytes: counters.tx_bytes,
            active_interfaces,
        })
    }
}

pub fn compute_rates(prev: NetworkCounters, curr: NetworkCounters, elapsed_secs: f64) -> (f64, f64) {
    if elapsed_secs <= 0.0 {
        return (0.0, 0.0);
    }
    let rx_delta = curr.rx_bytes.saturating_sub(prev.rx_bytes) as f64;
    let tx_delta = curr.tx_bytes.saturating_sub(prev.tx_bytes) as f64;
    (rx_delta / elapsed_secs, tx_delta / elapsed_secs)
}

fn is_primary_network_interface(name: &str) -> bool {
    name.starts_with("en") || name.starts_with("pdp_ip") || name.starts_with("bond")
}

fn read_network_counters() -> Result<(NetworkCounters, Vec<String>), String> {
    let mut addrs: *mut libc::ifaddrs = ptr::null_mut();
    let ret = unsafe { libc::getifaddrs(&mut addrs) };
    if ret != 0 || addrs.is_null() {
        return Err(format!("getifaddrs failed with code {ret}"));
    }

    let mut primary = NetworkCounters::default();
    let mut primary_ifaces = Vec::new();
    let mut fallback = NetworkCounters::default();
    let mut fallback_ifaces = Vec::new();

    unsafe {
        let mut cursor = addrs;
        while !cursor.is_null() {
            let ifa = &*cursor;
            if !ifa.ifa_addr.is_null() && !ifa.ifa_data.is_null() && !ifa.ifa_name.is_null() {
                let family = (*ifa.ifa_addr).sa_family as i32;
                let flags = ifa.ifa_flags as i32;
                let is_up = (flags & libc::IFF_UP) != 0;
                let is_running = (flags & libc::IFF_RUNNING) != 0;
                let is_loopback = (flags & libc::IFF_LOOPBACK) != 0;

                if family == libc::AF_LINK && is_up && is_running && !is_loopback {
                    let name = CStr::from_ptr(ifa.ifa_name).to_string_lossy().into_owned();
                    let data = &*(ifa.ifa_data as *const libc::if_data);
                    let ibytes = data.ifi_ibytes as u64;
                    let obytes = data.ifi_obytes as u64;

                    if is_primary_network_interface(&name) {
                        primary.rx_bytes = primary.rx_bytes.saturating_add(ibytes);
                        primary.tx_bytes = primary.tx_bytes.saturating_add(obytes);
                        if (ibytes > 0 || obytes > 0) && !primary_ifaces.contains(&name) {
                            primary_ifaces.push(name);
                        }
                    } else if !name.starts_with("awdl")
                        && !name.starts_with("llw")
                        && !name.starts_with("anpi")
                        && !name.starts_with("ap")
                    {
                        fallback.rx_bytes = fallback.rx_bytes.saturating_add(ibytes);
                        fallback.tx_bytes = fallback.tx_bytes.saturating_add(obytes);
                        if (ibytes > 0 || obytes > 0) && !fallback_ifaces.contains(&name) {
                            fallback_ifaces.push(name);
                        }
                    }
                }
            }
            cursor = ifa.ifa_next;
        }
        libc::freeifaddrs(addrs);
    }

    if !primary_ifaces.is_empty() || primary.rx_bytes > 0 || primary.tx_bytes > 0 {
        Ok((primary, primary_ifaces))
    } else {
        Ok((fallback, fallback_ifaces))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_network_rates() {
        let prev = NetworkCounters { rx_bytes: 1_000, tx_bytes: 500 };
        let curr = NetworkCounters { rx_bytes: 3_000, tx_bytes: 1_500 };
        let (rx, tx) = compute_rates(prev, curr, 2.0);
        assert!((rx - 1_000.0).abs() < 0.01);
        assert!((tx - 500.0).abs() < 0.01);
    }

    #[test]
    fn reads_live_macos_network() {
        let mut sampler = NetworkSampler::new();
        let net = sampler.sample().expect("live network");
        assert!(net.rx_bytes_per_sec >= 0.0);
        assert!(net.tx_bytes_per_sec >= 0.0);
    }
}
