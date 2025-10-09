//! Core logger implementation providing structured logging, profiling, and component tracking.

use crate::component::{ComponentGuard, ComponentTracker};
use crate::output::{
    BufferedOutput, ConsoleOutput, FileOutput, MultiOutput, OutputDestination, RotatingFileOutput,
};
use crate::{config::Config, context::Context, level::LogLevel, profile::ProfileGuard};

#[cfg(feature = "async")]
use crate::async_output::AsyncOutput;

#[cfg(feature = "system-monitor")]
use crate::monitor::SystemMonitor;

use parking_lot::RwLock;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

thread_local! {
    /// Thread-local buffer pool to reuse HashMaps and avoid allocations
    static LOG_BUFFER: RefCell<HashMap<String, Value>> = RefCell::new(HashMap::with_capacity(16));

    /// Thread-local timestamp buffer to avoid allocation
    static TIMESTAMP_BUFFER: RefCell<String> = RefCell::new(String::with_capacity(35));
}

/// Main logger instance providing structured logging, context management, and performance profiling.
///
/// The logger is thread-safe and can be cloned cheaply for use across multiple threads.
///
/// # Examples
///
/// ```
/// use telelog::Logger;
///
/// let logger = Logger::new("my_app");
/// logger.info("Application started");
///
/// // Structured logging
/// logger.info_with("User logged in", &[("user_id", "123")]);
///
/// // Context management
/// logger.add_context("request_id", "abc");
/// logger.info("Processing"); // includes request_id
/// logger.clear_context();
/// ```
pub struct Logger {
    name: String,
    config: Arc<RwLock<Config>>,
    context: Arc<RwLock<Context>>,
    output: Arc<dyn OutputDestination>,
    component_tracker: Arc<ComponentTracker>,
    #[cfg(feature = "system-monitor")]
    system_monitor: Arc<RwLock<SystemMonitor>>,
}

impl Logger {
    /// Creates a new logger with the given name and default configuration.
    pub fn new(name: &str) -> Self {
        Self::with_config(name, Config::default())
    }

    /// Creates a new logger with a custom configuration.
    ///
    /// # Arguments
    ///
    /// * `name` - The logger name, included in all log messages
    /// * `config` - Configuration for output, performance, and visualization
    ///
    /// # Panics
    ///
    /// Panics if the configuration fails validation.
    pub fn with_config(name: &str, config: Config) -> Self {
        if let Err(e) = config.validate() {
            panic!("Invalid configuration: {}", e);
        }

        // Build output destinations based on config
        let output = Self::build_output(&config);

        Self {
            name: name.to_string(),
            config: Arc::new(RwLock::new(config)),
            context: Arc::new(RwLock::new(Context::new())),
            output,
            component_tracker: Arc::new(ComponentTracker::new()),
            #[cfg(feature = "system-monitor")]
            system_monitor: Arc::new(RwLock::new(SystemMonitor::new())),
        }
    }

    fn build_output(config: &Config) -> Arc<dyn OutputDestination> {
        let mut multi_output = MultiOutput::new();

        if config.output.console_enabled {
            let console = Box::new(ConsoleOutput::new(config.output.colored_output));
            multi_output = multi_output.add_output(console);
        }

        if config.output.file_enabled {
            if let Some(file_path) = &config.output.file_path {
                if config.output.max_file_size > 0 && config.output.max_files > 1 {
                    match RotatingFileOutput::new(
                        file_path,
                        config.output.max_file_size,
                        config.output.max_files,
                        config.output.json_format,
                    ) {
                        Ok(rotating) => {
                            multi_output = multi_output.add_output(Box::new(rotating));
                        }
                        Err(e) => {
                            eprintln!("Failed to create rotating file output: {}", e);
                            if let Ok(file) = FileOutput::new(file_path, config.output.json_format)
                            {
                                multi_output = multi_output.add_output(Box::new(file));
                            }
                        }
                    }
                } else {
                    if let Ok(file) = FileOutput::new(file_path, config.output.json_format) {
                        multi_output = multi_output.add_output(Box::new(file));
                    }
                }
            }
        }
        let output: Arc<dyn OutputDestination> = Arc::new(multi_output);

        let output = if config.performance.buffering_enabled {
            Arc::new(BufferedOutput::new(output, config.performance.buffer_size))
        } else {
            output
        };

        #[cfg(feature = "async")]
        let output = if config.performance.async_enabled {
            match AsyncOutput::new(output.clone()) {
                Ok(async_output) => Arc::new(async_output) as Arc<dyn OutputDestination>,
                Err(e) => {
                    eprintln!("Failed to create async output: {}", e);
                    output
                }
            }
        } else {
            output
        };

        #[cfg(not(feature = "async"))]
        let output = output;

        output
    }

