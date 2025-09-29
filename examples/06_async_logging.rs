//! Async Logging Example
//! Demonstrates logging in async environments (using threads to simulate).

use telelog::Logger;
use std::{thread, time::Duration};

fn simulate_async_task(logger: &Logger, task_name: &str, duration_ms: u64) {
    let _timer = logger.profile(&format!("async_{}", task_name));
    logger.info(&format!("Starting {}", task_name));
    thread::sleep(Duration::from_millis(duration_ms));
    logger.info(&format!("Completed {}", task_name));
}

fn main() {
    let logger = Logger::new("async_demo");
    
    logger.info("Starting async logging demo");
    
    // Add context for the async session
    logger.add_context("session_id", "async_123");
    
    // Simulate concurrent async tasks using threads
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
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    logger.info("All async tasks completed");
    println!("✅ Async logging example finished");
}