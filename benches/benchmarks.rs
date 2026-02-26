//! Comprehensive benchmarks for all telelog use-case scenarios.
//!
//! # Benchmark Groups
//!
//! 1. **basic_logging**        – plain messages at every log level
//! 2. **structured_logging**   – key/value payloads of varying sizes
//! 3. **level_filtering**      – messages filtered-out by min-level (near-zero path)
//! 4. **context_management**   – add/remove/clear context, scoped guard
//! 5. **performance_profiling**– ProfileGuard creation + drop (timing overhead)
//! 6. **component_tracking**   – ComponentGuard lifecycle, nested components
//! 7. **output_modes**         – buffered vs unbuffered, JSON vs plain text
//! 8. **file_output**          – write to a temp file (real I/O path)
//! 9. **logger_init**          – logger construction with various configs
//! 10. **high_volume**         – tight loops measuring throughput
//! 11. **concurrent_logging**  – multi-threaded shared logger
//! 12. **visualization**       – Mermaid diagram generation from tracked components

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;
use std::thread;
use telelog::{
    ChartConfig, ChartType, ComponentTracker, Config, LogLevel, Logger, MermaidGenerator,
};
use tempfile::tempdir;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Build a logger that writes nowhere (no console, no file). Minimises I/O noise
/// and measures the pure serialisation / routing overhead of the library.
fn silent_logger(name: &str) -> Logger {
    let config = Config::new()
        .with_console_output(false)
        .with_min_level(LogLevel::Debug);
    Logger::with_config(name, config)
}

