//! Asynchronous logging support with bounded channels and backpressure control.
//!
//! This module provides [`AsyncOutput`], which processes log messages in a background
//! task with batching for improved performance. It uses bounded channels to prevent
//! memory exhaustion under high load.
//!
//! # Examples
//!
//! ```no_run
//! use telelog::{AsyncOutput, Logger};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     let logger = Logger::new("app");
//!     // AsyncOutput handles batching automatically
//!     logger.info("Async logging active");
//! }
//! ```

#[cfg(feature = "async")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature = "async")]
use tokio::sync::mpsc;
#[cfg(feature = "async")]
use tokio::time::{timeout, Duration};

use crate::level::LogLevel;
use crate::output::{LogMessage, OutputDestination};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Asynchronous output destination with bounded channel and backpressure.
///
/// Messages are batched and processed in a background task for optimal performance.
/// When the channel is full (capacity: 1000), writes will fail with `WouldBlock` error.
#[cfg(feature = "async")]
pub struct AsyncOutput {
    sender: mpsc::Sender<LogMessage>,
    _handle: tokio::task::JoinHandle<()>,
    shutdown: Arc<AtomicBool>,
}

#[cfg(feature = "async")]
impl AsyncOutput {
    /// Creates a new async output that wraps the given destination.
    ///
    /// # Arguments
    ///
    /// * `destination` - The underlying output destination to write to
    ///
    /// # Errors
    ///
    /// Returns an error if the background task cannot be spawned.
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

    /// Background task that processes log messages with batching.
    ///
    /// Messages are collected into batches of up to 100 messages or 100ms intervals,
    /// whichever comes first. This reduces I/O overhead while maintaining responsiveness.
    async fn background_task(
        mut receiver: mpsc::Receiver<LogMessage>,
        destination: Arc<dyn OutputDestination>,
        shutdown: Arc<AtomicBool>,
    ) {
        let mut batch = Vec::new();
        let batch_size = 100;
        let flush_interval = Duration::from_millis(100);

        loop {
            match timeout(flush_interval, receiver.recv()).await {
                Ok(Some(message)) => {
                    batch.push(message);

                    while batch.len() < batch_size {
                        match receiver.try_recv() {
                            Ok(message) => batch.push(message),
                            Err(_) => break,
                        }
                    }

                    Self::process_batch(&batch, &destination).await;
                    batch.clear();
                }
                Ok(None) => {
                    if !batch.is_empty() {
                        Self::process_batch(&batch, &destination).await;
                    }
                    break;
                }
                Err(_) => {
                    if !batch.is_empty() {
                        Self::process_batch(&batch, &destination).await;
                        batch.clear();
                    }
                }
            }

            if shutdown.load(Ordering::Relaxed) {
                while let Ok(message) = receiver.try_recv() {
                    batch.push(message);
                }
                if !batch.is_empty() {
                    Self::process_batch(&batch, &destination).await;
                }
                break;
            }
        }
    }

    /// Processes a batch of log messages sequentially.
    ///
    /// Writes all messages in the batch and flushes the destination once.
    async fn process_batch(batch: &[LogMessage], destination: &Arc<dyn OutputDestination>) {
        for message in batch {
            if let Err(e) = destination.write(message.level, &message.data) {
                eprintln!("Async log write error: {}", e);
            }
        }

        if let Err(e) = destination.flush() {
            eprintln!("Async log flush error: {}", e);
        }
    }

    /// Gracefully shuts down the async output and waits for pending messages.
    ///
    /// This ensures all buffered messages are written before the output is closed.
    ///
    /// # Errors
    ///
    /// Returns an error if the background task panicked or failed to complete.
    pub async fn shutdown(self) -> std::io::Result<()> {
        self.shutdown.store(true, Ordering::Relaxed);

        drop(self.sender);

        match self._handle.await {
            Ok(_) => Ok(()),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }
}

#[cfg(feature = "async")]
impl OutputDestination for AsyncOutput {
    /// Writes a log message to the async output channel.
    ///
    /// Uses non-blocking `try_send` to provide backpressure when the channel is full.
    ///
    /// # Errors
    ///
    /// Returns `WouldBlock` error if the channel is at capacity (1000 messages).
    fn write(&self, level: LogLevel, data: &HashMap<String, Value>) -> std::io::Result<()> {
        let message = LogMessage::new(level, data.clone());

        self.sender.try_send(message).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::WouldBlock,
                format!("Log channel full (backpressure active): {}", e),
            )
        })?;

        Ok(())
    }

    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(all(test, feature = "async"))]
mod tests {
    use super::*;
    use crate::output::ConsoleOutput;
    use std::time::Duration;
    use tokio;

    #[tokio::test]
    async fn test_async_output() {
        let console = Arc::new(ConsoleOutput::new(false));
        let async_output = AsyncOutput::new(console).unwrap();

        let mut data = HashMap::new();
        data.insert(
            "message".to_string(),
            Value::String("Test async message".to_string()),
        );

        // Send some messages
        for i in 0..10 {
            data.insert("count".to_string(), Value::Number(i.into()));
            async_output.write(LogLevel::Info, &data).unwrap();
        }

        // Give background task time to process
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Shutdown
        async_output.shutdown().await.unwrap();
    }
}
