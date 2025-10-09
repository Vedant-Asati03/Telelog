#!/usr/bin/env python3
"""Thread-safe logging in async environments."""

import asyncio
import telelog as tl

async def async_task(logger, task_name, duration):
    with logger.profile(f"async_{task_name}"):
        logger.info(f"Starting {task_name}")
        await asyncio.sleep(duration)
        logger.info(f"Completed {task_name}")

async def main():
    logger = tl.Logger("async_demo")
    
    logger.info("Starting async logging demo")
    
    tasks = [
        async_task(logger, "database_fetch", 0.1),
        async_task(logger, "api_call", 0.08),
        async_task(logger, "cache_update", 0.05)
    ]
    
    await asyncio.gather(*tasks)
    
    logger.info("All async tasks completed")
    print("✅ Async logging example finished")

if __name__ == "__main__":
    asyncio.run(main())