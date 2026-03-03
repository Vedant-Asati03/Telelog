use crate::component::{ComponentGuard, ComponentTracker};
use crate::output::{LogRecord, OutputDestination};

pub struct OutputPipeline(pub Arc<dyn OutputDestination>);
use crate::{config::Config, context::Context, level::LogLevel};

use arc_swap::ArcSwap;
use std::cell::RefCell;
use std::fmt::Write;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

#[cfg(feature = "system-monitor")]
use crate::monitor::SystemMonitor;

thread_local! {
    static TIMESTAMP_BUF: RefCell<String> = RefCell::new(String::with_capacity(35));
}

pub struct Logger {
    name: Arc<str>,
    min_level: AtomicU8,
    config: ArcSwap<Config>,
    output: ArcSwap<OutputPipeline>,
    context: Arc<Context>,
    component_tracker: Arc<ComponentTracker>,
    #[cfg(feature = "system-monitor")]
    system_monitor: Arc<parking_lot::RwLock<SystemMonitor>>,
}

impl Logger {
    pub fn new(name: &str) -> Self {
        Self::with_config(name, Config::default())
    }

    pub fn with_config(name: &str, config: Config) -> Self {
        config.validate().expect("Invalid Logger Configuration");
        let output = build_output_pipeline(&config);
        let output = Arc::new(OutputPipeline(output));

        Self {
            name: Arc::from(name),
            min_level: AtomicU8::new(config.min_level as u8),
            config: ArcSwap::from_pointee(config),
            output: ArcSwap::from(output),
            context: Arc::new(Context::new()),
            component_tracker: Arc::new(ComponentTracker::new()),
            #[cfg(feature = "system-monitor")]
            system_monitor: Arc::new(parking_lot::RwLock::new(SystemMonitor::new())),
        }
    }

    pub fn debug(&self, m: &str) {
        self.log(LogLevel::Debug, m, None);
    }
    pub fn info(&self, m: &str) {
        self.log(LogLevel::Info, m, None);
    }
    pub fn warning(&self, m: &str) {
        self.log(LogLevel::Warning, m, None);
    }
    pub fn error(&self, m: &str) {
        self.log(LogLevel::Error, m, None);
    }
    pub fn critical(&self, m: &str) {
        self.log(LogLevel::Critical, m, None);
    }

    pub fn debug_with(&self, m: &str, d: &[(&str, &str)]) {
        self.log(LogLevel::Debug, m, Some(d));
    }
    pub fn info_with(&self, m: &str, d: &[(&str, &str)]) {
        self.log(LogLevel::Info, m, Some(d));
    }
    pub fn warning_with(&self, m: &str, d: &[(&str, &str)]) {
        self.log(LogLevel::Warning, m, Some(d));
    }
    pub fn error_with(&self, m: &str, d: &[(&str, &str)]) {
        self.log(LogLevel::Error, m, Some(d));
    }
    pub fn critical_with(&self, m: &str, d: &[(&str, &str)]) {
        self.log(LogLevel::Critical, m, Some(d));
    }

    pub fn log_with(&self, level: LogLevel, message: &str, data: &[(&str, &str)]) {
        self.log(level, message, Some(data));
    }

    pub fn add_context(&self, key: &str, value: &str) {
        self.context.add(key, value);
    }
    pub fn remove_context(&self, key: &str) {
        self.context.remove(key);
    }
    pub fn clear_context(&self) {
        self.context.clear();
    }

    pub fn with_context(&self, key: &str, value: &str) -> crate::context::ContextGuard {
        self.context.add(key, value);
        crate::context::ContextGuard::new(key.to_string(), Arc::clone(&self.context))
    }

    #[inline]
    fn log(&self, level: LogLevel, message: &str, data: Option<&[(&str, &str)]>) {
        if (level as u8) < self.min_level.load(Ordering::Relaxed) {
            return;
        }

        TIMESTAMP_BUF.with(|buf| {
            let mut b = buf.borrow_mut();
            b.clear();
            let _ = write!(b, "{}", chrono::Utc::now().to_rfc3339());

            let record = LogRecord {
                timestamp: &b,
                level,
                logger: &self.name,
                message,
                context: &self.context.data.read(),
                data,
            };

            if let Err(e) = self.output.load().0.write(&record) {
                eprintln!("[Telelog] Write error: {}", e);
            }
        });
    }

    pub fn set_config(&self, config: Config) {
        if config.validate().is_ok() {
            self.min_level
                .store(config.min_level as u8, Ordering::Release);
            let new_pipeline = Arc::new(OutputPipeline(build_output_pipeline(&config)));
            self.output.store(new_pipeline);
            self.config.store(Arc::new(config));
        }
    }

    pub fn get_config(&self) -> Config {
        (**self.config.load()).clone()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn get_component_tracker(&self) -> &ComponentTracker {
        &self.component_tracker
    }
    pub fn component_tracker(&self) -> &Arc<ComponentTracker> {
        &self.component_tracker
    }

    pub fn profile(&self, op: &str) -> crate::profile::ProfileGuard {
        crate::profile::ProfileGuard::new(op, self.clone())
    }

    pub fn track_component(&self, name: &str) -> ComponentGuard {
        #[cfg(feature = "system-monitor")]
        return ComponentGuard::new_with_monitor(
            name,
            Arc::clone(&self.component_tracker),
            Arc::clone(&self.system_monitor),
        );
        #[cfg(not(feature = "system-monitor"))]
        return ComponentGuard::new(name, Arc::clone(&self.component_tracker));
    }

    pub fn generate_visualization(
        &self,
        chart_type: crate::visualization::ChartType,
        output_path: Option<&str>,
    ) -> Result<String, String> {
        let generator = crate::visualization::MermaidGenerator::new(
            crate::visualization::ChartConfig::new().with_chart_type(chart_type),
        );
        let diagram = generator.generate_diagram(&self.component_tracker)?;
        if let Some(path) = output_path {
            std::fs::write(path, &diagram).map_err(|e| e.to_string())?;
        }
        Ok(diagram)
    }

    #[cfg(feature = "system-monitor")]
    pub fn system_monitor(&self) -> &Arc<parking_lot::RwLock<SystemMonitor>> {
        &self.system_monitor
    }
}

impl Clone for Logger {
    fn clone(&self) -> Self {
        Self {
            name: Arc::clone(&self.name),
            min_level: AtomicU8::new(self.min_level.load(Ordering::Relaxed)),
            config: ArcSwap::from(self.config.load_full()),
            output: ArcSwap::from(self.output.load_full()),
            context: Arc::clone(&self.context),
            component_tracker: Arc::clone(&self.component_tracker),
            #[cfg(feature = "system-monitor")]
            system_monitor: Arc::clone(&self.system_monitor),
        }
    }
}

pub(crate) fn build_output_pipeline(config: &Config) -> Arc<dyn OutputDestination> {
    use crate::output::{
        BufferedOutput, ConsoleOutput, FileOutput, MultiOutput, RotatingFileOutput,
    };
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
                        if let Ok(file) = FileOutput::new(file_path, config.output.json_format) {
                            multi_output = multi_output.add_output(Box::new(file));
                        }
                    }
                }
            } else if let Ok(file) = FileOutput::new(file_path, config.output.json_format) {
                multi_output = multi_output.add_output(Box::new(file));
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
        match crate::async_output::AsyncOutput::new(output.clone()) {
            Ok(async_output) => Arc::new(async_output) as Arc<dyn OutputDestination>,
            Err(e) => {
                eprintln!("Failed to create async output: {}", e);
                output
            }
        }
    } else {
        output
    };

    output
}
