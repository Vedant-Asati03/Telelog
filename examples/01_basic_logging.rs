//! Basic Logging Example
//! Demonstrates core logging functionality with different log levels.

use telelog::Logger;

fn main() {
    // Create logger
    let logger = Logger::new("basic_demo");
    
    // Different log levels
    logger.debug("Debug message");
    logger.info("Application started");
    logger.warning("This is a warning");
    logger.error("This is an error");
    logger.critical("This is critical");
    
    // Structured logging
    logger.info_with(
        "User action",
        &[
            ("user_id", "12345"),
            ("action", "login"),
            ("ip", "192.168.1.1"),
        ],
    );
    
    logger.info("Basic logging demo complete");
    println!("✅ Basic logging example finished");
}