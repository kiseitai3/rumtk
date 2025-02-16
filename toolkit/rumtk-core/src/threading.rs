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
    use std::pin::Pin;
    use std::sync::{mpsc, Arc, Mutex};
    use std::sync::mpsc::{Receiver, Sender};
    use tokio::sync::{futures, RwLock};
    use tokio::runtime::Runtime as TokioRuntime;
    use tokio::task::JoinHandle;
    use crate::core::{RUMResult, RUMVec};
    use crate::cache::{new_cache, LazyRUMCache};

    /**************************** Globals **************************************/
    pub static mut rt_cache: TokioRtCache = new_cache();
    /**************************** Helpers ***************************************/
    pub fn init_cache(threads: &usize) -> SafeTokioRuntime {
        let mut builder = tokio::runtime::Builder::new_multi_thread();
        builder.worker_threads(*threads);
        builder.enable_all();
        match builder.build() {
            Ok(handle) => Arc::new(handle),
            Err(e) => panic!("Unable to initialize threading tokio runtime because {}!", &e),
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

pub mod threading_functions {
    use std::time::Duration;
    use std::thread::{available_parallelism, sleep as std_sleep};
    use tokio::time::sleep as tokio_sleep;
    use num_cpus;

    pub const NANOS_PER_SEC: u64 = 1000000000;
    pub const MILLIS_PER_SEC: u64 = 1000;
    pub const MICROS_PER_SEC: u64 = 1000000;

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

    pub fn sleep(s: f32) {
        let ns = s * NANOS_PER_SEC as f32;
        let rounded_ns = ns.round() as u64;
        let duration = Duration::from_nanos(rounded_ns);
        std_sleep(duration);
    }

    async fn async_sleep(s: f32) {
        let ns = s * NANOS_PER_SEC as f32;
        let rounded_ns = ns.round() as u64;
        let duration = Duration::from_nanos(rounded_ns);
        tokio_sleep(duration).await;
    }
}

pub mod threading_macros {
    use crate::threading::thread_primitives::TaskResult;

    #[macro_export]
    macro_rules! rumtk_init_threads {
        ( ) => {{
            use crate::threading::thread_primitives::{rt_cache, init_cache};
            use crate::threading::threading_functions::get_default_system_thread_count;
            use crate::rumtk_cache_fetch;
            let rt = rumtk_cache_fetch!(&mut rt_cache, &get_default_system_thread_count(), init_cache);
            rt
        }};
        ( $threads:expr ) => {{
            use crate::threading::thread_primitives::{rt_cache, init_cache};
            use crate::rumtk_cache_fetch;
            let rt = rumtk_cache_fetch!(&mut rt_cache, $threads, init_cache);
            rt
        }};
    }

    #[macro_export]
    macro_rules! rumtk_spawn_task {
        ( $rt:expr, $func:expr ) => {{
            $rt.spawn($func)
        }};
    }

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

    #[macro_export]
    macro_rules! rumtk_resolve_task {
        ( $rt:expr, $future:expr ) => {{
            $rt.block_on(async move {
                $future.await
            }).unwrap()
        }};
    }

    #[macro_export]
    macro_rules! rumtk_create_task {
        ( $func:expr, $args:expr ) => {{
            async move {
                let f = $func;
                f(&$args).await
            }
        }};
    }

    #[macro_export]
    macro_rules! rumtk_create_task_args {
        ( $($args:expr),+ ) => {{
            use $crate::threading::thread_primitives::{TaskArgs, SafeTaskArgs, TaskItems};
            use tokio::sync::RwLock;
            SafeTaskArgs::new(RwLock::new(vec![$($args),+]))
        }};
    }

    #[macro_export]
    macro_rules! rumtk_sleep {
        ( $dur:expr) => {{
            use $crate::threading::threading_functions::{sleep};
            sleep($dur as f32)
        }};
    }

    #[macro_export]
    macro_rules! rumtk_async_sleep {
        ( $dur:expr) => {{
            use $crate::threading::threading_functions::{async_sleep};
            async_sleep($dur as f32)
        }};
    }
}
