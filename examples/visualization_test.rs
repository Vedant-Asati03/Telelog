use std::fs;
use telelog::visualization::{ChartConfig, ChartType, Direction, MermaidGenerator};
use telelog::{Config, Logger};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Telelog Visualization Testing ===\n");

    // Get output directory from command line args or use default
    let args: Vec<String> = std::env::args().collect();
    let charts_dir = args.get(1).map(String::as_str).unwrap_or("./test_charts");

    // Create logger with component tracking
    let config = Config::performance_analysis(charts_dir);
    let logger = Logger::with_config("viz_test", config);

    logger.info("Starting visualization testing");

    // Create a complex component hierarchy
    {
        let _app = logger.track_component("Application");
        logger.info("Application started");

        // Simulate web server components
        {
            let _web_server = logger.track_component("Web Server");
            logger.info("Web server initializing");

            {
                let _router = logger.track_component("Request Router");
                logger.info("Routing incoming request");
                std::thread::sleep(std::time::Duration::from_millis(5));
            }

            {
                let _middleware = logger.track_component("Middleware Stack");
                logger.info("Processing middleware");

                {
                    let _auth = logger.track_component("Authentication");
                    logger.info("Validating user token");
                    std::thread::sleep(std::time::Duration::from_millis(15));
                }

                {
                    let _rate_limit = logger.track_component("Rate Limiter");
                    logger.info("Checking rate limits");
                    std::thread::sleep(std::time::Duration::from_millis(3));
                }
            }
        }

        // Simulate business logic
        {
            let _business = logger.track_component("Business Logic");
            logger.info("Processing business logic");

            {
                let _validation = logger.track_component("Input Validation");
                logger.info("Validating input data");
                std::thread::sleep(std::time::Duration::from_millis(8));
            }

            {
                let _processing = logger.track_component("Data Processing");
                logger.info("Processing user data");
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
        }

        // Simulate database operations
        {
            let _db = logger.track_component("Database Layer");
            logger.info("Database operations starting");

            {
                let _connection = logger.track_component("DB Connection Pool");
                logger.info("Acquiring database connection");
                std::thread::sleep(std::time::Duration::from_millis(12));
            }

            {
                let _query = logger.track_component("Query Execution");
                logger.info("Executing SQL query");
                std::thread::sleep(std::time::Duration::from_millis(30));
            }
        }

        logger.info("Application completed");
    }

    // Get the component tracker
    let tracker = logger.component_tracker();

    println!("\n=== Generating Different Chart Types ===");

    // Get output directory from command line args or use default
    let args: Vec<String> = std::env::args().collect();
    let output_dir = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("./visualization_demo");

    // Create charts directory
    fs::create_dir_all(output_dir)?;

    // 1. Basic Flowchart
    println!("📊 Generating basic flowchart...");
    let basic_config = ChartConfig::new()
        .with_chart_type(ChartType::Flowchart)
        .with_direction(Direction::TopDown)
        .with_timing(true);

    let generator = MermaidGenerator::new(basic_config);
    let flowchart_path = format!("{}/flowchart", output_dir);
    generator
        .save_diagram(tracker, std::path::Path::new(&flowchart_path))
        .unwrap_or_else(|_| {
            // Fallback to just generating the diagram content if CLI is not available
            let flowchart = generator.generate_diagram(tracker).unwrap();
            fs::write(format!("{}/flowchart.mmd", output_dir), &flowchart).unwrap();
        });
    println!("✅ Flowchart saved to: {}/flowchart.mmd", output_dir);

    // 2. Timeline Chart
    println!("📈 Generating timeline chart...");
    let timeline_config = ChartConfig::new()
        .with_chart_type(ChartType::Timeline)
        .with_timing(true);

    let timeline_generator = MermaidGenerator::new(timeline_config);
    let timeline_path = format!("{}/timeline", output_dir);
    timeline_generator
        .save_diagram(tracker, std::path::Path::new(&timeline_path))
        .unwrap_or_else(|_| {
            // Fallback to just generating the diagram content if CLI is not available
            let timeline = timeline_generator.generate_diagram(tracker).unwrap();
            fs::write(format!("{}/timeline.mmd", output_dir), &timeline).unwrap();
        });
    println!("✅ Timeline saved to: {}/timeline.mmd", output_dir);

    // 3. Gantt Chart
    println!("📊 Generating Gantt chart...");
    let gantt_config = ChartConfig::new().with_chart_type(ChartType::Gantt);

    let gantt_generator = MermaidGenerator::new(gantt_config);
    let gantt_path = format!("{}/gantt", output_dir);
    gantt_generator
        .save_diagram(tracker, std::path::Path::new(&gantt_path))
        .unwrap_or_else(|_| {
            // Fallback to just generating the diagram content if CLI is not available
            let gantt = gantt_generator.generate_diagram(tracker).unwrap();
            fs::write(format!("{}/gantt.mmd", output_dir), &gantt).unwrap();
        });
    println!("✅ Gantt chart saved to: {}/gantt.mmd", output_dir);

    // Display sample content
    println!("\n=== Sample Flowchart Content ===");
    let sample_generator =
        MermaidGenerator::new(ChartConfig::new().with_chart_type(ChartType::Flowchart));
    let sample_flowchart = sample_generator.generate_diagram(tracker)?;
    println!("{}", sample_flowchart);

    println!("\n=== Generated Files ===");
    println!("📁 {}/", output_dir);
    println!("  ├── flowchart.mmd (Basic flowchart)");
    println!("  ├── timeline.mmd (Timeline view)");
    println!("  ├── gantt.mmd (Gantt chart)");

    println!("\n🎨 View these files in:");
    println!("  • Mermaid Live Editor: https://mermaid.live/");
    println!("  • VS Code with Mermaid extension");
    println!("  • GitHub (supports Mermaid in markdown)");

    println!("\n💡 Usage:");
    println!("  cargo run --example visualization_test [output_directory]");
    println!("  Example: cargo run --example visualization_test /tmp/my_charts");

    println!("\n=== Visualization Testing Complete ===");

    Ok(())
}
