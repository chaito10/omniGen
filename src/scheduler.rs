//! Parallel task scheduler for pipeline execution.

use anyhow::Result;
use rayon::prelude::*;

/// The task scheduler for parallel pipeline execution.
pub struct Scheduler {
    thread_count: usize,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { thread_count: 0 }
    }

    pub fn with_threads(count: usize) -> Self {
        Self {
            thread_count: count,
        }
    }

    pub fn execute_parallel<T, F>(&self, tasks: Vec<F>) -> Result<Vec<T>>
    where
        T: Send,
        F: FnOnce() -> Result<T> + Send,
    {
        if self.thread_count > 0 {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(self.thread_count)
                .build()
                .map_err(|e| anyhow::anyhow!("failed to build thread pool: {}", e))?;

            pool.install(|| {
                tasks
                    .into_par_iter()
                    .map(|task| task())
                    .collect::<Result<Vec<_>>>()
            })
        } else {
            tasks
                .into_par_iter()
                .map(|task| task())
                .collect::<Result<Vec<_>>>()
        }
    }

    pub fn execute_sequential<T, F>(&self, tasks: Vec<F>) -> Result<Vec<T>>
    where
        F: FnOnce() -> Result<T>,
    {
        tasks.into_iter().map(|task| task()).collect()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}
