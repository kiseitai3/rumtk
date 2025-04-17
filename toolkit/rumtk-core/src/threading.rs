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

///
/// This module provides all of the primitives needed to build a multithreaded application.
///
pub mod thread_primitives {
    use crate::cache::{new_cache, LazyRUMCache};
    use crate::core::{RUMResult, RUMVec};
    use std::future::IntoFuture;
    use std::sync::Arc;
    use tokio::runtime::Runtime as TokioRuntime;
    use tokio::sync::RwLock;
    use tokio::task::JoinHandle;

    /**************************** Globals **************************************/
    pub static mut rt_cache: TokioRtCache = new_cache();
    /**************************** Helpers ***************************************/
    pub fn init_cache(threads: &usize) -> SafeTokioRuntime {
        let mut builder = tokio::runtime::Builder::new_multi_thread();
        builder.worker_threads(*threads);
        builder.enable_all();
        match builder.build() {
            Ok(handle) => Arc::new(handle),
            Err(e) => panic!(
                "Unable to initialize threading tokio runtime because {}!",
                &e
            ),
        }
    }

    /**************************** Types ***************************************/
    pub type SafeTokioRuntime = Arc<TokioRuntime>;
    pub type TokioRtCache = LazyRUMCache<usize, SafeTokioRuntime>;
    pub type TaskItems<T> = RUMVec<T>;
    /// This type aliases a vector of T elements that will be used for passing arguments to the task processor.
    pub type TaskArgs<T> = TaskItems<T>;
    /// Type to use to define how task results are expected to be returned.
    pub type TaskResult<R> = RUMResult<TaskItems<R>>;
    pub type TaskResults<R> = TaskItems<TaskResult<R>>;
    /// Function signature defining the interface of task processing logic.
    pub type SafeTaskArgs<T> = Arc<RwLock<TaskItems<T>>>;
    pub type AsyncTaskHandle<R> = JoinHandle<TaskResult<R>>;
    pub type AsyncTaskHandles<R> = Vec<AsyncTaskHandle<R>>;
    //pub type TaskProcessor<T, R, Fut: Future<Output = TaskResult<R>>> = impl FnOnce(&SafeTaskArgs<T>) -> Fut;
}

///
/// This module contains a few helper.
///
/// For example, you can find a function for determining number of threads available in system.
/// The sleep family of functions are also here.
///
pub mod threading_functions {
    use num_cpus;
    use std::thread::{available_parallelism, sleep as std_sleep};
    use std::time::Duration;
    use tokio::time::sleep as tokio_sleep;

    pub const NANOS_PER_SEC: u64 = 1000000000;
    pub const MILLIS_PER_SEC: u64 = 1000;
    pub const MICROS_PER_SEC: u64 = 1000000;

    pub fn get_default_system_thread_count() -> usize {
        let cpus: usize = num_cpus::get();
        let parallelism = match available_parallelism() {
            Ok(n) => n.get(),
            Err(_) => 0,
        };

        if parallelism >= cpus {
            parallelism
        } else {
            cpus
        }
    }

    pub fn sleep(s: f32) {
        let ns = s * NANOS_PER_SEC as f32;
        let rounded_ns = ns.round() as u64;
        let duration = Duration::from_nanos(rounded_ns);
        std_sleep(duration);
    }

    pub async fn async_sleep(s: f32) {
        let ns = s * NANOS_PER_SEC as f32;
        let rounded_ns = ns.round() as u64;
        let duration = Duration::from_nanos(rounded_ns);
        tokio_sleep(duration).await;
    }
}

///
/// Main API for interacting with the threading back end. Remember, we use tokio as our executor.
/// This means that by default, all jobs sent to the thread pool have to be async in nature.
/// These macros make handling of these jobs at the sync/async boundary more convenient.
///
pub mod threading_macros {
    use crate::threading::thread_primitives;
    use crate::threading::thread_primitives::SafeTaskArgs;

