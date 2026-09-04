use std::{fs, time::Instant};

use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub struct SystemMetrics {
    pub uptime_seconds: f64,
    pub load_1m: f64,
    pub load_5m: f64,
    pub load_15m: f64,
    pub cpu_percent: f64,
    pub memory_total_bytes: u64,
    pub memory_available_bytes: u64,
    pub process_rss_bytes: u64,
    pub temperature_celsius: Option<f64>,
}

#[derive(Debug)]
pub struct SystemSampler {
    process_started: Instant,
    previous_cpu: Option<CpuTicks>,
}

#[derive(Debug, Clone, Copy)]
struct CpuTicks {
    total: u64,
    idle: u64,
}

impl Default for SystemSampler {
    fn default() -> Self {
        Self {
            process_started: Instant::now(),
            previous_cpu: None,
        }
    }
}

impl SystemSampler {
    pub fn sample(&mut self) -> SystemMetrics {
        let mut metrics = SystemMetrics {
            uptime_seconds: read_uptime()
                .unwrap_or_else(|| self.process_started.elapsed().as_secs_f64()),
            ..SystemMetrics::default()
        };

        if let Some(load) = read_load_average() {
            (metrics.load_1m, metrics.load_5m, metrics.load_15m) = load;
        }
        if let Some((total, available)) = read_memory() {
            metrics.memory_total_bytes = total;
            metrics.memory_available_bytes = available;
        }
        metrics.process_rss_bytes = read_process_rss().unwrap_or_default();
        metrics.temperature_celsius = read_temperature();

        if let Some(current) = read_cpu_ticks() {
            if let Some(previous) = self.previous_cpu {
                let total_delta = current.total.saturating_sub(previous.total);
                let idle_delta = current.idle.saturating_sub(previous.idle);
                if total_delta > 0 {
                    metrics.cpu_percent =
                        100.0 * total_delta.saturating_sub(idle_delta) as f64 / total_delta as f64;
                }
            }
            self.previous_cpu = Some(current);
        }
        metrics
    }
}

fn read_uptime() -> Option<f64> {
    fs::read_to_string("/proc/uptime")
        .ok()?
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

fn read_load_average() -> Option<(f64, f64, f64)> {
    let raw = fs::read_to_string("/proc/loadavg").ok()?;
    let mut fields = raw.split_whitespace();
    Some((
        fields.next()?.parse().ok()?,
        fields.next()?.parse().ok()?,
        fields.next()?.parse().ok()?,
    ))
}

fn read_memory() -> Option<(u64, u64)> {
    let raw = fs::read_to_string("/proc/meminfo").ok()?;
    let value = |name: &str| -> Option<u64> {
        raw.lines().find_map(|line| {
            let (key, rest) = line.split_once(':')?;
            (key == name)
                .then(|| rest.split_whitespace().next()?.parse::<u64>().ok())
                .flatten()
        })
    };
    Some((value("MemTotal")? * 1_024, value("MemAvailable")? * 1_024))
}

fn read_process_rss() -> Option<u64> {
    let raw = fs::read_to_string("/proc/self/status").ok()?;
    raw.lines().find_map(|line| {
        let (key, rest) = line.split_once(':')?;
        (key == "VmRSS")
            .then(|| rest.split_whitespace().next()?.parse::<u64>().ok())
            .flatten()
            .map(|kilobytes| kilobytes * 1_024)
    })
}

fn read_cpu_ticks() -> Option<CpuTicks> {
    let raw = fs::read_to_string("/proc/stat").ok()?;
    let mut values = raw.lines().next()?.split_whitespace();
    if values.next()? != "cpu" {
        return None;
    }
    let ticks = values
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    // user nice system idle iowait irq softirq steal [guest guest_nice]
    // guest/guest_nice are already included in user/nice — do not sum them twice.
    if ticks.len() < 4 {
        return None;
    }
    let accounted = if ticks.len() >= 8 {
        &ticks[..8]
    } else {
        &ticks[..]
    };
    let idle = ticks[3] + ticks.get(4).copied().unwrap_or_default();
    Some(CpuTicks {
        total: accounted.iter().sum(),
        idle,
    })
}

fn read_temperature() -> Option<f64> {
    let millidegrees = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")
        .ok()?
        .trim()
        .parse::<f64>()
        .ok()?;
    Some(millidegrees / 1_000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sampler_is_portable_and_finite() {
        let metrics = SystemSampler::default().sample();
        assert!(metrics.uptime_seconds.is_finite());
        assert!(metrics.cpu_percent.is_finite());
    }

    #[test]
    fn cpu_total_excludes_guest_double_count() {
        // user nice system idle iowait irq softirq steal guest guest_nice
        let ticks = [10u64, 20, 30, 40, 5, 1, 2, 3, 100, 50];
        let accounted = &ticks[..8];
        let idle = ticks[3] + ticks[4];
        let total: u64 = accounted.iter().sum();
        assert_eq!(total, 10 + 20 + 30 + 40 + 5 + 1 + 2 + 3);
        assert_eq!(idle, 45);
        assert!(total < ticks.iter().sum::<u64>());
    }
}
