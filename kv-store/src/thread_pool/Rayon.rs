use super::ThreadPool;
use crate::Result;

/// a Threadpool implementation using "work stealing" strategy
pub struct RayonThreadPool {
    threads: rayon::ThreadPool,
}

impl ThreadPool for RayonThreadPool {
    /// create a new threadpool, spawn the specific number of threads
    fn new(threads: u32) ->  Result<RayonThreadPool> {
        let thread_pool = rayon::ThreadPoolBuilder::new().num_threads(threads as usize).build().unwrap();

        Ok(RayonThreadPool { threads: thread_pool })
    }
    /// spawn a function into the threadpool
    fn spawn<F>(&self, job: F) 
        where
            F: FnOnce() + Send + 'static {

        self.threads.spawn(job);
    }
}