/// Build a logger with buffered in-memory output (no console / file).
fn buffered_silent_logger(name: &str) -> Logger {
    let config = Config::new()
        .with_console_output(false)
        .with_min_level(LogLevel::Debug)
        .with_buffering(true)
        .with_buffer_size(1024);
    Logger::with_config(name, config)
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Basic logging – one plain message per level
// ─────────────────────────────────────────────────────────────────────────────

fn bench_basic_logging(c: &mut Criterion) {
    let logger = silent_logger("bench_basic");

    let mut group = c.benchmark_group("basic_logging");

    group.bench_function("debug", |b| b.iter(|| logger.debug("debug message")));
    group.bench_function("info", |b| b.iter(|| logger.info("info message")));
    group.bench_function("warning", |b| b.iter(|| logger.warning("warning message")));
    group.bench_function("error", |b| b.iter(|| logger.error("error message")));
    group.bench_function("critical", |b| {
        b.iter(|| logger.critical("critical message"))
    });

    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Structured logging – key/value payloads of 1, 5, and 10 fields
// ─────────────────────────────────────────────────────────────────────────────

fn bench_structured_logging(c: &mut Criterion) {
    let logger = silent_logger("bench_structured");

    let data_1 = [("user_id", "1234")];
    let data_5 = [
        ("user_id", "1234"),
        ("session", "abcdef"),
        ("action", "login"),
        ("ip", "10.0.0.1"),
        ("region", "us-east-1"),
    ];
    let data_10 = [
        ("user_id", "1234"),
        ("session", "abcdef"),
        ("action", "login"),
        ("ip", "10.0.0.1"),
        ("region", "us-east-1"),
        ("latency_ms", "42"),
        ("status", "200"),
        ("service", "auth"),
        ("trace_id", "xyz987"),
        ("span_id", "span001"),
    ];

    let mut group = c.benchmark_group("structured_logging");

    group.bench_function("1_field", |b| {
        b.iter(|| logger.info_with("user event", &data_1))
    });
    group.bench_function("5_fields", |b| {
        b.iter(|| logger.info_with("user event", &data_5))
    });
    group.bench_function("10_fields", |b| {
        b.iter(|| logger.info_with("user event", &data_10))
    });

    // Generic log_with variant
    group.bench_function("log_with_5_fields", |b| {
        b.iter(|| logger.log_with(LogLevel::Info, "user event", &data_5))
    });

    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Level filtering – messages silenced before any serialisation
// ─────────────────────────────────────────────────────────────────────────────

fn bench_level_filtering(c: &mut Criterion) {
    // Logger that only accepts Warning and above so Debug/Info are filtered.
    let config = Config::new()
        .with_console_output(false)
        .with_min_level(LogLevel::Warning);
    let logger = Logger::with_config("bench_filter", config);

    let mut group = c.benchmark_group("level_filtering");

    group.bench_function("filtered_debug", |b| {
        b.iter(|| logger.debug("silenced debug"))
    });
    group.bench_function("filtered_info", |b| b.iter(|| logger.info("silenced info")));
    group.bench_function("passing_warning", |b| {
        b.iter(|| logger.warning("passing warning"))
    });
    group.bench_function("passing_error", |b| {
        b.iter(|| logger.error("passing error"))
    });

    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Context management
// ─────────────────────────────────────────────────────────────────────────────

fn bench_context_management(c: &mut Criterion) {
    let logger = silent_logger("bench_context");

    let mut group = c.benchmark_group("context_management");

    // Add a single context key
    group.bench_function("add_context", |b| {
        b.iter(|| {
            logger.add_context("request_id", "abc-123");
            logger.clear_context();
        })
    });

    // Log while context is populated
    group.bench_function("log_with_context_1_key", |b| {
        logger.add_context("request_id", "abc-123");
        b.iter(|| logger.info("processing"));
        logger.clear_context();
    });

    group.bench_function("log_with_context_5_keys", |b| {
        logger.add_context("request_id", "abc-123");
        logger.add_context("user_id", "u-456");
        logger.add_context("session", "s-789");
        logger.add_context("region", "eu-west-1");
        logger.add_context("service", "api");
        b.iter(|| logger.info("processing"));
        logger.clear_context();
    });

    // Scoped context guard – creation + drop
    group.bench_function("context_guard_create_drop", |b| {
        b.iter(|| {
            let _guard = logger.with_context("request_id", "abc-123");
            // guard drops here, context removed automatically
        })
    });

    // Log inside a context guard scope
    group.bench_function("log_inside_context_guard", |b| {
        b.iter(|| {
            let _guard = logger.with_context("request_id", "abc-123");
            logger.info("inside guard");
        })
    });

    // Remove a specific key
    group.bench_function("remove_context", |b| {
        b.iter(|| {
            logger.add_context("key", "value");
            logger.remove_context("key");
        })
    });

    // Clear all context
    group.bench_function("clear_context", |b| {
        b.iter(|| {
            logger.add_context("a", "1");
            logger.add_context("b", "2");
            logger.add_context("c", "3");
            logger.clear_context();
        })
    });

    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Performance profiling – ProfileGuard overhead
// ─────────────────────────────────────────────────────────────────────────────

fn bench_performance_profiling(c: &mut Criterion) {
    let logger = silent_logger("bench_profiling");

    let mut group = c.benchmark_group("performance_profiling");

    // Immediate drop – measures the guard's own overhead, not the work inside
    group.bench_function("profile_guard_create_drop", |b| {
        b.iter(|| {
            let _guard = logger.profile("instant_op");
            // drops immediately
        })
    });

    // Read elapsed without dropping
    group.bench_function("profile_guard_elapsed", |b| {
        b.iter(|| {
            let guard = logger.profile("instant_op");
            let _ = guard.elapsed();
        })
    });

    // Nested profiling (two levels deep)
    group.bench_function("nested_profiles_2_deep", |b| {
        b.iter(|| {
            let _outer = logger.profile("outer");
            let _inner = logger.profile("inner");
        })
    });

    // Three levels deep
    group.bench_function("nested_profiles_3_deep", |b| {
        b.iter(|| {
            let _a = logger.profile("a");
            let _b = logger.profile("b");
            let _c = logger.profile("c");
        })
    });

    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Component tracking
// ─────────────────────────────────────────────────────────────────────────────

fn bench_component_tracking(c: &mut Criterion) {
    let logger = silent_logger("bench_component");

    let mut group = c.benchmark_group("component_tracking");

    // Single component guard (start + complete)
    group.bench_function("component_guard_create_drop", |b| {
        b.iter(|| {
            let _guard = logger.track_component("op");
        })
    });

    // Accessing the component tracker directly
    group.bench_function("component_tracker_len", |b| {
        b.iter(|| {
            let tracker = logger.component_tracker();
            let _ = tracker.get_components().len();
        })
    });

    // Nested components (2 levels)
    group.bench_function("nested_components_2_deep", |b| {
        b.iter(|| {
            let _parent = logger.track_component("parent");
            let _child = logger.track_component("child");
        })
    });

    // Sequential components – simulate pipeline stages
    group.bench_function("sequential_5_components", |b| {
        b.iter(|| {
            for stage in &["parse", "validate", "transform", "persist", "respond"] {
                let _g = logger.track_component(stage);
            }
        })
    });

    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. Output mode comparisons
// ─────────────────────────────────────────────────────────────────────────────

fn bench_output_modes(c: &mut Criterion) {
    // No output (null sink) – pure overhead
    let logger_null = silent_logger("bench_null");

    // Buffered no-console logger
    let logger_buffered = buffered_silent_logger("bench_buffered");

    // JSON format, no console
    let config_json = Config::new()
        .with_console_output(false)
        .with_json_format(true)
        .with_min_level(LogLevel::Debug);
    let logger_json = Logger::with_config("bench_json", config_json);

    // Plain text (default), no console
    let config_plain = Config::new()
        .with_console_output(false)
        .with_json_format(false)
        .with_min_level(LogLevel::Debug);
    let logger_plain = Logger::with_config("bench_plain", config_plain);

    let mut group = c.benchmark_group("output_modes");

    group.bench_function("null_sink_info", |b| {
        b.iter(|| logger_null.info("no output"))
    });
    group.bench_function("buffered_info", |b| {
        b.iter(|| logger_buffered.info("buffered"))
    });
    group.bench_function("json_format_info", |b| {
        b.iter(|| logger_json.info("json message"))
    });
    group.bench_function("plain_text_info", |b| {
        b.iter(|| logger_plain.info("plain message"))
    });

    // Structured: JSON vs plain
    let data = [("service", "api"), ("latency_ms", "12"), ("status", "200")];
    group.bench_function("json_format_structured", |b| {
        b.iter(|| logger_json.info_with("event", &data))
    });
    group.bench_function("plain_text_structured", |b| {
        b.iter(|| logger_plain.info_with("event", &data))
    });

    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. File output – real filesystem I/O
// ─────────────────────────────────────────────────────────────────────────────

fn bench_file_output(c: &mut Criterion) {
    let dir = tempdir().expect("tempdir");

    // Plain text file
    let plain_path = dir.path().join("bench_plain.log");
    let config_plain_file = Config::new()
        .with_console_output(false)
        .with_file_output(plain_path.to_str().unwrap())
        .with_json_format(false)
        .with_min_level(LogLevel::Debug);
    let logger_plain_file = Logger::with_config("bench_file_plain", config_plain_file);

    // JSON file
    let json_path = dir.path().join("bench_json.log");
    let config_json_file = Config::new()
        .with_console_output(false)
        .with_file_output(json_path.to_str().unwrap())
        .with_json_format(true)
        .with_min_level(LogLevel::Debug);
    let logger_json_file = Logger::with_config("bench_file_json", config_json_file);

    // Rotating file
    let rotating_path = dir.path().join("bench_rotating.log");
    let config_rotating = Config::new()
        .with_console_output(false)
        .with_file_output(rotating_path.to_str().unwrap())
        .with_file_rotation(1024 * 1024, 3) // 1 MB, 3 files
        .with_min_level(LogLevel::Debug);
    let logger_rotating = Logger::with_config("bench_rotating", config_rotating);

    // Buffered + file
    let buf_path = dir.path().join("bench_buffered.log");
    let config_buf_file = Config::new()
        .with_console_output(false)
        .with_file_output(buf_path.to_str().unwrap())
        .with_buffering(true)
        .with_buffer_size(256)
        .with_min_level(LogLevel::Debug);
    let logger_buf_file = Logger::with_config("bench_buf_file", config_buf_file);

    let mut group = c.benchmark_group("file_output");

    group.bench_function("plain_text_file", |b| {
        b.iter(|| logger_plain_file.info("plain log to file"))
    });
    group.bench_function("json_file", |b| {
        b.iter(|| logger_json_file.info("json log to file"))
    });
    group.bench_function("rotating_file", |b| {
        b.iter(|| logger_rotating.info("rotating log"))
    });
    group.bench_function("buffered_file", |b| {
        b.iter(|| logger_buf_file.info("buffered file log"))
    });

    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. Logger initialisation – construction cost of different configs
// ─────────────────────────────────────────────────────────────────────────────

fn bench_logger_init(c: &mut Criterion) {
    let mut group = c.benchmark_group("logger_init");

    group.bench_function("default", |b| b.iter(|| Logger::new("init_bench")));

    group.bench_function("development_preset", |b| {
        b.iter(|| {
            let cfg = Config::development();
            Logger::with_config("init_bench", cfg)
        })
    });

    group.bench_function("silent_config", |b| {
        b.iter(|| {
            let cfg = Config::new().with_console_output(false);
            Logger::with_config("init_bench", cfg)
        })
    });

    group.bench_function("with_buffering_config", |b| {
        b.iter(|| {
            let cfg = Config::new()
                .with_console_output(false)
                .with_buffering(true)
                .with_buffer_size(512);
            Logger::with_config("init_bench", cfg)
        })
    });

    group.bench_function("clone_existing", |b| {
        let logger = Logger::new("original");
        b.iter(|| logger.clone())
    });

    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. High-volume throughput – tight loop, measures messages-per-second
// ─────────────────────────────────────────────────────────────────────────────

fn bench_high_volume(c: &mut Criterion) {
    let logger = silent_logger("bench_throughput");
    let logger_buffered = buffered_silent_logger("bench_throughput_buf");

    let mut group = c.benchmark_group("high_volume");

    // Measure bytes / iteration for throughput estimation
    const BATCH: u64 = 100;
    group.throughput(Throughput::Elements(BATCH));

    group.bench_function("100_plain_messages", |b| {
        b.iter(|| {
            for i in 0..BATCH {
                logger.info(&format!("message {}", i));
            }
        })
    });

    group.bench_function("100_buffered_messages", |b| {
        b.iter(|| {
            for i in 0..BATCH {
                logger_buffered.info(&format!("message {}", i));
            }
        })
    });

    group.bench_function("100_structured_messages", |b| {
        b.iter(|| {
            for i in 0..BATCH {
                logger.info_with(
                    "event",
                    &[
                        ("idx", &i.to_string()),
                        ("service", "bench"),
                        ("ok", "true"),
                    ],
                );
            }
        })
    });

    group.bench_function("100_mixed_levels", |b| {
        b.iter(|| {
            for i in 0..BATCH {
                match i % 5 {
                    0 => logger.debug("debug"),
                    1 => logger.info("info"),
                    2 => logger.warning("warn"),
                    3 => logger.error("error"),
                    _ => logger.critical("critical"),
                }
            }
        })
    });

    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 11. Concurrent logging – shared Arc<Logger> across N threads
// ─────────────────────────────────────────────────────────────────────────────

fn bench_concurrent_logging(c: &mut Criterion) {
    let logger = Arc::new(silent_logger("bench_concurrent"));

    let mut group = c.benchmark_group("concurrent_logging");

    for thread_count in [2usize, 4, 8] {
        group.bench_with_input(
            BenchmarkId::new("threads", thread_count),
            &thread_count,
            |b, &n| {
                b.iter(|| {
                    let handles: Vec<_> = (0..n)
                        .map(|i| {
                            let l = Arc::clone(&logger);
                            thread::spawn(move || {
                                l.info_with(
                                    "concurrent event",
                                    &[("thread", &i.to_string()), ("msg", "hello")],
                                );
                            })
                        })
                        .collect();
                    for h in handles {
                        h.join().unwrap();
                    }
                })
            },
        );
    }

    // Shared logger cloned to each thread (cheap Arc clone)
    group.bench_function("8_threads_cloned_logger", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..8)
                .map(|i| {
                    let l = (*logger).clone(); // Clone the Logger (Arc internally)
                    thread::spawn(move || {
                        l.info_with("event", &[("t", &i.to_string())]);
                    })
                })
                .collect();
            for h in handles {
                h.join().unwrap();
            }
        })
    });

    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 12. Visualization – Mermaid diagram generation
// ─────────────────────────────────────────────────────────────────────────────

fn bench_visualization(c: &mut Criterion) {
    let mut group = c.benchmark_group("visualization");

    // Helper to build a populated ComponentTracker with N top-level components
    fn make_tracker(n: usize) -> Arc<ComponentTracker> {
        let logger = silent_logger("vis_bench");
        for i in 0..n {
            let _g = logger.track_component(&format!("component_{}", i));
        }
        logger.component_tracker().clone()
    }

    // Flowchart generation
    group.bench_function("flowchart_5_components", |b| {
        let tracker = make_tracker(5);
        let cfg = ChartConfig::new().with_chart_type(ChartType::Flowchart);
        let gen = MermaidGenerator::new(cfg);
        b.iter(|| {
            let _ = gen.generate_diagram(&tracker).unwrap();
        })
    });

    group.bench_function("flowchart_20_components", |b| {
        let tracker = make_tracker(20);
        let cfg = ChartConfig::new().with_chart_type(ChartType::Flowchart);
        let gen = MermaidGenerator::new(cfg);
        b.iter(|| {
            let _ = gen.generate_diagram(&tracker).unwrap();
        })
    });

    // Timeline generation
    group.bench_function("timeline_5_components", |b| {
        let tracker = make_tracker(5);
        let cfg = ChartConfig::new()
            .with_chart_type(ChartType::Timeline)
            .with_timing(true);
        let gen = MermaidGenerator::new(cfg);
        b.iter(|| {
            let _ = gen.generate_diagram(&tracker).unwrap();
        })
    });

    group.bench_function("timeline_20_components", |b| {
        let tracker = make_tracker(20);
        let cfg = ChartConfig::new()
            .with_chart_type(ChartType::Timeline)
            .with_timing(true);
        let gen = MermaidGenerator::new(cfg);
        b.iter(|| {
            let _ = gen.generate_diagram(&tracker).unwrap();
        })
    });

    // Gantt generation
    group.bench_function("gantt_5_components", |b| {
        let tracker = make_tracker(5);
        let cfg = ChartConfig::new().with_chart_type(ChartType::Gantt);
        let gen = MermaidGenerator::new(cfg);
        b.iter(|| {
            let _ = gen.generate_diagram(&tracker).unwrap();
        })
    });

    // via Logger::generate_visualization shortcut
    group.bench_function("logger_generate_visualization_flowchart", |b| {
        let logger = silent_logger("vis_logger");
        for i in 0..10 {
            let _g = logger.track_component(&format!("op_{}", i));
        }
        b.iter(|| {
            let _ = logger
                .generate_visualization(ChartType::Flowchart, None)
                .unwrap();
        })
    });

    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// Register all groups
// ─────────────────────────────────────────────────────────────────────────────

criterion_group!(
    benches,
    bench_basic_logging,
    bench_structured_logging,
    bench_level_filtering,
    bench_context_management,
    bench_performance_profiling,
    bench_component_tracking,
    bench_output_modes,
    bench_file_output,
    bench_logger_init,
    bench_high_volume,
    bench_concurrent_logging,
    bench_visualization,
);
criterion_main!(benches);
