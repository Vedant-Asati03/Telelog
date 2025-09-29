//! Main logger implementation

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
use std::collections::HashMap;
use std::sync::Arc;

/// The main logger struct
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
    /// Create a new logger with the given name and default configuration
    pub fn new(name: &str) -> Self {
        Self::with_config(name, Config::default())
    }

    /// Create a new logger with custom configuration
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

    /// Build output destinations based on configuration
    fn build_output(config: &Config) -> Arc<dyn OutputDestination> {
        let mut multi_output = MultiOutput::new();

        // Add console output if enabled
        if config.output.console_enabled {
            let console = Box::new(ConsoleOutput::new(config.output.colored_output));
            multi_output = multi_output.add_output(console);
        }

        // Add file output if enabled
        if config.output.file_enabled {
            if let Some(file_path) = &config.output.file_path {
                // Use rotating file output by default when max_file_size is reasonable
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
                            // Fall back to regular file output
                            if let Ok(file) = FileOutput::new(file_path, config.output.json_format)
                            {
                                multi_output = multi_output.add_output(Box::new(file));
                            }
                        }
                    }
                } else {
                    // Use regular file output
                    if let Ok(file) = FileOutput::new(file_path, config.output.json_format) {
                        multi_output = multi_output.add_output(Box::new(file));
                    }
                }
            }
        }

        let output: Arc<dyn OutputDestination> = Arc::new(multi_output);

        // Wrap with buffering if enabled
        let output = if config.performance.buffering_enabled {
            Arc::new(BufferedOutput::new(output, config.performance.buffer_size))
        } else {
            output
        };

        // Wrap with async if enabled
        #[cfg(feature = "async")]
        let output = if config.performance.async_enabled {
            match AsyncOutput::new(output.clone()) {
                Ok(async_output) => Arc::new(async_output) as Arc<dyn OutputDestination>,
                Err(e) => {
                    eprintln!("Failed to create async output: {}", e);
                    output // Fall back to sync output
                }
            }
        } else {
            output
        };

        #[cfg(not(feature = "async"))]
        let output = output;

        output
    }

    /// Get the logger name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the component tracker for visualization
    pub fn get_component_tracker(&self) -> &ComponentTracker {
        &self.component_tracker
    }

    /// Update the logger configuration
    pub fn set_config(&self, config: Config) {
        if let Err(e) = config.validate() {
            eprintln!("Invalid configuration: {}", e);
            return;
        }

        // Rebuild output with new config
        let _new_output = Self::build_output(&config);

        // Update both config and output atomically
        *self.config.write() = config;

        // Note: We can't replace the Arc<dyn OutputDestination> easily here
        // This is a limitation - in a real implementation we might want to
        // redesign this to allow dynamic reconfiguration
        eprintln!("Warning: Configuration updated but output destinations remain unchanged");
        eprintln!(
            "Consider creating a new logger instance for the new configuration to take full effect"
        );
    }

    /// Get a copy of the current configuration
    pub fn get_config(&self) -> Config {
        self.config.read().clone()
    }

    /// Log a debug message
    pub fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message, None);
    }

    /// Log a debug message with structured data
    pub fn debug_with(&self, message: &str, data: &[(&str, &str)]) {
        self.log(LogLevel::Debug, message, Some(data));
    }

    /// Log an info message
    pub fn info(&self, message: &str) {
        self.log(LogLevel::Info, message, None);
    }

    /// Log an info message with structured data
    pub fn info_with(&self, message: &str, data: &[(&str, &str)]) {
        self.log(LogLevel::Info, message, Some(data));
    }

    /// Log a warning message
    pub fn warning(&self, message: &str) {
        self.log(LogLevel::Warning, message, None);
    }

    /// Log a warning message with structured data
    pub fn warning_with(&self, message: &str, data: &[(&str, &str)]) {
        self.log(LogLevel::Warning, message, Some(data));
    }

    /// Log an error message
    pub fn error(&self, message: &str) {
        self.log(LogLevel::Error, message, None);
    }

    /// Log an error message with structured data
    pub fn error_with(&self, message: &str, data: &[(&str, &str)]) {
        self.log(LogLevel::Error, message, Some(data));
    }

    /// Log a critical message
    pub fn critical(&self, message: &str) {
        self.log(LogLevel::Critical, message, None);
    }

    /// Log a critical message with structured data
    pub fn critical_with(&self, message: &str, data: &[(&str, &str)]) {
        self.log(LogLevel::Critical, message, Some(data));
    }

    /// Add context that will be included in all subsequent log messages
    pub fn add_context(&self, key: &str, value: &str) {
        self.context.write().add(key, value);
    }

    /// Remove context
    pub fn remove_context(&self, key: &str) {
        self.context.write().remove(key);
    }

    /// Clear all context
    pub fn clear_context(&self) {
        self.context.write().clear();
    }

    /// Start profiling an operation
    pub fn profile(&self, operation: &str) -> ProfileGuard {
        ProfileGuard::new(operation, self.clone())
    }

    /// Start tracking a component (returns RAII guard)
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

    /// Get access to the component tracker
    pub fn component_tracker(&self) -> &Arc<ComponentTracker> {
        &self.component_tracker
    }

    /// Get access to the system monitor (if available)
    #[cfg(feature = "system-monitor")]
    pub fn system_monitor(&self) -> &Arc<RwLock<SystemMonitor>> {
        &self.system_monitor
    }

    /// Internal logging method
    fn log(&self, level: LogLevel, message: &str, data: Option<&[(&str, &str)]>) {
        let config = self.config.read();

        // Check if we should log this level
        if !level.should_log(config.min_level) {
            return;
        }

        // Build log entry
        let mut log_data = HashMap::new();
        log_data.insert(
            "timestamp".to_string(),
            Value::String(chrono::Utc::now().to_rfc3339()),
        );
        log_data.insert("level".to_string(), Value::String(level.to_string()));
        log_data.insert("logger".to_string(), Value::String(self.name.clone()));
        log_data.insert("message".to_string(), Value::String(message.to_string()));

        // Add context
        let context = self.context.read();
        for (key, value) in context.iter() {
            log_data.insert(key.clone(), Value::String(value.clone()));
        }

        // Add structured data
        if let Some(data) = data {
            for (key, value) in data {
                log_data.insert(key.to_string(), Value::String(value.to_string()));
            }
        }

        // Output the log
        self.output_log(level, &log_data, &config);
    }

    /// Output the log to configured destinations
    fn output_log(&self, level: LogLevel, data: &HashMap<String, Value>, _config: &Config) {
        // Use the unified output system
        if let Err(e) = self.output.write(level, data) {
            eprintln!("Failed to write log: {}", e);
        }
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
