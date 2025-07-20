//! Simple thread pool implementation for parallel job execution.
//!
//! This module provides a basic thread pool for running jobs concurrently across a fixed number of worker threads.
//!
//! # Features
//! - Fixed-size thread pool
//! - Graceful shutdown on drop
//! - Named worker threads for easier debugging
//!
//! # Example
//! ```
//! use chain_oracle::ThreadPool;
//! let pool = ThreadPool::new(4, "my_pool".to_string());
//! pool.execute(|| println!("Hello from the thread pool!"));
//! ```

use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

/// A simple thread pool for parallel processing.
///
/// The thread pool manages a fixed number of worker threads, each capable of executing jobs submitted to the pool.
/// Jobs are executed in the order they are received.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

impl ThreadPool {
    /// Creates a new `ThreadPool`.
    ///
    /// # Arguments
    /// * `size` - The number of worker threads in the pool. Must be greater than zero.
    /// * `t_name` - A name prefix for the worker threads (useful for debugging).
    ///
    /// # Panics
    /// Panics if `size` is zero.
    pub fn new(size: usize, t_name: String) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, t_name.clone(), Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    /// Executes a job in the thread pool.
    ///
    /// # Arguments
    /// * `f` - A closure or function to execute. Must be `Send` and `'static`.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }

    /// Gracefully shuts down the thread pool, waiting for all workers to finish.
    ///
    /// This method sends a terminate message to each worker and joins their threads.
    pub fn shutdown(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Drop for ThreadPool {
    /// Drops the thread pool, ensuring all workers are terminated.
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

/// A worker in the thread pool.
///
/// Each worker runs in its own thread and executes jobs sent to it via a channel.
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Creates a new worker with the given ID and thread name prefix.
    ///
    /// # Arguments
    /// * `id` - The worker's unique identifier.
    /// * `t_name` - The name prefix for the thread.
    /// * `receiver` - The shared receiver for job messages.
    fn new(id: usize, t_name: String, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::Builder::new()
            .name(format!("{}-{}", t_name, id))
            .spawn(move || loop {
                let message = receiver.lock().unwrap().recv().unwrap();

                match message {
                    Message::NewJob(job) => {
                        // println!("Worker {} got a job; executing.", id);
                        job();
                    }
                    Message::Terminate => {
                        // println!("Worker {} was told to terminate.", id);
                        break;
                    }
                }
            });

        Worker {
            id,
            thread: Some(thread.unwrap()),
        }
    }
}
