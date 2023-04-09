use crate::Result;

/// a thread_pool trait
pub trait ThreadPool {
    /// create a new threadpool, spawn the specific number of threads
    fn new(threads: u32) ->  Result<Self>
        where
            Self: Sized;
    /// spawn a function into the threadpool
    fn spawn<F>(&self, job: F) 
        where
            F: FnOnce() + Send + 'static;
}
mod naive;
pub use naive::NaiveThreadPool;
mod shared;
pub use shared::SharedQueueThreadPool;
mod Rayon;
pub use Rayon::RayonThreadPool;