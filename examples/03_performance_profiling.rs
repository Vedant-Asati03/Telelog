//! Performance Profiling Example
//! Demonstrates timing operations and performance measurement.

use telelog::Logger;
use std::{thread, time::Duration};

fn main() {
    let logger = Logger::new("perf_demo");
    
    logger.info("Starting performance demo");
    
    // Simple profiling with guard
    {
        let _timer = logger.profile("database_query");
        thread::sleep(Duration::from_millis(100));  // Simulate database work
        logger.info("Query executed");
    }  // Timer automatically logs when dropped
    
    // Nested profiling
    {
        let _request_timer = logger.profile("request_processing");
        logger.info("Processing request");
        
        {
            let _validation_timer = logger.profile("validation");
            thread::sleep(Duration::from_millis(50));  // Simulate validation
            logger.info("Input validated");
        }
        
        {
            let _logic_timer = logger.profile("business_logic");
            thread::sleep(Duration::from_millis(80));  // Simulate processing
            logger.info("Business logic executed");
        }
        
        logger.info("Request completed");
    }
    
    println!("✅ Performance profiling example finished");
}