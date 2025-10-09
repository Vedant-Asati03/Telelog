//! Managing persistent context that applies to multiple log entries.

use telelog::Logger;

fn main() {
    let logger = Logger::new("context_demo");

    logger.info("Starting application");

    logger.add_context("request_id", "req_123");
    logger.add_context("user_id", "user_456");
    logger.info("Processing request");

    logger.add_context("session_id", "sess_789");
    logger.info("User authenticated");

    logger.remove_context("session_id");
    logger.info("Session context removed");

    logger.clear_context();
    logger.info("All context cleared");

    {
        let _guard = logger.with_context("temp_id", "tmp_999");
        logger.info("Inside scoped context");
    }

    logger.info("Outside scoped context");

    println!("✅ Context management example finished");
}