    ///
    /// First, let's make sure we have *tokio* initialized at least once. The runtime created here
    /// will be saved to the global context so the next call to this macro will simply grab a
    /// reference to the previously initialized runtime.
    ///
    /// Passing nothing will default to initializing a runtime using the default number of threads
    /// for this system. This is typically equivalent to number of cores/threads for your CPU.
    ///
    /// Passing `threads` number will yield a runtime that allocates that many threads.
    ///
    ///
    /// ## Examples
    ///
    /// ```
    ///     use rumtk_core::{rumtk_init_threads, rumtk_resolve_task, rumtk_create_task_args, rumtk_create_task, rumtk_spawn_task};
    ///     use rumtk_core::core::RUMResult;
    ///     use rumtk_core::threading::thread_primitives::SafeTaskArgs;
    ///
    ///     async fn test(args: &SafeTaskArgs<i32>) -> RUMResult<Vec<i32>> {
    ///         let mut result = Vec::<i32>::new();
    ///         for arg in args.read().await.iter() {
    ///             result.push(*arg);
    ///         }
    ///         Ok(result)
    ///     }
    ///
    ///     let rt = rumtk_init_threads!();                                      // Creates runtime instance
    ///     let args = rumtk_create_task_args!(1);                               // Creates a vector of i32s
    ///     let task = rumtk_create_task!(test, args);                           // Creates a standard task which consists of a function or closure accepting a Vec<T>
    ///     let result = rumtk_resolve_task!(&rt, rumtk_spawn_task!(&rt, task)); // Spawn's task and waits for it to conclude.
    /// ```
    ///
    /// ```
    ///     use rumtk_core::{rumtk_init_threads, rumtk_resolve_task, rumtk_create_task_args, rumtk_create_task, rumtk_spawn_task};
    ///     use rumtk_core::core::RUMResult;
    ///     use rumtk_core::threading::thread_primitives::SafeTaskArgs;
    ///
    ///     async fn test(args: &SafeTaskArgs<i32>) -> RUMResult<Vec<i32>> {
    ///         let mut result = Vec::<i32>::new();
    ///         for arg in args.read().await.iter() {
    ///             result.push(*arg);
    ///         }
    ///         Ok(result)
    ///     }
    ///
    ///     let thread_count: usize = 10;
    ///     let rt = rumtk_init_threads!(&thread_count);
    ///     let args = rumtk_create_task_args!(1);
    ///     let task = rumtk_create_task!(test, args);
    ///     let result = rumtk_resolve_task!(&rt, rumtk_spawn_task!(&rt, task));
    /// ```
    #[macro_export]
    macro_rules! rumtk_init_threads {
        ( ) => {{
            use $crate::rumtk_cache_fetch;
            use $crate::threading::thread_primitives::{init_cache, rt_cache};
            use $crate::threading::threading_functions::get_default_system_thread_count;
            let rt = rumtk_cache_fetch!(
                &mut rt_cache,
                &get_default_system_thread_count(),
                init_cache
            );
            rt
        }};
        ( $threads:expr ) => {{
            use $crate::rumtk_cache_fetch;
            use $crate::threading::thread_primitives::{init_cache, rt_cache};
            let rt = rumtk_cache_fetch!(&mut rt_cache, $threads, init_cache);
            rt
        }};
    }

    ///
    /// Puts task onto the runtime queue.
    ///
    /// The parameters to this macro are a reference to the runtime (`rt`) and a future (`func`).
    ///
    /// The return is a [thread_primitives::JoinHandle<T>] instance. If the task was a standard
    /// framework task, you will get [thread_primitives::AsyncTaskHandle] instead.
    ///
    #[macro_export]
    macro_rules! rumtk_spawn_task {
        ( $rt:expr, $func:expr ) => {{
            $rt.spawn($func)
        }};
    }

