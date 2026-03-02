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

use crate::level::LogLevel;
use parking_lot::Mutex;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Trait for log message output destinations.
///
/// Implement this trait to create custom output destinations.
use serde::ser::{SerializeMap, Serializer};

#[derive(Clone, Debug)]
pub struct LogRecord<'a> {
    pub timestamp: &'a str,
    pub level: LogLevel,
    pub logger: &'a str,
    pub message: &'a str,
    pub context: &'a HashMap<String, String>,
    pub data: Option<&'a [(&'a str, &'a str)]>,
}

impl<'a> LogRecord<'a> {
    pub fn to_hashmap(&self) -> HashMap<String, Value> {
        let mut map = HashMap::with_capacity(
            4 + self.context.len() + self.data.map(|d| d.len()).unwrap_or(0),
        );
        map.insert(
            "timestamp".to_string(),
            Value::String(self.timestamp.to_string()),
        );
        map.insert("level".to_string(), Value::String(self.level.to_string()));
        map.insert("logger".to_string(), Value::String(self.logger.to_string()));
        map.insert(
            "message".to_string(),
            Value::String(self.message.to_string()),
        );
        for (k, v) in self.context {
            map.insert(k.clone(), Value::String(v.clone()));
        }
        if let Some(data) = self.data {
            for (k, v) in data {
                map.insert(k.to_string(), Value::String(v.to_string()));
            }
        }
        map
    }
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

pub trait OutputDestination: Send + Sync {
    /// Writes a log message.
    fn write(&self, record: &LogRecord<'_>) -> io::Result<()>;

    /// Flushes any buffered output to ensure data is written.
    fn flush(&self) -> io::Result<()>;
}

/// Console output destination with optional colored output.
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
        let timestamp = record.timestamp;
        let logger = record.logger;
        let message = record.message;
        let level = record.level;

        if self.colored {
            #[cfg(feature = "console")]
            {
                println!(
                    "{}{} [{}] {}: {}\x1b[0m",
                    level.color(),
                    timestamp,
                    level,
                    logger,
                    message
                );
            }
            #[cfg(not(feature = "console"))]
            {
                println!("{} [{}] {}: {}", timestamp, level, logger, message);
            }
        } else {
            println!("{} [{}] {}: {}", timestamp, level, logger, message);
        }

        Ok(())
    }

    fn flush(&self) -> io::Result<()> {
        io::stdout().flush()
    }
}

/// File output destination with optional JSON formatting.
pub struct FileOutput {
    writer: Arc<Mutex<BufWriter<File>>>,
    path: PathBuf,
    json_format: bool,
}

impl FileOutput {
    /// Creates a new file output destination.
    ///
    /// # Arguments
    ///
    /// * `path` - File path for log output
    /// * `json_format` - If true, writes JSON; otherwise plain text
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or opened.
    pub fn new<P: AsRef<Path>>(path: P, json_format: bool) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new().create(true).append(true).open(&path)?;

        let writer = Arc::new(Mutex::new(BufWriter::new(file)));

        Ok(Self {
            writer,
            path,
            json_format,
        })
    }

    /// Returns the path to the log file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl OutputDestination for FileOutput {
    fn write(&self, record: &LogRecord<'_>) -> io::Result<()> {
        let mut writer = self.writer.lock();

        if self.json_format {
            let json = serde_json::to_string(record)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            writeln!(writer, "{}", json)?;
        } else {
            writeln!(
                writer,
                "{} [{}] {}: {}",
                record.timestamp, record.level, record.logger, record.message
            )?;
        }

        Ok(())
    }

    fn flush(&self) -> io::Result<()> {
        self.writer.lock().flush()
    }
}

/// Rotating file output that creates new files when size limit is reached.
pub struct RotatingFileOutput {
    base_path: PathBuf,
    max_size: u64,
    max_files: u32,
    current_size: Arc<Mutex<u64>>,
    current_file: Arc<Mutex<Option<BufWriter<File>>>>,
    json_format: bool,
}

impl RotatingFileOutput {
    /// Creates a new rotating file output.
    ///
    /// Files are rotated when they exceed `max_size` bytes. Old files are numbered
    /// (e.g., `app.log.1`, `app.log.2`) and the oldest is deleted when `max_files` is reached.
    ///
    /// # Arguments
    ///
    /// * `base_path` - Base path for log files
    /// * `max_size` - Maximum file size in bytes before rotation
    /// * `max_files` - Maximum number of rotated files to keep
    /// * `json_format` - If true, writes JSON; otherwise plain text
    pub fn new<P: AsRef<Path>>(
        base_path: P,
        max_size: u64,
        max_files: u32,
        json_format: bool,
    ) -> io::Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();

