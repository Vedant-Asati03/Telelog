//! Async logging support

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

/// Async output destination that buffers messages and sends them to background tasks
#[cfg(feature = "async")]
pub struct AsyncOutput {
    sender: mpsc::UnboundedSender<LogMessage>,
    _handle: tokio::task::JoinHandle<()>,
    shutdown: Arc<AtomicBool>,
}

#[cfg(feature = "async")]
impl AsyncOutput {
    /// Create a new async output with the given destination
    pub fn new(destination: Arc<dyn OutputDestination>) -> std::io::Result<Self> {
        let (sender, receiver) = mpsc::unbounded_channel();
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

    /// Background task that processes log messages
    async fn background_task(
        mut receiver: mpsc::UnboundedReceiver<LogMessage>,
        destination: Arc<dyn OutputDestination>,
        shutdown: Arc<AtomicBool>,
    ) {
        let mut batch = Vec::new();
        let batch_size = 100;
        let flush_interval = Duration::from_millis(100);

        loop {
            // Try to receive messages with timeout for periodic flushing
            match timeout(flush_interval, receiver.recv()).await {
                Ok(Some(message)) => {
                    batch.push(message);

                    // Collect more messages up to batch size
                    while batch.len() < batch_size {
                        match receiver.try_recv() {
                            Ok(message) => batch.push(message),
                            Err(_) => break,
                        }
                    }

                    // Process the batch
                    Self::process_batch(&batch, &destination).await;
                    batch.clear();
                }
                Ok(None) => {
                    // Channel closed, process remaining messages and exit
                    if !batch.is_empty() {
                        Self::process_batch(&batch, &destination).await;
                    }
                    break;
                }
                Err(_) => {
                    // Timeout - flush any pending messages
                    if !batch.is_empty() {
                        Self::process_batch(&batch, &destination).await;
                        batch.clear();
                    }
                }
            }

            // Check for shutdown signal
            if shutdown.load(Ordering::Relaxed) {
                // Process remaining messages
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

    /// Process a batch of log messages
    async fn process_batch(batch: &[LogMessage], destination: &Arc<dyn OutputDestination>) {
        for message in batch {
            if let Err(e) = destination.write(message.level, &message.data) {
                eprintln!("Async log write error: {}", e);
            }
        }

        // Flush after processing batch
        if let Err(e) = destination.flush() {
            eprintln!("Async log flush error: {}", e);
        }
    }

    /// Shutdown the async logger and wait for pending messages
    pub async fn shutdown(self) -> std::io::Result<()> {
        self.shutdown.store(true, Ordering::Relaxed);

        // Close the sender to signal the background task
        drop(self.sender);

        // Wait for the background task to complete
        match self._handle.await {
            Ok(_) => Ok(()),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }
}

#[cfg(feature = "async")]
impl OutputDestination for AsyncOutput {
    fn write(&self, level: LogLevel, data: &HashMap<String, Value>) -> std::io::Result<()> {
        let message = LogMessage::new(level, data.clone());

        self.sender.send(message).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                format!("Failed to send log message: {}", e),
            )
        })?;

        Ok(())
    }

    fn flush(&self) -> std::io::Result<()> {
        // For async output, flush is handled by the background task
        // We could implement a flush mechanism using additional channels if needed
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
