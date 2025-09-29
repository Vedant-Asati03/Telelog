//! Component Tracking Example
//! Demonstrates tracking architectural components and their relationships.

use telelog::Logger;
use std::{thread, time::Duration};

fn main() {
    let logger = Logger::new("component_demo");
    
    logger.info("Starting component tracking demo");
    
    // Track web server component
    {
        let _web_server = logger.track_component("web_server");
        logger.info("Web server handling request");
        
        // Track authentication service
        {
            let _auth_service = logger.track_component("auth_service");
            thread::sleep(Duration::from_millis(50));  // Simulate auth work
            logger.info("User authenticated");
        }
        
        // Track database layer
        {
            let _database = logger.track_component("database");
            thread::sleep(Duration::from_millis(80));  // Simulate DB work
            logger.info("Data retrieved");
        }
        
        // Track response generation
        {
            let _response_builder = logger.track_component("response_builder");
            thread::sleep(Duration::from_millis(30));  // Simulate response building
            logger.info("Response generated");
        }
        
        logger.info("Request completed");
    }
    
    println!("✅ Component tracking example finished");
}