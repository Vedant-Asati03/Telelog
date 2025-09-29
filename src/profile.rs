//! Performance profiling utilities

use crate::{level::LogLevel, logger::Logger};
use std::time::Instant;

/// A guard that automatically logs performance metrics when dropped
pub struct ProfileGuard {
    operation: String,
    start_time: Instant,
    logger: Logger,
}

impl ProfileGuard {
    /// Create a new profile guard
    pub fn new(operation: &str, logger: Logger) -> Self {
        let start_time = Instant::now();

        // Log start of operation
        logger.debug(&format!("Started operation: {}", operation));

        Self {
            operation: operation.to_string(),
            start_time,
            logger,
        }
    }

    /// Get the elapsed time since the guard was created
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Get the operation name
    pub fn operation(&self) -> &str {
        &self.operation
    }
}

impl Drop for ProfileGuard {
    fn drop(&mut self) {
        let elapsed = self.start_time.elapsed();
        let elapsed_ms = elapsed.as_millis();

        // Determine log level based on duration
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

        // Log with structured data
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

/// A profiler that can measure multiple operations
pub struct Profiler {
    operations: Vec<(String, Instant)>,
}

impl Profiler {
    /// Create a new profiler
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    /// Start timing an operation
    pub fn start(&mut self, operation: &str) {
        self.operations
            .push((operation.to_string(), Instant::now()));
    }

    /// End timing the most recent operation and return the duration
    pub fn end(&mut self) -> Option<(String, std::time::Duration)> {
        self.operations
            .pop()
            .map(|(op, start)| (op, start.elapsed()))
    }

    /// Get timing for a specific operation (if it's currently being timed)
    pub fn get_timing(&self, operation: &str) -> Option<std::time::Duration> {
        self.operations
            .iter()
            .rev()
            .find(|(op, _)| op == operation)
            .map(|(_, start)| start.elapsed())
    }

    /// Clear all operations
    pub fn clear(&mut self) {
        self.operations.clear();
    }

    /// Get the number of active operations
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
