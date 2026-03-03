//! Multi-output router that fans log records out to multiple destinations.

use crate::output::{LogRecord, OutputDestination};
use std::io;

/// Routes each log record to all registered output destinations.
///
/// Errors from individual outputs are printed to stderr but do not prevent
/// writing to the remaining destinations.
pub struct MultiOutput {
    outputs: Vec<Box<dyn OutputDestination>>,
}

impl MultiOutput {
    /// Creates an empty multi-output router.
    pub fn new() -> Self {
        Self {
            outputs: Vec::new(),
        }
    }

    /// Appends an output destination to the router.
    pub fn add_output(mut self, output: Box<dyn OutputDestination>) -> Self {
        self.outputs.push(output);
        self
    }
}

impl Default for MultiOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputDestination for MultiOutput {
    fn write(&self, record: &LogRecord<'_>) -> io::Result<()> {
        for output in &self.outputs {
            if let Err(e) = output.write(record) {
                eprintln!("Output error: {}", e);
            }
        }
        Ok(())
    }

    fn flush(&self) -> io::Result<()> {
        for output in &self.outputs {
            if let Err(e) = output.flush() {
                eprintln!("Flush error: {}", e);
            }
        }
        Ok(())
    }

    fn write_bytes(&self, bytes: &[u8]) -> io::Result<()> {
        for output in &self.outputs {
            if let Err(e) = output.write_bytes(bytes) {
                eprintln!("Output error: {}", e);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::LogLevel;
    use crate::output::{console::ConsoleOutput, file::FileOutput};
    use std::collections::HashMap;
    use tempfile::NamedTempFile;

    #[test]
    fn test_multi_output() {
        let temp_file = NamedTempFile::new().unwrap();
        let multi = MultiOutput::new()
            .add_output(Box::new(FileOutput::new(temp_file.path(), false).unwrap()))
            .add_output(Box::new(ConsoleOutput::new(false)));

        assert!(multi
            .write(&LogRecord {
                timestamp: "2025-09-07T10:30:00Z",
                level: LogLevel::Info,
                logger: "test",
                message: "Test message",
                context: &HashMap::new(),
                data: None,
            })
            .is_ok());
        assert!(multi.flush().is_ok());
    }
}
