use telelog::Logger;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Telelog Basic Example ===\n");

    // Create a basic logger
    let logger = Logger::new("basic_app");

    // Basic logging
    logger.info("Application started");
    logger.debug("Debug message (won't show with default config)");
    logger.warning("Warning message");
    logger.error("Error message");

    // Structured logging
    logger.info_with(
        "User action",
        &[
            ("user_id", "12345"),
            ("action", "login"),
            ("ip", "192.168.1.1"),
        ],
    );

    // Context management
    logger.add_context("request_id", "req_abc123");
    logger.add_context("session_id", "sess_xyz789");

    logger.info("Processing request"); // Includes context
    logger.info("Request completed"); // Includes context

    logger.clear_context();
    logger.info("Context cleared"); // No context

    // Performance profiling
    {
        let _guard = logger.profile("expensive_operation");
        // Simulate work
        std::thread::sleep(std::time::Duration::from_millis(50));
    } // Timing automatically logged when guard drops

    logger.info("Application finished");
    println!("\n=== Basic Example Complete ===");

    Ok(())
}
