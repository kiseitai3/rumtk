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
    use tokio::sync::{futures, RwLock};
    use tokio::runtime::Runtime as TokioRuntime;
    use tokio::task::JoinHandle;
    use crate::core::{RUMResult, RUMVec};
    use crate::cache::{new_cache, LazyRUMCache, get_or_set_from_cache};
    use crate::rum_cache_fetch;
    use crate::strings::RUMString;
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
    pub type SafeTaskArgs<T> = Arc<RwLock<TaskItems<T>>>;
    pub type ThreadReceiver<T, R> = Arc<Mutex<Receiver<SafeTask<T, R>>>>;
    pub type ThreadSender<T, R> = Sender<SafeTask<T, R>>;
    pub type AsyncTaskHandle<R> = JoinHandle<TaskResult<R>>;
    pub type AsyncTaskHandles<R> = Vec<AsyncTaskHandle<R>>;
    pub type TaskProcessor<T, R> = fn(args: &SafeTaskArgs<T>) -> TaskResult<R>;


    /**************************** Globals **************************************/
    pub static mut rt_cache: TokioRtCache = new_cache();
    /**************************** Types *****************************************/
    type TokioRtCache = LazyRUMCache<usize, Arc<TokioRuntime>>;
    /**************************** Helpers ***************************************/
    pub fn init_cache(threads: &usize) -> Arc<TokioRuntime> {
        let mut builder = tokio::runtime::Builder::new_multi_thread();
        builder.worker_threads(*threads);
        builder.enable_all();
        match builder.build() {
            Ok(handle) => Arc::new(handle),
            Err(e) => panic!("Unable to initialize threading tokio runtime because {}!", &e),
        }
    }

    /**************************** Types ***************************************/

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
        T: Sync + Send + Clone + 'static,
        R: Sync + Send + Clone + 'static,
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
        runtime: &'static Runtime
    }

    impl ThreadPool
    {
        ///
        /// Initializes an instance of [`RUMResult<ThreadPool>`] using the default number of threads available in the system.
        /// This is biased towards the bigger value between what Rust std reports and the actual cpu count
        /// reported by num_cpus crate.
        ///
        pub fn default() -> RUMResult<ThreadPool> {
            ThreadPool::new(&get_default_system_thread_count())
        }

        ///
        /// Creates an instance of [`RUMResult<ThreadPool>`] with a pool of `size` threads pre-running
        /// and waiting for work. When this instance gets dropped, we signal threads to exit.
        ///
        pub fn new(threads: &usize) -> RUMResult<ThreadPool> {
            let rt = rum_cache_fetch!(&mut rt_cache, &threads, init_cache);

            Ok( ThreadPool{runtime: rt })
        }

        ///
        /// Execute a [`MicroTaskQueue<T, R>`] filled with instances of [`Task<T, R>`].
        /// We actually send a clone of the reference of the microtask queue.
        /// This allows the main thread to poll for results and own the original tasks.
        ///
        pub fn execute<T: Sync + Send + Clone + 'static, R: Sync + Clone + Send  + 'static>(&self, task: SafeTask<T, R>) -> AsyncTaskHandle<R> {
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

pub mod threading_macros {
    #[macro_export]
    macro_rules! rum_init_threads {
        ( ) => {{
            use crate::threading::thread_primitives::{rt_cache, init_cache};
            use crate::threading::threading_functions::get_default_system_thread_count;
            use crate::rum_cache_fetch;
            let rt = rum_cache_fetch!(&mut rt_cache, &get_default_system_thread_count(), init_cache);
            rt
        }};
        ( $threads:expr ) => {{
            use crate::threading::thread_primitives::{rt_cache, init_cache};
            use crate::rum_cache_fetch;
            let rt = rum_cache_fetch!(&mut rt_cache, $threads, init_cache);
            rt
        }};
    }

    #[macro_export]
    macro_rules! rum_spawn_job {
        ( $rt:expr, $func:expr ) => {{
            $rt.spawn(async move {
                $func().await
            })
        }};
        ( $rt:expr, $func:expr, $($arg_items:expr),+ ) => {{
            $rt.spawn(async move {
                $func($($arg_items),+).await
            })
        }};
    }

    #[macro_export]
    macro_rules! rum_wait_on_job {
        ( $rt:expr, $func:expr ) => {{
            $rt.block_on(async move {
                $func().await
            })
        }};
        ( $rt:expr, $func:expr, $($arg_items:expr),+ ) => {{
            $rt.block_on(async move {
                $func($($arg_items),+).await
            })
        }};
    }

    #[macro_export]
    macro_rules! rum_resolve_job {
        ( $rt:expr, $future:expr ) => {{
            $rt.block_on(async move {
                $future.await
            })
        }};
    }

    #[macro_export]
    macro_rules! run_async_as_sync {
        ( $func:expr ) => {{
            let tokio_runtime = tokio::runtime::Handle::current();
            tokio_runtime.block_on(async move {
                $func().await
            })
        }};
        ( $func:expr, $($arg_items:expr),+ ) => {{
            let tokio_runtime = tokio::runtime::Handle::current();
            tokio_runtime.block_on(async move {
                $func($($arg_items),+).await
            })
        }};
    }

    #[macro_export]
    macro_rules! run_quick_async_as_sync {
        ( $func:expr ) => {{
            let tokio_runtime = tokio::runtime::Handle::current();
            let arg_list = vec![];
            let args = create_task_args!(arg_list);
            tokio_runtime.block_on(async move {
                $func(&args).await
            })
        }};
        ( $func:expr, $($arg_items:expr),+ ) => {{
            let tokio_runtime = tokio::runtime::Handle::current();
            let arg_list = vec![$($arg_items),+];
            let args = create_task_args!(arg_list);
            tokio_runtime.block_on(async move {
                $func(&args).await
            })
        }};
    }

    #[macro_export]
    macro_rules! run_quick_background_task {
        ( $func:expr ) => {{
            let tokio_runtime = get;
            let arg_list = vec![];
            let args = create_task_args!(arg_list);
            tokio_runtime.spawn(async move {
                $func(&args).await
            })
        }};
        ( $func:expr, $($arg_items:expr),+ ) => {{
            let tokio_runtime = tokio::runtime::Handle::current();
            let arg_list = vec![$($arg_items),+];
            let args = create_task_args!(arg_list);
            tokio_runtime.spawn(async move {
                $func(&args).await
            })
        }};
    }

    #[macro_export]
    macro_rules! run_quick_task {
        ( $task:expr ) => {{
            use $crate::threading::thread_primitives::{ThreadPool};
            let mut init = ThreadPool::new(1)?;
            init.resolve_task(init.execute($task))
        }};
    }

    #[macro_export]
    macro_rules! create_task {
        ( $processor:expr, $args:expr ) => {{
            use $crate::threading::thread_primitives::{Task, SafeTask};
            SafeTask::new(Mutex::new(Task::new($processor, $args)))
        }};
    }

    #[macro_export]
    macro_rules! create_task_args {
        ( $args:expr ) => {{
            use $crate::threading::thread_primitives::{TaskArgs, SafeTaskArgs};
            use tokio::sync::RwLock;
            SafeTaskArgs::new(RwLock::new($args))
        }};
    }

    #[macro_export]
    macro_rules! create_thread_pool {
        ( $threads:expr ) => {{
            use $crate::threading::thread_primitives::{ThreadPool};
            ThreadPool::new($threads)
        }};
        ( ) => {{
            use $crate::threading::thread_primitives::{ThreadPool};
            ThreadPool::default()
        }};
    }

    #[macro_export]
    macro_rules! execute_task {
        ( $pool:expr, $task:expr ) => {{
            $pool.execute($task)
        }};
    }
}
