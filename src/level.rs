//! Log level definitions and utilities for filtering and formatting.
//!
//! Provides the [`LogLevel`] enum for categorizing log messages by severity,
//! with support for filtering, ordering, and color-coded output.
//!
//! # Examples
//!
//! ```
//! use telelog::LogLevel;
//!
//! let level = LogLevel::Info;
//! assert_eq!(level.as_str(), "INFO");
//! assert!(level.should_log(LogLevel::Debug));
//! assert!(!LogLevel::Debug.should_log(LogLevel::Info));
//!
//! // Parse from string
//! let level: LogLevel = "warning".parse().unwrap();
//! assert_eq!(level, LogLevel::Warning);
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

/// Log severity levels in ascending order of importance.
///
/// Levels can be compared and ordered: Debug < Info < Warning < Error < Critical
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warning = 2,
    Error = 3,
    Critical = 4,
}

impl LogLevel {
    /// Returns the string representation of the log level.
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
            LogLevel::Critical => "CRITICAL",
        }
    }

    /// Returns the ANSI color code for terminal output (requires `console` feature).
    #[cfg(feature = "console")]
    pub fn color(&self) -> &'static str {
        match self {
            LogLevel::Debug => "\x1b[36m",
            LogLevel::Info => "\x1b[32m",
            LogLevel::Warning => "\x1b[33m",
            LogLevel::Error => "\x1b[31m",
            LogLevel::Critical => "\x1b[91m",
        }
    }

    /// Determines if this log level should be logged given the minimum level.
    ///
    /// Returns `true` if this level is equal to or higher than the minimum level.
    ///
    /// # Examples
    ///
    /// ```
    /// use telelog::LogLevel;
    ///
    /// assert!(LogLevel::Error.should_log(LogLevel::Info));
    /// assert!(!LogLevel::Debug.should_log(LogLevel::Info));
    /// ```
    pub fn should_log(&self, min_level: LogLevel) -> bool {
        *self >= min_level
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DEBUG" => Ok(LogLevel::Debug),
            "INFO" => Ok(LogLevel::Info),
            "WARNING" | "WARN" => Ok(LogLevel::Warning),
            "ERROR" => Ok(LogLevel::Error),
            "CRITICAL" | "CRIT" => Ok(LogLevel::Critical),
            _ => Err(format!("Invalid log level: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warning);
        assert!(LogLevel::Warning < LogLevel::Error);
        assert!(LogLevel::Error < LogLevel::Critical);
    }

    #[test]
    fn test_should_log() {
        assert!(LogLevel::Error.should_log(LogLevel::Info));
        assert!(!LogLevel::Debug.should_log(LogLevel::Info));
        assert!(LogLevel::Info.should_log(LogLevel::Info));
    }

    #[test]
    fn test_from_str() {
        assert_eq!("INFO".parse::<LogLevel>().unwrap(), LogLevel::Info);
        assert_eq!("warn".parse::<LogLevel>().unwrap(), LogLevel::Warning);
        assert!("invalid".parse::<LogLevel>().is_err());
    }
}
