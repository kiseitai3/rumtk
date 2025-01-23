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
pub mod queue {
    use std::sync::Mutex;
    use std::time::Duration;
    use std::thread::{sleep};
    pub use crate::threading::thread_primitives::*;

    pub const DEFAULT_SLEEP_DURATION: Duration = Duration::from_millis(1);
    pub const DEFAULT_QUEUE_CAPACITY: usize = 10;
    pub const DEFAULT_MICROTASK_QUEUE_CAPACITY: usize = 5;


    type TaskQueueData<T, R> = SafeTasks<T, R>;

    pub struct TaskQueue<T, R> {
        tasks: TaskQueueData<T, R>,
        threads: ThreadPool<T, R>
    }

    impl<T, R> TaskQueue<T, R>
    where
        T: Send + Sync + Clone + 'static,
        R: Send + Sync + Clone + 'static,
        Box<T>: Send + Sync + Clone + 'static,
    {
        ///
        /// This method creates a [`TaskQueue`] instance using sensible defaults.
        ///
        /// The `worker_num` parameter is computed from the number of cores present in system.
        /// The `microtask_queue` is set to [`DEFAULT_MICROTASK_QUEUE_CAPACITY`].
        /// The `microtask_queue` is set to [`DEFAULT_QUEUE_CAPACITY`].
        ///
        pub fn default() -> TaskQueue<T, R> {
            Self::new(5)
        }

        ///
        /// Creates an instance of [`ThreadedTaskQueue<T, R>`] in the form of [`SafeThreadedTaskQueue<T, R>`].
        /// Expects you to provide the count of threads to spawn and the microtask queue size
        /// allocated by each thread.
        ///
        /// This method calls [`Self::with_capacity()`] for the actual object creation.
        /// The main queue capacity is pre-allocated to [`DEFAULT_QUEUE_CAPACITY`].
        ///
        pub fn new(worker_num: usize) -> TaskQueue<T, R> {
            let tasks = SafeTasks::with_capacity(DEFAULT_QUEUE_CAPACITY);
            let threads = ThreadPool::new(worker_num);
            TaskQueue{tasks, threads}
        }

        ///
        /// Add a task to the processing queue. The idea is that you can queue a processor function
        /// and list of args that will be picked up by one of the threads for processing.
        ///
        pub fn add_task(&mut self, processor: TaskProcessor<T, R>, args: SafeTaskArgs<T>) {
            let task = Task::new(processor, args);
            let safe_task = SafeTask::new(Mutex::new(task));
            self.threads.execute(&safe_task);
            self.tasks.push(safe_task);
        }

        ///
        /// This method waits until all queued tasks have been processed from the main queue.
        ///
        /// We poll the status of the main queue every [`DEFAULT_SLEEP_DURATION`] ms.
        ///
        /// Upon completion,
        ///
        /// 1. We collect the results generated (if any).
        /// 2. We reset the main task and result internal queue states.
        /// 3. Return the list of results ([`TaskResults<R>`]).
        ///
        /// ### Note:
        ///
        ///     Results returned here are not guaranteed to be in the same order as the order in which
        ///     the tasks were queued for work. You will need to pass a type as T that automatically
        ///     tracks its own id or has a way for you to resort results.
        ///
        pub fn wait(&mut self) -> TaskResults<R> {
            while !self.is_completed() {
                sleep(DEFAULT_SLEEP_DURATION);
            }

            let results = self.gather();
            self.reset();
            results
        }

        ///
        /// Check if all work has been completed from the task queue.
        ///
        pub fn is_completed(&self) -> bool {
            for task in self.tasks.iter() {
                if !task.lock().unwrap().is_completed() {
                    return false;
                }
            }
            true
        }

        ///
        /// Reset task queue and results queue states.
        ///
        pub fn reset(&mut self) {
            self.tasks.clear();
        }

        fn gather(&mut self) -> TaskResults<R> {
            let mut result_queue = TaskResults::<R>::with_capacity(self.tasks.len());
            for task in self.tasks.iter() {
                result_queue.push(task.lock().unwrap().get_result().clone());
            }
            result_queue
        }
    }
}

pub mod queue_macros {
    #[macro_export]
    macro_rules! rumtk_new_task_queue {
        ( $worker_num:expr ) => {{
            use $crate::queue::queue::{TaskQueue};
            TaskQueue::new($worker_num);
        }};
    }
}
