//! File-based output destinations: plain/JSON file and size-rotating file.

use crate::output::{LogRecord, OutputDestination};
use parking_lot::Mutex;
use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Appends log records to a single file in either plain-text or JSON format.
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
    /// * `json_format` - If true, writes newline-delimited JSON; otherwise plain text
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

/// Appends log records to a file, rotating to a new file when a size limit is reached.
///
/// Rotated files are numbered (e.g. `app.log.1`, `app.log.2`); the oldest is deleted
/// once `max_files` is exceeded.
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
    /// # Arguments
    ///
    /// * `base_path` - Base path for log files
    /// * `max_size` - Maximum file size in bytes before rotation
    /// * `max_files` - Maximum number of rotated files to keep
    /// * `json_format` - If true, writes newline-delimited JSON; otherwise plain text
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
            std::fs::rename(&self.base_path, self.get_rotated_path(1))?;
        }

        Ok(())
    }

    fn get_rotated_path(&self, number: u32) -> PathBuf {
        let mut path = self.base_path.clone();
        let stem = path.file_stem().unwrap_or_default().to_string_lossy().into_owned();
        let ext = path.extension().unwrap_or_default().to_string_lossy().into_owned();

        if ext.is_empty() {
            path.set_file_name(format!("{}.{}", stem, number));
        } else {
            path.set_file_name(format!("{}.{}.{}", stem, number, ext));
        }
        path
    }

    fn ensure_file(&self) -> io::Result<()> {
        let mut current = self.current_file.lock();
        if current.is_none() {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.base_path)?;
            *self.current_size.lock() = file.metadata()?.len();
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
                format!(
                    "{} [{}] {}: {}\n",
                    record.timestamp, record.level, record.logger, record.message
                )
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

    fn write_bytes(&self, bytes: &[u8]) -> io::Result<()> {
        self.rotate_if_needed()?;
        self.ensure_file()?;
        let mut current = self.current_file.lock();
        if let Some(ref mut writer) = *current {
            writer.write_all(bytes)?;
            *self.current_size.lock() += bytes.len() as u64;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::LogLevel;
    use std::collections::HashMap;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_output() {
        let temp_file = NamedTempFile::new().unwrap();
        let output = FileOutput::new(temp_file.path(), false).unwrap();

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

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("Test message"));
    }
}
