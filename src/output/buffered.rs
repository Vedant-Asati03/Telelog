//! Buffered output that accumulates log records in memory before flushing.
//!
//! `BufferedOutput` stores owned copies of each [`LogRecord`]'s fields and
//! replays them through the underlying destination's `write` method on flush.
//! This preserves the destination's configured format (plain-text or JSON)
//! regardless of how the buffer is used.

use crate::level::LogLevel;
use crate::output::{LogRecord, OutputDestination};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::io;
use std::sync::Arc;

/// Owned copy of a [`LogRecord`] held inside the buffer.
struct OwnedRecord {
    timestamp: String,
    level: LogLevel,
    logger: String,
    message: String,
    context: HashMap<String, String>,
    data: Vec<(String, String)>,
}

impl OwnedRecord {
    fn from_record(r: &LogRecord<'_>) -> Self {
        Self {
            timestamp: r.timestamp.to_owned(),
            level: r.level,
            logger: r.logger.to_owned(),
            message: r.message.to_owned(),
            context: r.context.clone(),
            data: r
                .data
                .unwrap_or_default()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }
}

/// Accumulates log records in memory and writes them all at once on flush.
///
/// Respects the underlying destination's format (plain-text or JSON).
/// Automatically flushes when the buffer reaches its capacity.
pub struct BufferedOutput {
    destination: Arc<dyn OutputDestination>,
    buffer: Arc<Mutex<Vec<OwnedRecord>>>,
    buffer_size: usize,
}

impl BufferedOutput {
    /// Creates a new buffered output wrapping the given destination.
    ///
    /// # Arguments
    ///
    /// * `destination` - The underlying output destination
    /// * `buffer_size` - Number of records to buffer before auto-flush
    pub fn new(destination: Arc<dyn OutputDestination>, buffer_size: usize) -> Self {
        Self {
            destination,
            buffer: Arc::new(Mutex::new(Vec::with_capacity(buffer_size))),
            buffer_size,
        }
    }

    /// Flushes all buffered records to the underlying destination.
    pub fn flush_buffer(&self) -> io::Result<()> {
        let mut buffer = self.buffer.lock();
        for owned in buffer.drain(..) {
            let data_refs: Vec<(&str, &str)> = owned
                .data
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            let record = LogRecord {
                timestamp: &owned.timestamp,
                level: owned.level,
                logger: &owned.logger,
                message: &owned.message,
                context: &owned.context,
                data: if data_refs.is_empty() {
                    None
                } else {
                    Some(&data_refs)
                },
            };
            self.destination.write(&record)?;
        }
        self.destination.flush()?;
        Ok(())
    }

    /// Returns the number of records currently in the buffer.
    pub fn buffer_len(&self) -> usize {
        self.buffer.lock().len()
    }
}

impl OutputDestination for BufferedOutput {
    fn write(&self, record: &LogRecord<'_>) -> io::Result<()> {
        let owned = OwnedRecord::from_record(record);
        let mut buffer = self.buffer.lock();
        buffer.push(owned);

        if buffer.len() >= self.buffer_size {
            drop(buffer);
            self.flush_buffer()?;
        }

        Ok(())
    }

    fn flush(&self) -> io::Result<()> {
        self.flush_buffer()
    }
}

impl Drop for BufferedOutput {
    fn drop(&mut self) {
        if let Err(e) = self.flush_buffer() {
            eprintln!("Error flushing BufferedOutput on drop: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::LogLevel;
    use crate::output::file::FileOutput;
    use std::collections::HashMap;
    use tempfile::NamedTempFile;

    #[test]
    fn test_buffered_output() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_output = Arc::new(FileOutput::new(temp_file.path(), false).unwrap());
        let buffered = BufferedOutput::new(file_output, 3);
        let ctx = HashMap::new();

        buffered
            .write(&LogRecord {
                timestamp: "2025-09-07T10:30:00Z",
                level: LogLevel::Info,
                logger: "test",
                message: "Buffered message",
                context: &ctx,
                data: None,
            })
            .unwrap();
        assert_eq!(buffered.buffer_len(), 1);

        buffered
            .write(&LogRecord {
                timestamp: "2025-09-07T10:30:00Z",
                level: LogLevel::Warning,
                logger: "test",
                message: "Buffered message",
                context: &ctx,
                data: None,
            })
            .unwrap();
        assert_eq!(buffered.buffer_len(), 2);

        // Third write triggers auto-flush
        buffered
            .write(&LogRecord {
                timestamp: "2025-09-07T10:30:00Z",
                level: LogLevel::Error,
                logger: "test",
                message: "Buffered message",
                context: &ctx,
                data: None,
            })
            .unwrap();
        assert_eq!(buffered.buffer_len(), 0);

        buffered.flush().unwrap();
        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("Buffered message"));
    }
}
