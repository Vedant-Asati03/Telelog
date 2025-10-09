//! Performance profiling utilities with automatic timing.
//!
//! Provides RAII guards and profiler utilities for measuring operation duration
//! and automatically logging performance metrics.
//!
//! # Examples
//!
//! ```
//! use telelog::Logger;
//!
//! let logger = Logger::new("app");
//! {
//!     let _guard = logger.profile("database_query");
//!     // Operation is timed automatically
//! } // Duration logged when guard drops
//! ```

use crate::{level::LogLevel, logger::Logger};
use std::time::Instant;

/// RAII guard that profiles an operation and logs its duration.
///
/// The duration is automatically logged when the guard is dropped, with
/// the log level determined by execution time:
/// - < 100ms: Debug level
/// - 100-1000ms: Info level  
/// - > 1000ms: Warning level (slow operation)
pub struct ProfileGuard {
    operation: String,
    start_time: Instant,
    logger: Logger,
}

impl ProfileGuard {
    /// Creates a new profiling guard for the specified operation.
    pub fn new(operation: &str, logger: Logger) -> Self {
        let start_time = Instant::now();

        logger.debug(&format!("Started operation: {}", operation));

        Self {
            operation: operation.to_string(),
            start_time,
            logger,
        }
    }

    /// Returns the elapsed time since the guard was created.
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Returns the operation name being profiled.
    pub fn operation(&self) -> &str {
        &self.operation
    }
}

impl Drop for ProfileGuard {
    fn drop(&mut self) {
        let elapsed = self.start_time.elapsed();
        let elapsed_ms = elapsed.as_millis();

        let (level, message) = if elapsed_ms > 1000 {
            (
                LogLevel::Warning,
                format!(
                    "Slow operation completed: {} ({}ms)",
                    self.operation, elapsed_ms
                ),
            )
        } else if elapsed_ms > 100 {
            (
                LogLevel::Info,
                format!("Operation completed: {} ({}ms)", self.operation, elapsed_ms),
            )
        } else {
            (
                LogLevel::Debug,
                format!(
                    "Fast operation completed: {} ({}ms)",
                    self.operation, elapsed_ms
                ),
            )
        };

        match level {
            LogLevel::Debug => self.logger.debug_with(
                &message,
                &[
                    ("operation", &self.operation),
                    ("duration_ms", &elapsed_ms.to_string()),
                    ("duration_us", &elapsed.as_micros().to_string()),
                ],
            ),
            LogLevel::Info => self.logger.info_with(
                &message,
                &[
                    ("operation", &self.operation),
                    ("duration_ms", &elapsed_ms.to_string()),
                    ("duration_us", &elapsed.as_micros().to_string()),
                ],
            ),
            LogLevel::Warning => self.logger.warning_with(
                &message,
                &[
                    ("operation", &self.operation),
                    ("duration_ms", &elapsed_ms.to_string()),
                    ("duration_us", &elapsed.as_micros().to_string()),
                ],
            ),
            _ => unreachable!(),
        }
    }
}

/// Manual profiler for tracking multiple operations.
///
/// Unlike [`ProfileGuard`], this requires explicit start/end calls and
/// doesn't automatically log results.
pub struct Profiler {
    operations: Vec<(String, Instant)>,
}

impl Profiler {
    /// Creates a new empty profiler.
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    /// Starts timing an operation.
    pub fn start(&mut self, operation: &str) {
        self.operations
            .push((operation.to_string(), Instant::now()));
    }

    /// Ends the most recent operation and returns its name and duration.
    pub fn end(&mut self) -> Option<(String, std::time::Duration)> {
        self.operations
            .pop()
            .map(|(op, start)| (op, start.elapsed()))
    }

    /// Gets the elapsed time for a specific operation (most recent if multiple).
    pub fn get_timing(&self, operation: &str) -> Option<std::time::Duration> {
        self.operations
            .iter()
            .rev()
            .find(|(op, _)| op == operation)
            .map(|(_, start)| start.elapsed())
    }

    /// Clears all tracked operations.
    pub fn clear(&mut self) {
        self.operations.clear();
    }

    /// Returns the number of active (un-ended) operations.
    pub fn active_count(&self) -> usize {
        self.operations.len()
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_profile_guard() {
        let logger = Logger::with_config("test", Config::development());

        {
            let _guard = ProfileGuard::new("test_operation", logger.clone());
            thread::sleep(Duration::from_millis(10));
        } // Guard should log the duration when dropped
    }

    #[test]
    fn test_profiler() {
        let mut profiler = Profiler::new();

        assert_eq!(profiler.active_count(), 0);

        profiler.start("operation1");
        assert_eq!(profiler.active_count(), 1);

        thread::sleep(Duration::from_millis(10));

        let (op, duration) = profiler.end().unwrap();
        assert_eq!(op, "operation1");
        assert!(duration.as_millis() >= 10);
        assert_eq!(profiler.active_count(), 0);
    }

    #[test]
    fn test_nested_operations() {
        let mut profiler = Profiler::new();

        profiler.start("outer");
        profiler.start("inner");

        assert_eq!(profiler.active_count(), 2);

        let (op, _) = profiler.end().unwrap();
        assert_eq!(op, "inner");
        assert_eq!(profiler.active_count(), 1);

        let (op, _) = profiler.end().unwrap();
        assert_eq!(op, "outer");
        assert_eq!(profiler.active_count(), 0);
    }

    #[test]
    fn test_get_timing() {
        let mut profiler = Profiler::new();

        profiler.start("test_op");
        thread::sleep(Duration::from_millis(10));

        let timing = profiler.get_timing("test_op").unwrap();
        assert!(timing.as_millis() >= 10);

        assert!(profiler.get_timing("nonexistent").is_none());
    }
}
