//! # Telelog - High-Performance Structured Logging
//!
//! Telelog is a high-performance logging library for Rust that provides:
//! - Structured JSON-first logging
//! - Performance profiling and monitoring
//! - Cross-language bindings support
//! - System resource monitoring
//! - Context management and decorators
//!
//! ## Quick Start
//!
//! ```rust
//! use telelog::{Logger, LogLevel};
//!
//! let logger = Logger::new("my_app");
//! logger.info("Application started");
//!
//! // With structured data
//! logger.info_with("User logged in", &[
//!     ("user_id", "12345"),
//!     ("session_id", "abcdef"),
//! ]);
//!
//! // Performance profiling
//! let _guard = logger.profile("expensive_operation");
//! // Your expensive operation here
//! ```
//!
//! ## Features
//!
//! - **Zero-allocation logging** in hot paths
//! - **Structured logging** with JSON output
//! - **Performance profiling** with minimal overhead
//! - **Rich console output** with colors and formatting
//! - **System monitoring** integration
//! - **Thread-safe** and **async-ready**

pub mod component;
pub mod config;
pub mod console;
pub mod context;
pub mod format;
pub mod level;
pub mod logger;
pub mod output;
pub mod profile;
pub mod visualization;

#[cfg(feature = "async")]
pub mod async_output;

#[cfg(feature = "system-monitor")]
pub mod monitor;

// Re-export main types
pub use component::{
    Component, ComponentGuard, ComponentMetadata, ComponentStatus, ComponentTracker,
};
pub use config::Config;
pub use context::{Context, ContextGuard};
pub use level::LogLevel;
pub use logger::Logger;
pub use output::{BufferedOutput, LogMessage};
pub use profile::ProfileGuard;
pub use visualization::{ChartConfig, ChartType, Direction, MermaidGenerator};

#[cfg(feature = "async")]
pub use async_output::AsyncOutput;

#[cfg(feature = "system-monitor")]
pub use monitor::SystemMonitor;

// Language bindings
#[cfg(feature = "python")]
pub mod python;

/// Current version of the telelog library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize telelog with default configuration
///
/// This is a convenience function that sets up a logger with sensible defaults.
/// For more control, use `Logger::new()` directly.
///
/// # Example
///
/// ```rust
/// use telelog;
///
/// let logger = telelog::init("my_app");
/// logger.info("Hello, telelog!");
/// ```
pub fn init(name: &str) -> Logger {
    Logger::new(name)
}

/// Initialize telelog with custom configuration
///
/// # Example
///
/// ```rust
/// use telelog::{init_with_config, Config};
///
/// let config = Config::new()
///     .with_console_output(true)
///     .with_json_format(true);
///
/// let logger = init_with_config("my_app", config);
/// logger.info("Hello, telelog!");
/// ```
pub fn init_with_config(name: &str, config: Config) -> Logger {
    Logger::with_config(name, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let logger = init("test");
        assert_eq!(logger.name(), "test");
    }

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
