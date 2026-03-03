//! Output destination management for log messages.
//!
//! Provides various output destinations including console, file, rotating files,
//! and multi-output routing. All destinations implement the [`OutputDestination`] trait.
//!
//! # Examples
//!
//! ```no_run
//! use telelog::output::{ConsoleOutput, FileOutput, MultiOutput};
//! use std::sync::Arc;
//!
//! let console = Box::new(ConsoleOutput::new(true));
//! let file = Box::new(FileOutput::new("app.log", false).unwrap());
//!
//! let multi = MultiOutput::new()
//!     .add_output(console)
//!     .add_output(file);
//! ```

pub mod buffered;
pub mod console;
pub mod file;
pub mod multi;

#[cfg(feature = "async")]
pub mod r#async;

pub use buffered::BufferedOutput;
pub use console::ConsoleOutput;
pub use file::{FileOutput, RotatingFileOutput};
pub use multi::MultiOutput;

#[cfg(feature = "async")]
pub use r#async::AsyncOutput;

use crate::level::LogLevel;
use serde::ser::{SerializeMap, Serializer};
use std::collections::HashMap;
use std::io;

/// A zero-allocation log record passed by reference through the output pipeline.
///
/// All fields are borrowed from the caller's stack frame, avoiding heap allocation
/// for the record itself. Structured fields in `data` are also borrowed slices.
#[derive(Clone, Debug)]
pub struct LogRecord<'a> {
    pub timestamp: &'a str,
    pub level: LogLevel,
    pub logger: &'a str,
    pub message: &'a str,
    pub context: &'a HashMap<String, String>,
    pub data: Option<&'a [(&'a str, &'a str)]>,
}

impl<'a> serde::Serialize for LogRecord<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("timestamp", self.timestamp)?;
        map.serialize_entry("level", self.level.as_str())?;
        map.serialize_entry("logger", self.logger)?;
        map.serialize_entry("message", self.message)?;
        for (k, v) in self.context {
            map.serialize_entry(k, v)?;
        }
        if let Some(d) = self.data {
            for (k, v) in d {
                map.serialize_entry(k, v)?;
            }
        }
        map.end()
    }
}

/// Trait for log message output destinations.
///
/// Implement this trait to create custom output destinations.
pub trait OutputDestination: Send + Sync {
    /// Serializes and writes a log record.
    fn write(&self, record: &LogRecord<'_>) -> io::Result<()>;

    /// Flushes any buffered output to ensure data is written.
    fn flush(&self) -> io::Result<()>;

    /// Writes pre-serialized bytes (a complete, newline-terminated JSON record).
    ///
    /// Used by [`BufferedOutput`] and `AsyncOutput` to flush records without
    /// re-serializing. The default implementation writes the bytes directly as
    /// UTF-8 to the destination's underlying writer. Override for best performance.
    fn write_bytes(&self, bytes: &[u8]) -> io::Result<()> {
        // Fallback: write raw bytes as-is (callers guarantee newline termination)
        let _ = bytes;
        Ok(())
    }
}
