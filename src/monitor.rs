//! System resource monitoring utilities.
//!
//! Provides the [`SystemMonitor`] for tracking CPU and memory usage.
//! Requires the `system-monitor` feature to be enabled.

#[cfg(feature = "system-monitor")]
use sysinfo::System;

/// System resource monitor for tracking CPU and memory usage.
///
/// Uses the `sysinfo` crate to gather system metrics. Requires periodic
/// calls to [`refresh`](Self::refresh) to update measurements.
#[derive(Debug)]
pub struct SystemMonitor {
    system: System,
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemMonitor {
    /// Creates a new system monitor and initializes system information.
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }

    /// Refreshes all system information for updated metrics.
    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }

    /// Returns the current memory usage as a percentage (0-100).
    pub fn memory_usage(&self) -> f64 {
        let total = self.system.total_memory();
        let used = self.system.used_memory();
        if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Returns the current global CPU usage percentage.
    pub fn cpu_usage(&mut self) -> f64 {
        self.system.refresh_cpu_all();
        self.system.global_cpu_usage() as f64
    }

    /// Returns the memory usage of the current process in bytes.
    pub fn process_memory(&self) -> Option<u64> {
        let pid = sysinfo::get_current_pid().ok()?;
        Some(self.system.process(pid)?.memory())
    }
}

/// Stub implementation when `system-monitor` feature is disabled.
#[cfg(not(feature = "system-monitor"))]
pub struct SystemMonitor;

#[cfg(not(feature = "system-monitor"))]
impl SystemMonitor {
    /// Creates a stub monitor (no-op when feature is disabled).
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_monitor_creation() {
        let _monitor = SystemMonitor::new();
    }

    #[cfg(feature = "system-monitor")]
    #[test]
    fn test_system_metrics() {
        let mut monitor = SystemMonitor::new();
        monitor.refresh();

        let memory = monitor.memory_usage();
        assert!(memory >= 0.0 && memory <= 100.0);

        let cpu = monitor.cpu_usage();
        assert!(cpu >= 0.0);
    }
}
