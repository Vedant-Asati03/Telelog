//! Tracking code execution time with performance profiling.

use std::thread;
use std::time::Duration;
use telelog::Logger;

fn simulate_work(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}

fn main() {
    let logger = Logger::new("profiler_demo");

    logger.info("Starting performance profiling");

    {
        let _profile = logger.profile("database_query");
        simulate_work(100);
        println!("  Simulated database query (100ms)");
    }

    {
        let _profile = logger.profile("data_processing");
        simulate_work(50);
        println!("  Simulated data processing (50ms)");
    }

    {
        let _profile = logger.profile("api_call");
        simulate_work(150);
        println!("  Simulated API call (150ms)");
    }

    {
        let _profile = logger.profile("complete_request");
        {
            let _profile = logger.profile("validation");
            simulate_work(30);
            println!("  Validation (30ms)");
        }

        {
            let _profile = logger.profile("execution");
            simulate_work(80);
            println!("  Execution (80ms)");
        }

        {
            let _profile = logger.profile("response_formatting");
            simulate_work(20);
            println!("  Response formatting (20ms)");
        }
    }

    println!("✅ Performance profiling example finished");
}