        if let Some(parent) = base_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(Self {
            base_path,
            max_size,
            max_files,
            current_size: Arc::new(Mutex::new(0)),
            current_file: Arc::new(Mutex::new(None)),
            json_format,
        })
    }

    fn rotate_if_needed(&self) -> io::Result<()> {
        let mut size = self.current_size.lock();

        if *size >= self.max_size {
            self.rotate_files()?;
            *size = 0;
        }

        Ok(())
    }

    fn rotate_files(&self) -> io::Result<()> {
        {
            let mut current = self.current_file.lock();
            if let Some(mut writer) = current.take() {
                writer.flush()?;
            }
        }

        for i in (1..self.max_files).rev() {
            let old_path = self.get_rotated_path(i);
            let new_path = self.get_rotated_path(i + 1);

            if old_path.exists() {
                if new_path.exists() {
                    std::fs::remove_file(&new_path)?;
                }
                std::fs::rename(&old_path, &new_path)?;
            }
        }

        if self.base_path.exists() {
            let rotated_path = self.get_rotated_path(1);
            std::fs::rename(&self.base_path, &rotated_path)?;
        }

        Ok(())
    }

    fn get_rotated_path(&self, number: u32) -> PathBuf {
        let mut path = self.base_path.clone();
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let extension = path.extension().unwrap_or_default().to_string_lossy();

        if extension.is_empty() {
            path.set_file_name(format!("{}.{}", stem, number));
        } else {
            path.set_file_name(format!("{}.{}.{}", stem, number, extension));
        }

        path
    }

    /// Ensures the current file handle is open and ready for writing.
    fn ensure_file(&self) -> io::Result<()> {
        let mut current = self.current_file.lock();

        if current.is_none() {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.base_path)?;

            let metadata = file.metadata()?;
            *self.current_size.lock() = metadata.len();

            *current = Some(BufWriter::new(file));
        }

        Ok(())
    }
}

impl OutputDestination for RotatingFileOutput {
    fn write(&self, record: &LogRecord<'_>) -> io::Result<()> {
        self.rotate_if_needed()?;
        self.ensure_file()?;

        let mut current = self.current_file.lock();
        if let Some(ref mut writer) = *current {
            let content = if self.json_format {
                let json = serde_json::to_string(record)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                format!("{}\n", json)
            } else {
                let timestamp = record.timestamp;
                let level = record.level;
                let logger = record.logger;
                let message = record.message;

                format!("{} [{}] {}: {}\n", timestamp, level, logger, message)
            };

            writer.write_all(content.as_bytes())?;
            *self.current_size.lock() += content.len() as u64;
        }

        Ok(())
    }

    fn flush(&self) -> io::Result<()> {
        let mut current = self.current_file.lock();
        if let Some(ref mut writer) = *current {
            writer.flush()?;
        }
        Ok(())
    }
}

/// Multi-output router that writes to multiple destinations.
///
/// Errors from individual outputs are logged to stderr but don't prevent
/// writing to other outputs.
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

    /// Adds an output destination to the router.
    pub fn add_output(mut self, output: Box<dyn OutputDestination>) -> Self {
        self.outputs.push(output);
        self
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
}

impl Default for MultiOutput {
    fn default() -> Self {
        Self::new()
    }
}

/// A log message with level and structured data.
#[derive(Debug, Clone)]
pub struct LogMessage {
    pub level: LogLevel,
    pub data: HashMap<String, Value>,
}

impl LogMessage {
    /// Creates a new log message.
    pub fn new(level: LogLevel, data: HashMap<String, Value>) -> Self {
        Self { level, data }
    }
}

/// Buffered output that accumulates messages before writing.
///
/// Automatically flushes when the buffer reaches its capacity or when
/// explicitly flushed.
pub struct BufferedOutput {
    destination: Arc<dyn OutputDestination>,
    buffer: Arc<Mutex<Vec<LogMessage>>>,
    buffer_size: usize,
}

impl BufferedOutput {
    /// Creates a new buffered output wrapping the given destination.
    ///
    /// # Arguments
    ///
    /// * `destination` - The underlying output destination
    /// * `buffer_size` - Number of messages to buffer before auto-flush
    pub fn new(destination: Arc<dyn OutputDestination>, buffer_size: usize) -> Self {
        Self {
            destination,
            buffer: Arc::new(Mutex::new(Vec::with_capacity(buffer_size))),
            buffer_size,
        }
    }

