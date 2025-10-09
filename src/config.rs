//! Configuration management for telelog with preset configurations.
//!
//! Provides a flexible configuration system with builder pattern and presets
//! for common use cases (development, production, performance analysis).
//!
//! # Examples
//!
//! ```
//! use telelog::Config;
//!
//! // Custom configuration
//! let config = Config::new()
//!     .with_min_level(telelog::LogLevel::Debug)
//!     .with_console_output(true)
//!     .with_file_output("app.log");
//!
//! // Or use presets
//! let dev_config = Config::development();
//! let prod_config = Config::production("logs/app.log");
//! ```

use crate::level::LogLevel;
use crate::visualization::{ChartConfig, ChartType};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Output-related configuration options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub console_enabled: bool,
    pub colored_output: bool,
    pub file_enabled: bool,
    pub file_path: Option<PathBuf>,
    pub json_format: bool,
    pub max_file_size: u64,
    pub max_files: u32,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            console_enabled: true,
            colored_output: true,
            file_enabled: false,
            file_path: None,
            json_format: false,
            max_file_size: 10 * 1024 * 1024,
            max_files: 5,
        }
    }
}

/// Performance and profiling configuration options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub profiling_enabled: bool,
    pub monitoring_enabled: bool,
    pub component_tracking_enabled: bool,
    pub buffer_size: usize,
    pub buffering_enabled: bool,
    #[cfg(feature = "async")]
    pub async_enabled: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            profiling_enabled: true,
            monitoring_enabled: false,
            component_tracking_enabled: false,
            buffer_size: 1024,
            buffering_enabled: false,
            #[cfg(feature = "async")]
            async_enabled: false,
        }
    }
}

/// Visualization and chart generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VisualizationConfig {
    pub chart_config: Option<ChartConfig>,
    pub auto_generate_charts: bool,
    pub output_directory: Option<PathBuf>,
}

/// Main configuration for the logger.
///
/// Combines output, performance, and visualization settings with a builder pattern
/// for easy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub min_level: LogLevel,
    pub output: OutputConfig,
    pub performance: PerformanceConfig,
    pub visualization: VisualizationConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            min_level: LogLevel::Info,
            output: OutputConfig::default(),
            performance: PerformanceConfig::default(),
            visualization: VisualizationConfig::default(),
        }
    }
}

impl Config {
    /// Creates a new configuration with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the minimum log level for filtering messages.
    pub fn with_min_level(mut self, level: LogLevel) -> Self {
        self.min_level = level;
        self
    }

    /// Enables or disables console output.
    pub fn with_console_output(mut self, enabled: bool) -> Self {
        self.output.console_enabled = enabled;
        self
    }

