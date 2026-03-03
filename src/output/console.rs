//! Console output destination with optional ANSI color support.

use crate::output::{LogRecord, OutputDestination};
use std::io::{self, Write};

/// Writes log records to stdout, optionally with ANSI color codes per log level.
pub struct ConsoleOutput {
    colored: bool,
}

impl ConsoleOutput {
    /// Creates a new console output.
    ///
    /// # Arguments
    ///
    /// * `colored` - Enable ANSI color codes for log levels (requires `console` feature)
    pub fn new(colored: bool) -> Self {
        Self { colored }
    }
}

impl OutputDestination for ConsoleOutput {
    fn write(&self, record: &LogRecord<'_>) -> io::Result<()> {
        if self.colored {
            #[cfg(feature = "console")]
            {
                println!(
                    "{}{} [{}] {}: {}\x1b[0m",
                    record.level.color(),
                    record.timestamp,
                    record.level,
                    record.logger,
                    record.message
                );
            }
            #[cfg(not(feature = "console"))]
            {
                println!(
                    "{} [{}] {}: {}",
                    record.timestamp, record.level, record.logger, record.message
                );
            }
        } else {
            println!(
                "{} [{}] {}: {}",
                record.timestamp, record.level, record.logger, record.message
            );
        }

        Ok(())
    }

    fn flush(&self) -> io::Result<()> {
        io::stdout().flush()
    }

    fn write_bytes(&self, bytes: &[u8]) -> io::Result<()> {
        io::stdout().write_all(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::LogLevel;
    use std::collections::HashMap;

    #[test]
    fn test_console_output() {
        let output = ConsoleOutput::new(false);
        assert!(output
            .write(&LogRecord {
                timestamp: "2025-09-07T10:30:00Z",
                level: LogLevel::Info,
                logger: "test",
                message: "Test message",
                context: &HashMap::new(),
                data: None,
            })
            .is_ok());
        assert!(output.flush().is_ok());
    }
}
