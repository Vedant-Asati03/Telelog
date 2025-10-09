//! Tracking different components of your application.

use telelog::Logger;

fn database_operations(logger: &Logger) {
    let _component = logger.track_component("database");
    logger.info("Database connection established");
    logger.info("Running migration scripts");
    logger.info("Database ready");
}

fn api_operations(logger: &Logger) {
    let _component = logger.track_component("api");
    logger.info("API server starting");
    logger.info("Routes registered");
    logger.info("API server listening on port 8080");
}

fn auth_operations(logger: &Logger) {
    let _component = logger.track_component("auth");
    logger.info("Auth service initialized");
    logger.info("JWT validation enabled");
    logger.info("Auth ready");
}

fn main() {
    let logger = Logger::new("app");

    logger.info("Application starting");

    database_operations(&logger);
    api_operations(&logger);
    auth_operations(&logger);

    logger.info("All components initialized");

    println!("✅ Component tracking example finished");
}