    /// Enables file output with the specified path.
    pub fn with_file_output<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.output.file_enabled = true;
        self.output.file_path = Some(path.into());
        self
    }

    /// Enables or disables JSON format for structured logging.
    pub fn with_json_format(mut self, enabled: bool) -> Self {
        self.output.json_format = enabled;
        self
    }

    /// Enables or disables colored console output.
    pub fn with_colored_output(mut self, enabled: bool) -> Self {
        self.output.colored_output = enabled;
        self
    }

    /// Enables or disables performance profiling.
    pub fn with_profiling(mut self, enabled: bool) -> Self {
        self.performance.profiling_enabled = enabled;
        self
    }

    /// Enables or disables system monitoring.
    pub fn with_monitoring(mut self, enabled: bool) -> Self {
        self.performance.monitoring_enabled = enabled;
        self
    }

    /// Sets the buffer size for buffered output.
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.performance.buffer_size = size;
        self
    }

    /// Configures file rotation with maximum file size and file count.
    pub fn with_file_rotation(mut self, max_size: u64, max_files: u32) -> Self {
        self.output.max_file_size = max_size;
        self.output.max_files = max_files;
        self
    }

    /// Enables or disables async output (requires `async` feature).
    #[cfg(feature = "async")]
    pub fn with_async(mut self, enabled: bool) -> Self {
        self.performance.async_enabled = enabled;
        self
    }

    /// Enables or disables buffered output.
    pub fn with_buffering(mut self, enabled: bool) -> Self {
        self.performance.buffering_enabled = enabled;
        self
    }

    /// Enables or disables component tracking.
    pub fn with_component_tracking(mut self, enabled: bool) -> Self {
        self.performance.component_tracking_enabled = enabled;
        self
    }

    /// Sets the chart configuration for visualization.
    pub fn with_chart_config(mut self, config: ChartConfig) -> Self {
        self.visualization.chart_config = Some(config);
        self
    }

    /// Enables or disables automatic chart generation.
    pub fn with_auto_generate_charts(mut self, enabled: bool) -> Self {
        self.visualization.auto_generate_charts = enabled;
        self
    }

    /// Sets the output directory for generated charts.
    pub fn with_chart_output_directory<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.visualization.output_directory = Some(path.into());
        self
    }

    /// Creates a preset development configuration.
    ///
    /// Includes debug logging, colored console output, profiling, and component tracking.
    pub fn development() -> Self {
        Self::new()
            .with_min_level(LogLevel::Debug)
            .with_console_output(true)
            .with_colored_output(true)
            .with_profiling(true)
            .with_component_tracking(true)
    }

    /// Creates a preset production configuration.
    ///
    /// Uses info-level logging, JSON format, file output, and system monitoring.
    pub fn production<P: Into<PathBuf>>(log_file: P) -> Self {
        Self::new()
            .with_min_level(LogLevel::Info)
            .with_console_output(false)
            .with_file_output(log_file)
            .with_json_format(true)
            .with_colored_output(false)
            .with_monitoring(true)
    }

    /// Creates a configuration optimized for performance analysis with visualization.
    ///
    /// Includes all tracking and monitoring features with timeline chart generation.
    pub fn performance_analysis<P: Into<PathBuf>>(output_dir: P) -> Self {
        let chart_config = ChartConfig::new()
            .with_chart_type(ChartType::Timeline)
            .with_timing(true)
            .with_memory(true);

        Self::new()
            .with_min_level(LogLevel::Debug)
            .with_console_output(true)
            .with_profiling(true)
            .with_monitoring(true)
            .with_component_tracking(true)
            .with_chart_config(chart_config)
            .with_auto_generate_charts(true)
            .with_chart_output_directory(output_dir)
    }

    /// Validates the configuration for correctness.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File output is enabled but no path is specified
    /// - Buffer size is zero
    /// - Max file size is zero
    /// - Auto-generate charts is enabled without chart configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.output.file_enabled && self.output.file_path.is_none() {
            return Err("File output enabled but no file path specified".to_string());
        }

        if self.performance.buffer_size == 0 {
            return Err("Buffer size must be greater than 0".to_string());
        }

        if self.output.max_file_size == 0 {
            return Err("Max file size must be greater than 0".to_string());
        }

        if self.visualization.auto_generate_charts && self.visualization.chart_config.is_none() {
            return Err(
                "Auto-generate charts enabled but no chart configuration provided".to_string(),
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.min_level, LogLevel::Info);
        assert!(config.output.console_enabled);
        assert!(!config.output.file_enabled);
    }

    #[test]
    fn test_builder_pattern() {
        let config = Config::new()
            .with_min_level(LogLevel::Debug)
            .with_json_format(true)
            .with_profiling(false);

        assert_eq!(config.min_level, LogLevel::Debug);
        assert!(config.output.json_format);
        assert!(!config.performance.profiling_enabled);
    }

    #[test]
    fn test_development_config() {
        let config = Config::development();
        assert_eq!(config.min_level, LogLevel::Debug);
        assert!(config.output.console_enabled);
        assert!(config.output.colored_output);
        assert!(config.performance.profiling_enabled);
        assert!(config.performance.component_tracking_enabled);
    }

    #[test]
    fn test_performance_analysis_config() {
        let config = Config::performance_analysis("/tmp/charts");
        assert_eq!(config.min_level, LogLevel::Debug);
        assert!(config.performance.profiling_enabled);
        assert!(config.performance.monitoring_enabled);
        assert!(config.performance.component_tracking_enabled);
        assert!(config.visualization.auto_generate_charts);
        assert!(config.visualization.output_directory.is_some());
    }

    #[test]
    fn test_validation() {
        let mut config = Config::new();
        config.output.file_enabled = true;
        config.output.file_path = None;
        assert!(config.validate().is_err());

        config.output.file_path = Some("test.log".into());
        assert!(config.validate().is_ok());

        // Test auto-generate charts validation
        config.visualization.auto_generate_charts = true;
        config.visualization.chart_config = None;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_nested_config_access() {
        let config = Config::new()
            .with_buffer_size(2048)
            .with_file_rotation(20 * 1024 * 1024, 10)
            .with_auto_generate_charts(true);

        assert_eq!(config.performance.buffer_size, 2048);
        assert_eq!(config.output.max_file_size, 20 * 1024 * 1024);
        assert_eq!(config.output.max_files, 10);
        assert!(config.visualization.auto_generate_charts);
    }
}
