use super::ThreadPool;
use crate::Result;
use std::{thread, sync::{mpsc, Arc, Mutex}, panic::{self, AssertUnwindSafe}};

/// a thread with thread_id
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<WorkJob>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();
            match job {
                WorkJob::NewJob { job } => {
                    // 这里有点不太明白为什么要加一个AsserUnwindSafe才能通过编译
                    if let Err(err) = panic::catch_unwind(AssertUnwindSafe(job)) {
                        eprintln!("panic error message: {:?}", err);
                    }
                    
                },
                WorkJob::TerminateJob => {
                    break;
                }
            }
        });
        Worker { id, thread }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum WorkJob {
    // send a new job to the threadpool
    NewJob {job: Job},
    // when receive a shutdown message, all threads should be shut down
    TerminateJob,
}

/// a ThreadPool implementation with shared queue threadpools
pub struct SharedQueueThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<WorkJob>,
}

impl ThreadPool for SharedQueueThreadPool {
    /// create a shared queue with a specific number of threads
    fn new(threads: u32) ->  Result<SharedQueueThreadPool>{
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(threads as usize);
        for id in 0..threads {
            workers.push(Worker::new(id as usize, Arc::clone(&receiver)));
        }
        Ok(SharedQueueThreadPool{ workers, sender })
    }
    /// spawn a function into the threadpool
    fn spawn<F>(&self, job: F) 
        where
            F: FnOnce() + Send + 'static {
        
        let job = Box::new(job);
        self.sender.send(WorkJob::NewJob { job }).unwrap();
    }
}

impl Drop for SharedQueueThreadPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender.send(WorkJob::TerminateJob).unwrap();
        }
    }
}