    /// Flushes all buffered messages to the underlying destination.
    pub fn flush_buffer(&self) -> io::Result<()> {
        let mut buffer = self.buffer.lock();

        for message in buffer.drain(..) {
            let timestamp = message
                .data
                .get("timestamp")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let logger = message
                .data
                .get("logger")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let msg = message
                .data
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            // To pass context & data correctly without allocation, we iterate over the leftovers
            let mut ctx_map = HashMap::new();
            for (k, v) in &message.data {
                if k != "timestamp" && k != "level" && k != "logger" && k != "message" {
                    if let Value::String(s) = v {
                        ctx_map.insert(k.clone(), s.clone());
                    }
                }
            }

            let record = LogRecord {
                timestamp,
                level: message.level,
                logger,
                message: msg,
                context: &ctx_map,
                data: None,
            };

            self.destination.write(&record)?;
        }

        self.destination.flush()?;
        Ok(())
    }

    /// Returns the number of messages currently in the buffer.
    pub fn buffer_len(&self) -> usize {
        self.buffer.lock().len()
    }
}

impl OutputDestination for BufferedOutput {
    fn write(&self, record: &LogRecord<'_>) -> io::Result<()> {
        let message = LogMessage::new(record.level, record.to_hashmap());
        let mut buffer = self.buffer.lock();

        buffer.push(message);

        if buffer.len() >= self.buffer_size {
            drop(buffer); // Release lock before recursive call
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
            eprintln!("Error flushing buffer on drop: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_console_output() {
        let output = ConsoleOutput::new(false);
        let mut data = HashMap::new();
        data.insert(
            "timestamp".to_string(),
            Value::String("2025-09-07T10:30:00Z".to_string()),
        );
        data.insert("level".to_string(), Value::String("INFO".to_string()));
        data.insert("logger".to_string(), Value::String("test".to_string()));
        data.insert(
            "message".to_string(),
            Value::String("Test message".to_string()),
        );

        assert!(output
            .write(&LogRecord {
                timestamp: "2025-09-07T10:30:00Z",
                level: LogLevel::Info,
                logger: "test",
                message: "Test message",
                context: &HashMap::new(),
                data: None
            })
            .is_ok());
        assert!(output.flush().is_ok());
    }

    #[test]
    fn test_file_output() {
        let temp_file = NamedTempFile::new().unwrap();
        let output = FileOutput::new(temp_file.path(), false).unwrap();

        let mut data = HashMap::new();
        data.insert(
            "timestamp".to_string(),
            Value::String("2025-09-07T10:30:00Z".to_string()),
        );
        data.insert("level".to_string(), Value::String("INFO".to_string()));
        data.insert("logger".to_string(), Value::String("test".to_string()));
        data.insert(
            "message".to_string(),
            Value::String("Test message".to_string()),
        );

        assert!(output
            .write(&LogRecord {
                timestamp: "2025-09-07T10:30:00Z",
                level: LogLevel::Info,
                logger: "test",
                message: "Test message",
                context: &HashMap::new(),
                data: None
            })
            .is_ok());
        assert!(output.flush().is_ok());

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("Test message"));
    }

    #[test]
    fn test_multi_output() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_output = Box::new(FileOutput::new(temp_file.path(), false).unwrap());
        let console_output = Box::new(ConsoleOutput::new(false));

        let multi_output = MultiOutput::new()
            .add_output(file_output)
            .add_output(console_output);

        let mut data = HashMap::new();
        data.insert(
            "message".to_string(),
            Value::String("Multi test".to_string()),
        );

        assert!(multi_output
            .write(&LogRecord {
                timestamp: "2025-09-07T10:30:00Z",
                level: LogLevel::Info,
                logger: "test",
                message: "Test message",
                context: &HashMap::new(),
                data: None
            })
            .is_ok());
        assert!(multi_output.flush().is_ok());
    }

    #[test]
    fn test_buffered_output() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_output = Arc::new(FileOutput::new(temp_file.path(), false).unwrap());
        let buffered = BufferedOutput::new(file_output, 3);

        let mut data = HashMap::new();
        data.insert(
            "message".to_string(),
            Value::String("Buffered message".to_string()),
        );

        // Add messages to buffer
        buffered
            .write(&LogRecord {
                timestamp: "2025-09-07T10:30:00Z",
                level: LogLevel::Info,
                logger: "test",
                message: "Buffered message",
                context: &HashMap::new(),
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
                context: &HashMap::new(),
                data: None,
            })
            .unwrap();
        assert_eq!(buffered.buffer_len(), 2);

        // This should trigger auto-flush
        buffered
            .write(&LogRecord {
                timestamp: "2025-09-07T10:30:00Z",
                level: LogLevel::Error,
                logger: "test",
                message: "Buffered message",
                context: &HashMap::new(),
                data: None,
            })
            .unwrap();
        assert_eq!(buffered.buffer_len(), 0);

        // Verify content was written to file
        buffered.flush().unwrap();
        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("Buffered message"));
    }
}
