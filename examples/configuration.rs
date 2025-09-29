use telelog::{ChartConfig, ChartType, Config, LogLevel, Logger, OutputFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Telelog Configuration Example ===\n");

    // Preset Configurations

    println!("🔧 Development Configuration:");
    let dev_config = Config::development();
    let dev_logger = Logger::with_config("dev_app", dev_config);
    dev_logger.debug("Debug logging enabled in development");
    dev_logger.info("Development logger ready");
    println!();

    println!("🏭 Production Configuration:");
    let prod_config = Config::production("production.log");
    let prod_logger = Logger::with_config("prod_app", prod_config);
    prod_logger.info("Production logger configured");
    println!();

    println!("📊 Performance Analysis Configuration:");
    // Get output directory from command line args or use default
    let args: Vec<String> = std::env::args().collect();
    let analysis_charts_dir = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("./analysis_charts");
    let custom_charts_dir = args.get(2).map(String::as_str).unwrap_or("./custom_charts");

    let perf_config = Config::performance_analysis(analysis_charts_dir);
    let perf_logger = Logger::with_config("perf_app", perf_config);
    perf_logger.info("Performance analysis logger ready");
    {
        let _component = perf_logger.track_component("Analysis Component");
        perf_logger.info("Running performance analysis");
    }
    println!();

    // Custom Configuration
    println!("🛠️ Custom Configuration:");

    let chart_config = ChartConfig::new()
        .with_chart_type(ChartType::Timeline)
        .with_timing(true)
        .with_memory(true)
        .with_output_format(OutputFormat::Svg);

    let custom_config = Config::new()
        .with_min_level(LogLevel::Debug)
        .with_console_output(true)
        .with_colored_output(true)
        .with_file_output("custom.log")
        .with_profiling(true)
        .with_monitoring(true)
        .with_component_tracking(true)
        .with_chart_config(chart_config)
        .with_auto_generate_charts(true)
        .with_chart_output_directory(custom_charts_dir);

    println!("Custom config validation: {:?}", custom_config.validate());

    let custom_logger = Logger::with_config("custom_app", custom_config);
    custom_logger.info("Custom configuration applied");

    // Demonstrate features
    custom_logger.add_context("config_demo", "true");

    {
        let _profile = custom_logger.profile("custom_operation");
        let _component = custom_logger.track_component("Custom Component");
        custom_logger.info("Running with custom configuration");
        std::thread::sleep(std::time::Duration::from_millis(25));
    }

    custom_logger.info("Custom configuration demo complete");
    println!("\n📁 Check generated files:");
    println!("  - production.log (production logs)");
    println!("  - custom.log (custom logs)");
    println!("  - {}/ (performance charts)", analysis_charts_dir);
    println!("  - {}/ (custom charts)", custom_charts_dir);

    println!("\n💡 Usage:");
    println!("  cargo run --example configuration [analysis_charts_dir] [custom_charts_dir]");
    println!("  Example: cargo run --example configuration /tmp/analysis /tmp/custom");

    println!("\n=== Configuration Example Complete ===");

    Ok(())
}
