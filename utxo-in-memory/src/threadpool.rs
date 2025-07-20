//! ZkOS Thread Pool Implementation
//!
//! This module provides a custom thread pool implementation for concurrent processing
//! in the UTXO state management system. It supports job execution, graceful shutdown,
//! and named worker threads for debugging and monitoring.
//!
//! ## Features
//!
//! - **Concurrent Job Execution**: Execute multiple jobs simultaneously
//! - **Graceful Shutdown**: Properly terminate all workers on shutdown
//! - **Named Workers**: Each worker thread has a descriptive name
//! - **Message-Based Communication**: Uses channels for job distribution
//! - **Automatic Cleanup**: Implements Drop trait for resource cleanup
//!
//! ## Usage
//!
//! ```rust
//! use utxo_in_memory::ThreadPool;
//!
//! // Create a thread pool with 4 workers
//! let mut pool = ThreadPool::new(4, "UTXO_PROCESSOR".to_string());
//!
//! // Execute a job
//! pool.execute(|| {
//!     println!("Processing UTXO in worker thread");
//! });
//!
//! // Shutdown the pool
//! pool.shutdown();
//! ```
//!
//! ## Architecture
//!
//! The thread pool uses a producer-consumer pattern:
//! - **Producer**: Main thread sends jobs via channel
//! - **Consumers**: Worker threads receive and execute jobs
//! - **Message Types**: NewJob for work, Terminate for shutdown
//! - **Thread Safety**: Uses Arc<Mutex<>> for shared receiver

use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

/// Thread pool for concurrent job execution
///
/// A thread pool manages a collection of worker threads that can execute
/// jobs concurrently. Jobs are submitted via the `execute` method and
/// distributed among available workers.
///
/// # Fields
/// * `workers` - Collection of worker threads
/// * `sender` - Channel sender for job distribution
pub struct ThreadPool {
    /// Collection of worker threads
    workers: Vec<Worker>,
    /// Channel sender for job distribution
    sender: mpsc::Sender<Message>,
}

/// Type alias for job functions
///
/// Jobs are boxed functions that can be executed by worker threads.
/// They must be Send + 'static to be safely shared between threads.
type Job = Box<dyn FnOnce() + Send + 'static>;

/// Message types for thread pool communication
///
/// Messages are sent through the channel to coordinate between
/// the main thread and worker threads.
enum Message {
    /// New job to be executed
    NewJob(Job),
    /// Signal to terminate the worker
    Terminate,
}

impl ThreadPool {
    /// Creates a new ThreadPool with the specified number of workers
    ///
    /// The size parameter determines the number of worker threads that will
    /// be created to handle jobs. Each worker will have a unique name for
    /// debugging and monitoring purposes.
    ///
    /// # Arguments
    /// * `size` - Number of worker threads to create
    /// * `t_name` - Base name for worker threads (will be suffixed with worker ID)
    ///
    /// # Panics
    /// The `new` function will panic if the size is zero.
    ///
    /// # Returns
    /// * New ThreadPool instance
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

    /// Executes a job in the thread pool
    ///
    /// This method submits a job to the thread pool for execution by one
    /// of the available worker threads. The job will be executed asynchronously.
    ///
    /// # Arguments
    /// * `f` - Function to execute (must be Send + 'static)
    ///
    /// # Example
    /// ```rust
    /// use utxo_in_memory::ThreadPool;
    /// let pool = ThreadPool::new(2, "TEST".to_string());
    /// pool.execute(|| {
    ///     println!("Job executed in worker thread");
    /// });
    /// ```
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }

    /// Gracefully shuts down the thread pool
    ///
    /// This method sends termination signals to all worker threads and
    /// waits for them to complete. It ensures that all workers are properly
    /// cleaned up before returning.
    ///
    /// # Example
    /// ```rust
    /// use utxo_in_memory::ThreadPool;
    /// let mut pool = ThreadPool::new(2, "TEST".to_string());
    /// pool.shutdown();
    /// ```
    pub fn shutdown(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            // Instead of: self.sender.send(Message::Terminate).unwrap();
            match self.sender.send(Message::Terminate) {
                Ok(_) => (),
                Err(_) => {
                    // Channel is closed, workers already terminated
                    println!("Channel closed, workers already terminated");

                    break;
                }
            }
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
    /// Automatically shuts down the thread pool when dropped
    ///
    /// This implementation ensures that the thread pool is properly
    /// cleaned up even if shutdown() is not explicitly called.
    /// It sends termination signals to all workers and waits for
    /// them to complete.
    fn drop(&mut self) {
        // Check if workers are still active before trying to send termination messages
        if !self.workers.is_empty() {
            println!("Sending terminate message to all workers.");

            for _ in &self.workers {
                if let Err(_) = self.sender.send(Message::Terminate) {
                    // Channel is closed, workers already terminated
                    break;
                }
            }
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

/// Individual worker thread in the thread pool
///
/// A worker represents a single thread that continuously processes jobs
/// from the shared receiver channel. Each worker has a unique ID and
/// can be named for debugging purposes.
///
/// # Fields
/// * `id` - Unique identifier for the worker
/// * `thread` - Optional handle to the worker thread
struct Worker {
    /// Unique identifier for the worker
    id: usize,
    /// Optional handle to the worker thread
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Creates a new worker thread
    ///
    /// This method spawns a new thread that continuously listens for
    /// messages from the shared receiver. The thread will execute jobs
    /// or terminate based on the received messages.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the worker
    /// * `t_name` - Base name for the worker thread
    /// * `receiver` - Shared receiver for job messages
    ///
    /// # Returns
    /// * New Worker instance
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