    ///
    /// Using the initialized runtime, wait for the future to resolve in a thread blocking manner!
    ///
    /// If you pass a reference to the runtime (`rt`) and an async closure (`func`), we await the
    /// async closure without passing any arguments.
    ///
    /// You can pass a third argument to this macro in the form of any number of arguments (`arg_item`).
    /// In such a case, we pass those arguments to the call on the async closure and await on results.
    ///
    #[macro_export]
    macro_rules! rumtk_wait_on_task {
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

    ///
    /// This macro awaits a future.
    ///
    /// The arguments are a reference to the runtime (`rt) and a future.
    ///
    /// If there is a result, you will get the result of the future.
    ///
    /// ## Examples
    ///
    /// ```
    ///     use rumtk_core::{rumtk_init_threads, rumtk_resolve_task, rumtk_create_task_args, rumtk_create_task, rumtk_spawn_task};
    ///     use rumtk_core::core::RUMResult;
    ///     use rumtk_core::threading::thread_primitives::SafeTaskArgs;
    ///
    ///     async fn test(args: &SafeTaskArgs<i32>) -> RUMResult<Vec<i32>> {
    ///         let mut result = Vec::<i32>::new();
    ///         for arg in args.read().await.iter() {
    ///             result.push(*arg);
    ///         }
    ///         Ok(result)
    ///     }
    ///
    ///     let rt = rumtk_init_threads!();
    ///     let args = rumtk_create_task_args!(1);
    ///     let task = rumtk_create_task!(test, args);
    ///     let result = rumtk_resolve_task!(&rt, rumtk_spawn_task!(&rt, task));
    /// ```
    ///
    #[macro_export]
    macro_rules! rumtk_resolve_task {
        ( $rt:expr, $future:expr ) => {{
            $rt.block_on(async move { $future.await }).unwrap()
        }};
    }

    ///
    /// This macro creates an async body that calls the async closure and awaits it.
    ///
    #[macro_export]
    macro_rules! rumtk_create_task {
        ( $func:expr, $args:expr ) => {{
            async move {
                let f = $func;
                f(&$args).await
            }
        }};
    }

    ///
    /// Creates an instance of [SafeTaskArgs] with the arguments passed.
    ///
    /// ## Note
    ///
    /// All arguments must be of the same type
    ///
    #[macro_export]
    macro_rules! rumtk_create_task_args {
        ( $($args:expr),+ ) => {{
            use $crate::threading::thread_primitives::{TaskArgs, SafeTaskArgs, TaskItems};
            use tokio::sync::RwLock;
            SafeTaskArgs::new(RwLock::new(vec![$($args),+]))
        }};
    }

    ///
    /// Convenience macro for packaging the task components and launching the task in one line.
    ///
    /// One of the advantages is that you can generate a new `tokio` runtime by specifying the
    /// number of threads at the end. This is optional. Meaning, we will default to the system's
    /// number of threads if that value is not specified.
    ///
    /// Between the `func` parameter and the optional `threads` parameter, you can specify a
    /// variable number of arguments to pass to the task. each argument must be of the same type.
    /// If you wish to pass different arguments with different types, please define an abstract type
    /// whose underlying structure is a tuple of items and pass that instead.
    ///
    /// ## Examples
    ///
    /// ### 1
    /// ```
    ///     use rumtk_core::{rumtk_exec_task};
    ///     use rumtk_core::core::RUMResult;
    ///     use rumtk_core::threading::thread_primitives::SafeTaskArgs;
    ///
    ///     async fn test(args: &SafeTaskArgs<i32>) -> RUMResult<Vec<i32>> {
    ///         let mut result = Vec::<i32>::new();
    ///         for arg in args.read().await.iter() {
    ///             result.push(*arg);
    ///         }
    ///         Ok(result)
    ///     }
    ///
    ///     let result = rumtk_exec_task!(test, vec![1]);
    /// ```
    ///
    /// ### 2
    /// ```
    ///     use rumtk_core::{rumtk_exec_task};
    ///     use rumtk_core::core::RUMResult;
    ///     use rumtk_core::threading::thread_primitives::SafeTaskArgs;
    ///
    ///     async fn test(args: &SafeTaskArgs<i32>) -> RUMResult<Vec<i32>> {
    ///         let mut result = Vec::<i32>::new();
    ///         for arg in args.read().await.iter() {
    ///             result.push(*arg);
    ///         }
    ///         Ok(result)
    ///     }
    ///
    ///     let result = rumtk_exec_task!(test, vec![1, 5], 6);
    /// ```
    ///
    /// ## Equivalent To
    ///
    /// ```
    ///     use rumtk_core::{rumtk_init_threads, rumtk_resolve_task, rumtk_create_task_args, rumtk_create_task, rumtk_spawn_task};
    ///     use rumtk_core::core::RUMResult;
    ///     use rumtk_core::threading::thread_primitives::SafeTaskArgs;
    ///
    ///     async fn test(args: &SafeTaskArgs<i32>) -> RUMResult<Vec<i32>> {
    ///         let mut result = Vec::<i32>::new();
    ///         for arg in args.read().await.iter() {
    ///             result.push(*arg);
    ///         }
    ///         Ok(result)
    ///     }
    ///
    ///     let rt = rumtk_init_threads!();
    ///     let args = rumtk_create_task_args!(1);
    ///     let task = rumtk_create_task!(test, args);
    ///     let result = rumtk_resolve_task!(&rt, rumtk_spawn_task!(&rt, task));
    /// ```
    ///
    #[macro_export]
    macro_rules! rumtk_exec_task {
        ($func:expr, $args:expr ) => {{
            use tokio::sync::RwLock;
            use $crate::{
                rumtk_create_task, rumtk_create_task_args, rumtk_init_threads, rumtk_resolve_task,
            };
            let rt = rumtk_init_threads!();
            let args = SafeTaskArgs::new(RwLock::new($args));
            let task = rumtk_create_task!($func, args);
            rumtk_resolve_task!(&rt, task)
        }};
        ($func:expr, $args:expr , $threads:expr ) => {{
            use tokio::sync::RwLock;
            use $crate::{
                rumtk_create_task, rumtk_create_task_args, rumtk_init_threads, rumtk_resolve_task,
            };
            let rt = rumtk_init_threads!(&$threads);
            let args = SafeTaskArgs::new(RwLock::new($args));
            let task = rumtk_create_task!($func, args);
            rumtk_resolve_task!(&rt, task)
        }};
    }

    ///
    /// Sleep a duration of time in a sync context, so no await can be call on the result.
    ///
    /// You can pass any value that can be cast to f32.
    ///
    /// The precision is up to nanoseconds and it is depicted by the number of decimal places.
    ///
    /// ## Examples
    ///
    /// ```
    ///     use rumtk_core::rumtk_sleep;
    ///     rumtk_sleep!(1);           // Sleeps for 1 second.
    ///     rumtk_sleep!(0.001);       // Sleeps for 1 millisecond
    ///     rumtk_sleep!(0.000001);    // Sleeps for 1 microsecond
    ///     rumtk_sleep!(0.000000001); // Sleeps for 1 nanosecond
    /// ```
    ///
    #[macro_export]
    macro_rules! rumtk_sleep {
        ( $dur:expr) => {{
            use $crate::threading::threading_functions::sleep;
            sleep($dur as f32)
        }};
    }

    ///
    /// Sleep for some duration of time in an async context. Meaning, we can be awaited.
    ///
    /// You can pass any value that can be cast to f32.
    ///
    /// The precision is up to nanoseconds and it is depicted by the number of decimal places.
    ///
    /// ## Examples
    ///
    /// ```
    ///     use rumtk_core::rumtk_async_sleep;
    ///     rumtk_async_sleep!(1).await;           // Sleeps for 1 second.
    ///     rumtk_async_sleep!(0.001).await;       // Sleeps for 1 millisecond
    ///     rumtk_async_sleep!(0.000001).await;    // Sleeps for 1 microsecond
    ///     rumtk_async_sleep!(0.000000001).await; // Sleeps for 1 nanosecond
    /// ```
    ///
    #[macro_export]
    macro_rules! rumtk_async_sleep {
        ( $dur:expr) => {{
            use $crate::threading::threading_functions::async_sleep;
            async_sleep($dur as f32)
        }};
    }
}
