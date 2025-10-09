//! Thread-safe logging in concurrent environments.

use std::{thread, time::Duration};
use telelog::Logger;

fn simulate_async_task(logger: &Logger, task_name: &str, duration_ms: u64) {
    let _timer = logger.profile(&format!("async_{}", task_name));
    logger.info_with(
        &format!("Starting {}", task_name),
        &[
            ("task_type", "async"),
            ("duration_ms", &duration_ms.to_string()),
        ],
    );
    thread::sleep(Duration::from_millis(duration_ms));
    logger.info(&format!("Completed {}", task_name));
}

fn main() {
    let logger = Logger::new("async_demo");

    logger.info("Starting concurrent logging demo");

    let handles: Vec<_> = [
        ("database_fetch", 100),
        ("api_call", 80),
        ("cache_update", 50),
    ]
    .into_iter()
    .map(|(task_name, duration)| {
        let logger_clone = logger.clone();
        let task_name = task_name.to_string();
        thread::spawn(move || {
            simulate_async_task(&logger_clone, &task_name, duration);
        })
    })
    .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    logger.info("All tasks completed");
    println!("✅ Async logging example finished");
}
