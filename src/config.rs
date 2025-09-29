//! Configuration management for telelog

use crate::level::LogLevel;
use crate::visualization::{ChartConfig, ChartType};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Output configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Enable console output
    pub console_enabled: bool,
    /// Enable colored console output
    pub colored_output: bool,
    /// Enable file output
    pub file_enabled: bool,
    /// File path for log output
    pub file_path: Option<PathBuf>,
    /// Use JSON format for structured logging
    pub json_format: bool,
    /// Maximum file size before rotation (in bytes)
    pub max_file_size: u64,
    /// Number of rotated files to keep
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
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_files: 5,
        }
    }
}

/// Performance and monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable performance profiling
    pub profiling_enabled: bool,
    /// Enable system monitoring
    pub monitoring_enabled: bool,
    /// Enable component tracking
    pub component_tracking_enabled: bool,
    /// Buffer size for async logging
    pub buffer_size: usize,
    /// Enable buffered output
    pub buffering_enabled: bool,
    /// Enable async logging (requires async feature)
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

/// Visualization configuration settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VisualizationConfig {
    /// Chart configuration for visualization
    pub chart_config: Option<ChartConfig>,
    /// Auto-generate charts on component tracking completion
    pub auto_generate_charts: bool,
    /// Default output directory for generated charts
    pub output_directory: Option<PathBuf>,
}

/// Main configuration for telelog logger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Minimum log level to output
    pub min_level: LogLevel,
    /// Output configuration
    pub output: OutputConfig,
    /// Performance and monitoring configuration
    pub performance: PerformanceConfig,
    /// Visualization configuration
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
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set minimum log level
    pub fn with_min_level(mut self, level: LogLevel) -> Self {
        self.min_level = level;
        self
    }

    /// Enable or disable console output
    pub fn with_console_output(mut self, enabled: bool) -> Self {
        self.output.console_enabled = enabled;
        self
    }

    /// Enable file output with specified path
    pub fn with_file_output<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.output.file_enabled = true;
        self.output.file_path = Some(path.into());
        self
    }

    /// Enable or disable JSON format
    pub fn with_json_format(mut self, enabled: bool) -> Self {
        self.output.json_format = enabled;
        self
    }

    /// Enable or disable colored output
    pub fn with_colored_output(mut self, enabled: bool) -> Self {
        self.output.colored_output = enabled;
        self
    }

    /// Enable or disable performance profiling
    pub fn with_profiling(mut self, enabled: bool) -> Self {
        self.performance.profiling_enabled = enabled;
        self
    }

    /// Enable or disable system monitoring
    pub fn with_monitoring(mut self, enabled: bool) -> Self {
        self.performance.monitoring_enabled = enabled;
        self
    }

    /// Set buffer size for async logging
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.performance.buffer_size = size;
        self
    }

    /// Set file rotation parameters
    pub fn with_file_rotation(mut self, max_size: u64, max_files: u32) -> Self {
        self.output.max_file_size = max_size;
        self.output.max_files = max_files;
        self
    }

    /// Enable async logging (requires async feature)
    #[cfg(feature = "async")]
    pub fn with_async(mut self, enabled: bool) -> Self {
        self.performance.async_enabled = enabled;
        self
    }

    /// Enable buffered output
    pub fn with_buffering(mut self, enabled: bool) -> Self {
        self.performance.buffering_enabled = enabled;
        self
    }

    /// Enable component tracking
    pub fn with_component_tracking(mut self, enabled: bool) -> Self {
        self.performance.component_tracking_enabled = enabled;
        self
    }

    /// Set chart configuration for visualization
    pub fn with_chart_config(mut self, config: ChartConfig) -> Self {
        self.visualization.chart_config = Some(config);
        self
    }

    /// Enable auto-generation of charts
    pub fn with_auto_generate_charts(mut self, enabled: bool) -> Self {
        self.visualization.auto_generate_charts = enabled;
        self
    }

    /// Set output directory for generated charts
    pub fn with_chart_output_directory<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.visualization.output_directory = Some(path.into());
        self
    }

    /// Create a development configuration (debug level, colored console)
    pub fn development() -> Self {
        Self::new()
            .with_min_level(LogLevel::Debug)
            .with_console_output(true)
            .with_colored_output(true)
            .with_profiling(true)
            .with_component_tracking(true)
    }

    /// Create a production configuration (info level, JSON format, file output)
    pub fn production<P: Into<PathBuf>>(log_file: P) -> Self {
        Self::new()
            .with_min_level(LogLevel::Info)
            .with_console_output(false)
            .with_file_output(log_file)
            .with_json_format(true)
            .with_colored_output(false)
            .with_monitoring(true)
    }

    /// Create a performance analysis configuration with visualization
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

    /// Validate the configuration
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
