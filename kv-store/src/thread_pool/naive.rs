use super::ThreadPool;
use crate::Result;
use std::thread;

/// a naive ThreadPool implementation
pub struct NaiveThreadPool;

impl ThreadPool for NaiveThreadPool {
    /// create a new threadpool, spawn the specific number of threads
    fn new(threads: u32) ->  Result<NaiveThreadPool>{
        Ok(NaiveThreadPool)
    }
    /// spawn a function into the threadpool
    fn spawn<F>(&self, job: F) 
        where
            F: FnOnce() + Send + 'static {
        
        thread::spawn(job);
    }
}