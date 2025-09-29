use telelog::{Config, Logger};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Telelog Component Tracking Example ===\n");

    // Get output directory from command line args or use default
    let args: Vec<String> = std::env::args().collect();
    let charts_dir = args.get(1).map(String::as_str).unwrap_or("./charts");

    // Create logger with component tracking enabled
    let config = Config::performance_analysis(charts_dir);
    let logger = Logger::with_config("component_app", config);

    logger.info("Starting application with component tracking");

    // Track main application component
    {
        let _app = logger.track_component("Application");
        logger.info("Application component started");

        // Track user service
        {
            let _user_service = logger.track_component("User Service");
            logger.info("Processing user request");

            // Track database operations
            {
                let _database = logger.track_component("Database");
                logger.info("Querying user data");
                std::thread::sleep(std::time::Duration::from_millis(30));
                logger.info("User data retrieved");
            }

            // Track authentication
            {
                let _auth = logger.track_component("Authentication");
                logger.info("Validating credentials");
                std::thread::sleep(std::time::Duration::from_millis(20));
                logger.info("Authentication successful");
            }

            logger.info("User service request completed");
        }

        // Track notification service
        {
            let _notification = logger.track_component("Notification Service");
            logger.info("Sending welcome notification");
            std::thread::sleep(std::time::Duration::from_millis(15));
            logger.info("Notification sent");
        }

        logger.info("Application component finished");
    }

    logger.info("Component tracking complete");
    println!(
        "📊 Check '{}' directory for generated dependency charts!",
        charts_dir
    );
    println!("\n💡 Usage:");
    println!("  cargo run --example component_tracking [charts_directory]");
    println!("  Example: cargo run --example component_tracking /tmp/my_charts");
    println!("\n=== Component Tracking Example Complete ===");

    Ok(())
}
