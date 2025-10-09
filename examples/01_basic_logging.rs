//! Basic logging with different log levels and structured data.

use telelog::Logger;

fn main() {
    let logger = Logger::new("basic_demo");

    logger.debug("Debug message");
    logger.info("Application started");
    logger.warning("This is a warning");
    logger.error("This is an error");
    logger.critical("This is critical");

    logger.info_with(
        "User logged in",
        &[
            ("user_id", "12345"),
            ("action", "login"),
            ("ip", "192.168.1.1"),
        ],
    );

    logger.info("Basic logging demo complete");
    println!("✅ Basic logging example finished");
}
