/*
 * rumtk attempts to implement HL7 and medical protocols for interoperability in medicine.
 * This toolkit aims to be reliable, simple, performant, and standards compliant.
 * Copyright (C) 2025  Luis M. Santos, M.D.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
 */

pub mod thread_primitives {
    use std::sync::{mpsc, Arc, Mutex};
    use std::sync::mpsc::{Receiver, Sender};
    use std::thread;
    use std::thread::JoinHandle;
    use crate::core::{RUMResult, RUMVec};

    pub type TaskItems<T> = RUMVec<T>;
    /// This type aliases a vector of T elements that will be used for passing arguments to the task processor.
    pub type TaskArgs<T> = TaskItems<T>;
    /// Type to use to define how task results are expected to be returned.
    pub type TaskResult<R> = RUMResult<TaskItems<R>>;
    pub type TaskResults<R> = TaskItems<TaskResult<R>>;
    /// Function signature defining the interface of task processing logic.
    pub type MicroTaskQueue<T, R> = Arc<Mutex<RUMVec<Task<T, R>>>>;
    pub type SafeTask<T, R> = Arc<Mutex<Task<T, R>>>;
    pub type SafeTasks<T, R> = RUMVec<SafeTask<T, R>>;
    pub type SafeTaskArgs<T> = Arc<Mutex<TaskItems<T>>>;
    pub type ThreadReceiver<T, R> = Arc<Mutex<Receiver<SafeTask<T, R>>>>;
    pub type ThreadSender<T, R> = Sender<SafeTask<T, R>>;
    pub type AsyncPool = Vec<tokio::task::JoinHandle<()>>;
    pub type TaskProcessor<T, R> = fn(args: &SafeTaskArgs<T>) -> TaskResult<R>;


    ///
    /// A [`Task<T, R>`] is composed of a processing function closure, a list of args of
    /// `T` type and a list of results of `R` type.
    ///
    #[derive(Debug, Clone)]
    pub struct Task<T, R>
    {
        task_processor: TaskProcessor<T, R>,
        args: SafeTaskArgs<T>,
        result: TaskResult<R>,
        queued: bool,
        complete: bool,
    }

    impl<T, R> Task<T, R>
    where
        T: Send + Clone + 'static,
        R: Send + Clone + 'static,
        Box<T>: Send + Clone + 'static,
    {
        ///
        /// Create an instance of [`Task<T, R>`].
        ///
        /// A [`Task<T, R>`] is composed of a processing function closure, a list of args of
        /// `T` type and a list of results of `R` type.
        ///
        pub fn new(task_processor: TaskProcessor<T, R>, args: SafeTaskArgs<T>) -> Task<T, R> {
            let result = Ok(TaskItems::new());
            Task{task_processor, args, result, queued: false, complete: false}
        }

        ///
        /// Run the processor with the args and store the results in task
        ///
        pub fn execute(&mut self) {
            let processor = &self.task_processor;
            self.result = processor(&self.args);
            self.complete = true;
        }

        pub fn is_completed(&self) -> bool { self.complete }

        pub fn get_result(&self) -> &TaskResult<R> {
            &self.result
        }
    }

    #[derive(Debug)]
    pub struct Worker {
        id: usize,
        thread: Option<JoinHandle<()>>,
    }

    impl Worker
    {
        pub fn new<T: Send + Sync + Clone + 'static, R: Send + Sync + Clone + 'static>(id: usize, receiver: ThreadReceiver<T, R>) -> Worker {
            let thread = thread::spawn(move ||
                loop {
                    let locked_receiver = receiver.lock().unwrap();
                    match locked_receiver.recv() {
                        Ok(safe_task) => {
                            let mut task = safe_task.lock().unwrap();
                            task.execute();
                        }
                        Err(_) => {
                            println!("Worker {id} disconnected; shutting down.");
                            break;
                        }
                    }
                }
            );
            Worker{id, thread: Some(thread)}
        }
    }

    ///
    /// Thread Pool type that ensures we spawn the requested number of threads and call our
    /// Task type containing the payload and processing function. A reference is passed to the
    /// worker thread of an instance of [`MicroTaskQueue`].
    ///
    pub struct ThreadPool<T, R> {
        workers: Vec<Worker>,
        sender: Option<ThreadSender<T, R>>,
    }

    impl<T, R> ThreadPool<T, R>
    where
        T: Send + Sync + Clone + 'static,
        R: Send + Sync + Clone + 'static,
    {
        ///
        /// Creates an instance of [`ThreadPool<T, R>`] with a pool of `size` threads pre-running
        /// and waiting for work. When this instance gets dropped, we signal threads to exit.
        ///
        pub fn new(size: usize) -> ThreadPool<T, R> {
            let (sender, receiver) = mpsc::channel();

            let receiver = ThreadReceiver::new(Mutex::new(receiver));
            let mut workers = Vec::with_capacity(size);
            for id in 0..size {
                workers.push(Worker::new(id, Arc::clone(&receiver)));
            }

            ThreadPool{workers, sender: Some(sender)}
        }

        ///
        /// Execute a [`MicroTaskQueue<T, R>`] filled with instances of [`Task<T, R>`].
        /// We actually send a clone of the reference of the microtask queue.
        /// This allows the main thread to poll for results and own the original tasks.
        ///
        pub fn execute(&self, task: &SafeTask<T, R>) {
            self.sender.as_ref().unwrap().send(Arc::clone(task)).unwrap()
        }
    }

    impl<T, R> Drop for ThreadPool<T, R> {
        ///
        /// Drop the sender and signal threads to exit.
        /// Then, join the threads until they have all exited.
        ///
        fn drop(&mut self) {
            drop(self.sender.take());
            for worker in &mut self.workers {
                println!("Shutting down worker {}", worker.id);
                if let Some(thread) = worker.thread.take() {
                    thread.join().unwrap();
                }
            }
        }
    }
}