    /// Returns the logger's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a reference to the component tracker.
    pub fn get_component_tracker(&self) -> &ComponentTracker {
        &self.component_tracker
    }

    /// Updates the logger configuration.
    ///
    /// # Note
    ///
    /// Output destinations cannot be changed dynamically. Create a new logger instance
    /// for configuration changes to fully take effect.
    pub fn set_config(&self, config: Config) {
        if let Err(e) = config.validate() {
            eprintln!("Invalid configuration: {}", e);
            return;
        }

        let _new_output = Self::build_output(&config);
        *self.config.write() = config;

        eprintln!("Warning: Configuration updated but output destinations remain unchanged");
        eprintln!(
            "Consider creating a new logger instance for the new configuration to take full effect"
        );
    }

    /// Returns a copy of the current configuration.
    pub fn get_config(&self) -> Config {
        self.config.read().clone()
    }

    /// Logs a debug-level message.
    pub fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message, None);
    }

    /// Logs a debug-level message with structured data.
    pub fn debug_with(&self, message: &str, data: &[(&str, &str)]) {
        self.log(LogLevel::Debug, message, Some(data));
    }

    /// Logs an info-level message.
    pub fn info(&self, message: &str) {
        self.log(LogLevel::Info, message, None);
    }

    /// Logs an info-level message with structured data.
    pub fn info_with(&self, message: &str, data: &[(&str, &str)]) {
        self.log(LogLevel::Info, message, Some(data));
    }

    /// Logs a warning-level message.
    pub fn warning(&self, message: &str) {
        self.log(LogLevel::Warning, message, None);
    }

    /// Logs a warning-level message with structured data.
    pub fn warning_with(&self, message: &str, data: &[(&str, &str)]) {
        self.log(LogLevel::Warning, message, Some(data));
    }

    /// Logs an error-level message.
    pub fn error(&self, message: &str) {
        self.log(LogLevel::Error, message, None);
    }

    /// Logs an error-level message with structured data.
    pub fn error_with(&self, message: &str, data: &[(&str, &str)]) {
        self.log(LogLevel::Error, message, Some(data));
    }

    /// Logs a critical-level message.
    pub fn critical(&self, message: &str) {
        self.log(LogLevel::Critical, message, None);
    }

    /// Logs a critical-level message with structured data.
    pub fn critical_with(&self, message: &str, data: &[(&str, &str)]) {
        self.log(LogLevel::Critical, message, Some(data));
    }

    /// Logs a message with a specific level and structured data.
    ///
    /// # Example
    ///
    /// ```
    /// use telelog::{Logger, LogLevel};
    ///
    /// let logger = Logger::new("app");
    /// logger.log_with(LogLevel::Info, "User action", &[
    ///     ("user_id", "12345"),
    ///     ("action", "login"),
    /// ]);
    /// ```
    pub fn log_with(&self, level: LogLevel, message: &str, data: &[(&str, &str)]) {
        self.log(level, message, Some(data));
    }

    /// Adds context that will be included in all subsequent log messages.
    pub fn add_context(&self, key: &str, value: &str) {
        self.context.write().add(key, value);
    }

    /// Removes a specific key from the context.
    pub fn remove_context(&self, key: &str) {
        self.context.write().remove(key);
    }

    /// Clears all context key-value pairs.
    pub fn clear_context(&self) {
        self.context.write().clear();
    }

    /// Adds temporary context that will be automatically removed when the guard is dropped.
    ///
    /// # Example
    ///
    /// ```
    /// use telelog::Logger;
    ///
    /// let logger = Logger::new("app");
    /// {
    ///     let _guard = logger.with_context("request_id", "12345");
    ///     logger.info("Processing request"); // includes request_id
    /// } // request_id is automatically removed here
    /// logger.info("After request"); // no request_id
    /// ```
    pub fn with_context(&self, key: &str, value: &str) -> crate::context::ContextGuard {
        self.context.write().add(key, value);
        crate::context::ContextGuard::new(key.to_string(), Arc::clone(&self.context))
    }

    /// Starts profiling an operation. Returns a guard that logs duration when dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use telelog::Logger;
    ///
    /// let logger = Logger::new("app");
    /// {
    ///     let _guard = logger.profile("database_query");
    ///     // Query execution...
    /// } // Duration automatically logged
    /// ```
    pub fn profile(&self, operation: &str) -> ProfileGuard {
        ProfileGuard::new(operation, self.clone())
    }

    /// Starts tracking a component. Returns a guard that marks completion when dropped.
    ///
    /// With `system-monitor` feature, automatically tracks memory usage.
    pub fn track_component(&self, name: &str) -> ComponentGuard {
        #[cfg(feature = "system-monitor")]
        {
            ComponentGuard::new_with_monitor(
                name,
                Arc::clone(&self.component_tracker),
                Arc::clone(&self.system_monitor),
            )
        }
        #[cfg(not(feature = "system-monitor"))]
        {
            ComponentGuard::new(name, Arc::clone(&self.component_tracker))
        }
    }

    /// Returns a reference to the component tracker for advanced operations.
    pub fn component_tracker(&self) -> &Arc<ComponentTracker> {
        &self.component_tracker
    }

    /// Generates a visualization diagram from tracked components.
    ///
    /// # Arguments
    ///
    /// * `chart_type` - The type of chart to generate (Flowchart, Timeline, or Gantt)
    /// * `output_path` - Optional path to save the generated diagram
    ///
    /// # Example
    ///
    /// ```no_run
    /// use telelog::{Logger, ChartType};
    ///
    /// let logger = Logger::new("app");
    /// let diagram = logger.generate_visualization(ChartType::Flowchart, Some("chart.mmd")).unwrap();
    /// ```
    pub fn generate_visualization(
        &self,
        chart_type: crate::visualization::ChartType,
        output_path: Option<&str>,
    ) -> Result<String, String> {
        use crate::visualization::{ChartConfig, MermaidGenerator};

        let config = ChartConfig::new().with_chart_type(chart_type);
        let generator = MermaidGenerator::new(config);
        let diagram = generator.generate_diagram(&self.component_tracker)?;

        if let Some(path) = output_path {
            std::fs::write(path, &diagram)
                .map_err(|e| format!("Failed to write diagram to file: {}", e))?;
        }

        Ok(diagram)
    }

    /// Returns a reference to the system monitor (requires `system-monitor` feature).
    #[cfg(feature = "system-monitor")]
    pub fn system_monitor(&self) -> &Arc<RwLock<SystemMonitor>> {
        &self.system_monitor
    }

    /// Internal logging implementation with thread-local buffer optimization.
    fn log(&self, level: LogLevel, message: &str, data: Option<&[(&str, &str)]>) {
        let config = self.config.read();

        if !level.should_log(config.min_level) {
            return;
        }

        LOG_BUFFER.with(|buffer| {
            let mut log_data = buffer.borrow_mut();
            log_data.clear();

            let timestamp = TIMESTAMP_BUFFER.with(|ts_buf| {
                let mut ts = ts_buf.borrow_mut();
                ts.clear();
                use std::fmt::Write;
                write!(ts, "{}", chrono::Utc::now().to_rfc3339()).unwrap();
                ts.clone()
            });

            log_data.insert("timestamp".to_string(), Value::String(timestamp));
            log_data.insert("level".to_string(), Value::String(level.to_string()));
            log_data.insert("logger".to_string(), Value::String(self.name.clone()));
            log_data.insert("message".to_string(), Value::String(message.to_string()));

            let context = self.context.read();
            for (key, value) in context.iter() {
                log_data.insert(key.clone(), Value::String(value.clone()));
            }
            drop(context);

            if let Some(data) = data {
                for (key, value) in data {
                    log_data.insert(key.to_string(), Value::String(value.to_string()));
                }
            }

            if let Err(e) = self.output.write(level, &log_data) {
                eprintln!("Failed to write log: {}", e);
            }
        });
    }
}

impl Clone for Logger {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            config: Arc::clone(&self.config),
            context: Arc::clone(&self.context),
            output: Arc::clone(&self.output),
            component_tracker: Arc::clone(&self.component_tracker),
            #[cfg(feature = "system-monitor")]
            system_monitor: Arc::clone(&self.system_monitor),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_creation() {
        let logger = Logger::new("test");
        assert_eq!(logger.name(), "test");
    }

    #[test]
    fn test_logging_methods() {
        let logger = Logger::new("test");

        // These should not panic
        logger.debug("Debug message");
        logger.info("Info message");
        logger.warning("Warning message");
        logger.error("Error message");
        logger.critical("Critical message");
    }

    #[test]
    fn test_context_management() {
        let logger = Logger::new("test");

        logger.add_context("user_id", "12345");
        logger.add_context("session_id", "abcdef");

        // Context should be included in logs
        logger.info("Test message with context");

        logger.remove_context("user_id");
        logger.clear_context();
    }

    #[test]
    fn test_config_update() {
        let logger = Logger::new("test");
        let new_config = Config::new().with_min_level(LogLevel::Warning);

        logger.set_config(new_config);
        let current_config = logger.get_config();
        assert_eq!(current_config.min_level, LogLevel::Warning);
    }
}
