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
    use std::any::Any;
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};
    use tokio::task::JoinHandle as AsyncHandle;
    use tokio::task::spawn as async_spawn;
    use std::thread::JoinHandle as ThreadHandle;
    use std::thread::spawn as thread_spawn;
    use std::thread::{Result, sleep, Duration};
    use crate::strings::RUMString;

    const SLEEP_DURATION: Duration = Duration::from_millis(1);


    pub type TaskItems<T> = Vec<T>;
    /// This type aliases a vector of T elements that will be used for passing arguments to the task processor.
    pub type TaskArgs<T> = TaskItems<T>;
    /// Type to use to define how task results are expected to be returned.
    pub type TaskResult<T> = Result<TaskItems<T>>;
    /// Function signature defining the interface of task processing logic.
    pub type TaskProcessor<T> = fn(args: &TaskArgs<T>) -> TaskResult<T>;

    #[derive(Clone, Debug)]
    pub struct Task<T>
    {
        task_processor: TaskProcessor<T>,
        args: TaskArgs<T>,
        result: TaskResult<T>
    }

    impl<T> Task<T>{
        pub fn new(task_processor: TaskProcessor<T>, args: TaskArgs<T>) -> Task<T> {
            let result = Ok(TaskItems::new());
            Task{task_processor, args, result}
        }

        pub fn exec(&mut self) {
            let processor = &self.task_processor;
            self.result = processor(&self.args);
        }
    }

    type TaskQueueData<T> = VecDeque<Task<T>>;
    type AvailableTaskData<T> = VecDeque<Option<Task<T>>>;

    #[derive(Debug)]
    pub struct TaskQueue<T> {
        tasks: TaskQueueData<T>,
        queued: usize
    }

    impl<T> TaskQueue<T> {
        pub fn new() -> TaskQueue<T> {
            Self::with_capacity(10)
        }

        pub fn with_capacity(initial_size: usize) -> TaskQueue<T> {
            TaskQueue{tasks: VecDeque::with_capacity(initial_size), queued: 0}
        }

        pub fn queue(&mut self, task: Task<T>) {
            self.tasks.push_back(task);
            self.queued += 1;
        }

        pub fn queue_batch(&mut self, tasks: AvailableTaskData<T>) {
            for task in tasks {
                match task {
                    Some(task) => self.queue(task),
                    None => ()
                }
            }
        }

        pub fn dequeue(&mut self) -> Option<Task<T>> {
            self.tasks.pop_front()
        }

        pub fn task_done(&mut self) {
            if self.queued <= 0 {
                panic!("TaskQueue::task_done called more times than queued tasks!");
            }

            self.queued -= 1;
        }

        pub fn is_empty(&self) -> bool { self.queued > 0 }
        pub fn queued(&self) -> usize { self.queued }
    }

    impl<T> Iterator for TaskQueue<T> {
        type Item = Task<T>;

        fn next(&mut self) -> Option<Task<T>> {
            self.tasks.pop_front()
        }
    }

    pub type SafeTaskQueue<T> = Arc<Mutex<TaskQueue<T>>>;
    pub type SafeTaskResults<T> = Arc<Mutex<TaskQueue<T>>>;

    pub struct ThreadedTaskQueue<T> {
        queue: SafeTaskQueue<T>,
        results: SafeTaskResults<T>,
        workers: Vec<ThreadHandle<TaskProcessor<T>>>,
        microtask_size: usize
    }

    impl<T> ThreadedTaskQueue<T> {
        pub fn new(worker_num: usize, microtask_size: usize) -> ThreadedTaskQueue<T> {
            Self::with_capacity(worker_num, microtask_size, 10)
        }

        pub fn with_capacity(worker_num: usize, microtask_size: usize, capacity: usize) -> ThreadedTaskQueue<T> {
            let mut workers = Vec::with_capacity(worker_num);

            for i in 0..worker_num {
                workers.push(thread_spawn(Self::worker))
            }

            let queue = SafeTaskQueue::new(Mutex::new(TaskQueue::<T>::new()));
            let results = SafeTaskQueue::new(Mutex::new(TaskQueue::<T>::with_capacity(capacity)));
            ThreadedTaskQueue{workers, microtask_size, queue, results}
        }

        fn worker(&mut self) {
            // Init worker
            let micro_runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            let microtask_size = self.microtask_size.clone();
            let mut microtask_queue = TaskQueue::<T>::with_capacity(microtask_size);
            let mut async_handles = Vec::<AsyncHandle<T>>::with_capacity(microtask_size);

            // Begin processing
            loop {
                microtask_queue.queue_batch(self.take_tasks(microtask_size));

                // Begin scheduling tasks
                for mut task in microtask_queue {

                    async_handles.push(
                        micro_runtime.spawn(
                            async move || {
                                task.exec();
                                self.return_result(task);
                            }
                        )
                    );
                }

                // Wait on all tasks to complete
                for handle in async_handles {
                    micro_runtime.block_on(handle).unwrap();
                }

                // Rest briefly
                sleep(SLEEP_DURATION);
            }
        }

        pub fn add_task(&mut self, processor: TaskProcessor<T>, args: TaskArgs<T>) {
            self.queue.queue(Task::new(processor, args));
        }

        pub fn wait(&mut self) -> VecDeque<TaskResult<T>> {

            while !self.queue.is_empty() {
                sleep(SLEEP_DURATION);
            }

            let results = self.results.clone();
            self.results.clear();
            results
        }

        fn take_tasks(&mut self, n: usize) -> AvailableTaskData<T> {
            let mut tasks = self.queue.lock().unwrap();
            let mut taken = AvailableTaskData::with_capacity(n);
            for i in 0..n {
                taken.push_back(*tasks.dequeue());
            }
            taken
        }

        fn return_result(&mut self, task: Task<T>) {
            let mut results = self.results.lock().unwrap();
            *results.queue(task);
        }
    }
}
