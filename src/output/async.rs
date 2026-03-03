//! Asynchronous output destination with bounded channel and backpressure.
//!
//! [`AsyncOutput`] wraps any [`OutputDestination`] and offloads writes to a
//! background Tokio task. The caller's [`write`](OutputDestination::write) call
//! serializes the record to bytes and enqueues it via a non-blocking `try_send`,
//! returning immediately. The background task drains the channel in batches of
//! up to 100 records every 100 ms, minimising I/O syscalls.
//!
//! When the channel is full (capacity: 1000) a `WouldBlock` error is returned,
//! giving the caller explicit backpressure feedback.
//!
//! # Example
//!
//! ```no_run
//! use telelog::output::{AsyncOutput, ConsoleOutput};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     let dest = Arc::new(ConsoleOutput::new(false));
//!     let async_out = AsyncOutput::new(dest).unwrap();
//!     // use async_out as an OutputDestination …
//!     async_out.shutdown().await.unwrap();
//! }
//! ```

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

use crate::output::{LogRecord, OutputDestination};

/// Asynchronous output destination with bounded channel and backpressure.
pub struct AsyncOutput {
    sender: mpsc::Sender<Vec<u8>>,
    _handle: tokio::task::JoinHandle<()>,
    shutdown: Arc<AtomicBool>,
}

impl AsyncOutput {
    /// Creates a new async output wrapping the given destination.
    ///
    /// Spawns a Tokio background task that drains the channel in batches.
    pub fn new(destination: Arc<dyn OutputDestination>) -> std::io::Result<Self> {
        let (sender, receiver) = mpsc::channel(1000);
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = Arc::clone(&shutdown);

        let handle = tokio::spawn(async move {
            Self::background_task(receiver, destination, shutdown_clone).await;
        });

        Ok(Self {
            sender,
            _handle: handle,
            shutdown,
        })
    }

    /// Background task: collects records into batches, flushes every 100 ms.
    async fn background_task(
        mut receiver: mpsc::Receiver<Vec<u8>>,
        destination: Arc<dyn OutputDestination>,
        shutdown: Arc<AtomicBool>,
    ) {
        let mut batch: Vec<Vec<u8>> = Vec::new();
        let batch_size = 100;
        let flush_interval = Duration::from_millis(100);

        loop {
            match timeout(flush_interval, receiver.recv()).await {
                Ok(Some(message)) => {
                    batch.push(message);
                    while batch.len() < batch_size {
                        match receiver.try_recv() {
                            Ok(msg) => batch.push(msg),
                            Err(_) => break,
                        }
                    }
                    Self::flush_batch(&batch, &destination).await;
                    batch.clear();
                }
                Ok(None) => {
                    if !batch.is_empty() {
                        Self::flush_batch(&batch, &destination).await;
                    }
                    break;
                }
                Err(_timeout) => {
                    if !batch.is_empty() {
                        Self::flush_batch(&batch, &destination).await;
                        batch.clear();
                    }
                }
            }

            if shutdown.load(Ordering::Relaxed) {
                while let Ok(msg) = receiver.try_recv() {
                    batch.push(msg);
                }
                if !batch.is_empty() {
                    Self::flush_batch(&batch, &destination).await;
                }
                break;
            }
        }
    }

    /// Writes a batch of pre-serialized records and flushes the destination.
    async fn flush_batch(batch: &[Vec<u8>], destination: &Arc<dyn OutputDestination>) {
        for bytes in batch {
            if let Err(e) = destination.write_bytes(bytes) {
                eprintln!("AsyncOutput write error: {}", e);
            }
        }
        if let Err(e) = destination.flush() {
            eprintln!("AsyncOutput flush error: {}", e);
        }
    }

    /// Signals shutdown and waits for the background task to drain and exit.
    pub async fn shutdown(self) -> std::io::Result<()> {
        self.shutdown.store(true, Ordering::Relaxed);
        drop(self.sender);
        self._handle
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

impl OutputDestination for AsyncOutput {
    /// Serializes the record to JSON bytes and enqueues it without blocking.
    ///
    /// Returns `WouldBlock` if the channel is at capacity (1000 records).
    fn write(&self, record: &LogRecord<'_>) -> std::io::Result<()> {
        let mut buf = Vec::with_capacity(256);
        serde_json::to_writer(&mut buf, record)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        buf.push(b'\n');

        self.sender.try_send(buf).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::WouldBlock,
                format!("Log channel full (backpressure): {}", e),
            )
        })
    }

    fn flush(&self) -> std::io::Result<()> {
        // Flushing is handled by the background task; nothing to do here.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::LogLevel;
    use crate::output::console::ConsoleOutput;
    use std::collections::HashMap;
    use std::time::Duration;

    #[tokio::test]
    async fn test_async_output() {
        let console = Arc::new(ConsoleOutput::new(false));
        let async_output = AsyncOutput::new(console).unwrap();

        let ctx = HashMap::new();
        for i in 0..10u64 {
            let i_str = i.to_string();
            async_output
                .write(&LogRecord {
                    timestamp: "2025-09-07T10:30:00Z",
                    level: LogLevel::Info,
                    logger: "test",
                    message: "Test async message",
                    context: &ctx,
                    data: Some(&[("count", &i_str)]),
                })
                .unwrap();
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
        async_output.shutdown().await.unwrap();
    }
}
