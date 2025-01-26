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
    use std::future::{Future, IntoFuture};
    use std::sync::{mpsc, Arc, Mutex};
    use std::sync::mpsc::{Receiver, Sender};
    use std::thread;
    use compact_str::format_compact;
    use tokio::runtime::Runtime;
    use tokio::sync::futures;
    use tokio::task::JoinHandle;
    use crate::core::{RUMResult, RUMVec};
    use crate::threading::threading_functions::get_default_system_thread_count;

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
    pub type AsyncTaskHandle<R> = JoinHandle<TaskResult<R>>;
    pub type AsyncTaskHandles<R> = Vec<AsyncTaskHandle<R>>;
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
            Task{task_processor, args}
        }

        ///
        /// Run the processor with the args and store the results in task
        ///
        pub fn execute(&mut self) -> TaskResult<R> {
            let processor = &self.task_processor;
            processor(&self.args)
        }
    }

    ///
    /// Thread Pool type that ensures we spawn the requested number of threads and call our
    /// Task type containing the payload and processing function. A reference is passed to the
    /// worker thread of an instance of [`MicroTaskQueue`].
    ///
    pub struct ThreadPool {
        runtime: Runtime
    }

    impl ThreadPool
    {
        ///
        /// Initializes an instance of [`RUMResult<ThreadPool>`] using the default number of threads available in the system.
        /// This is biased towards the bigger value between what Rust std reports and the actual cpu count
        /// reported by num_cpus crate.
        ///
        pub fn default() -> RUMResult<ThreadPool> {
            ThreadPool::new(get_default_system_thread_count())
        }

        ///
        /// Creates an instance of [`RUMResult<ThreadPool>`] with a pool of `size` threads pre-running
        /// and waiting for work. When this instance gets dropped, we signal threads to exit.
        ///
        pub fn new(threads: usize) -> RUMResult<ThreadPool> {
            let mut builder = tokio::runtime::Builder::new_multi_thread();
            builder.worker_threads(threads);
            builder.enable_all();
            let handle = match builder.build() {
                Ok(handle) => handle,
                Err(e) => return Err(format_compact!("Unable to initialize threading tokio runtime because {}!", &e)),
            };

            Ok(ThreadPool{runtime: handle})
        }

        ///
        /// Execute a [`MicroTaskQueue<T, R>`] filled with instances of [`Task<T, R>`].
        /// We actually send a clone of the reference of the microtask queue.
        /// This allows the main thread to poll for results and own the original tasks.
        ///
        pub fn execute<T: Send + Clone + 'static, R: Clone + Send  + 'static>(&self, task: SafeTask<T, R>) -> AsyncTaskHandle<R> {
            self.runtime.spawn(async move {
                let mut task_handle = task.lock().unwrap();
                task_handle.execute()
            })
        }

        pub fn resolve_task<R: Send + Clone + 'static>(&self, task: AsyncTaskHandle<R>) -> TaskResult<R> {
            self.runtime.block_on(task).unwrap()
        }
    }
}

pub mod threading_functions {
    use std::thread::available_parallelism;
    use num_cpus;

    pub fn get_default_system_thread_count() -> usize {
        let cpus: usize = num_cpus::get();
        let parallelism = match available_parallelism() {
            Ok(n) => n.get(),
            Err(_) => 0
        };

        if parallelism >= cpus {
            parallelism
        } else {
            cpus
        }
    }
}
