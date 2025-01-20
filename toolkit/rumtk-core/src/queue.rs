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
    use std::time::Duration;
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex, RwLock};
    use tokio::task::JoinHandle as AsyncHandle;
    use tokio::task::spawn as async_spawn;
    use std::thread::JoinHandle as ThreadHandle;
    use std::thread::spawn as thread_spawn;
    use std::thread::{sleep};
    use compact_str::format_compact;
    use crate::core::RUMResult;
    use crate::strings::RUMString;

    const SLEEP_DURATION: Duration = Duration::from_millis(1);

    //static GLOBAL_Threads: Arc<Mutex<Vec<ThreadHandle<()>>>> = Arc::new(Mutex::new(vec![]));


    pub type TaskItems<T> = Vec<T>;
    /// This type aliases a vector of T elements that will be used for passing arguments to the task processor.
    pub type TaskArgs<T> = TaskItems<T>;
    /// Type to use to define how task results are expected to be returned.
    pub type TaskResult<R> = RUMResult<TaskItems<R>>;
    pub type TaskResults<R> = TaskItems<TaskResult<R>>;
    /// Function signature defining the interface of task processing logic.
    pub type TaskProcessor<T, R> = fn(args: &TaskArgs<T>) -> TaskResult<R>;

    #[derive(Debug, Clone)]
    pub struct Task<T, R>
    {
        task_processor: TaskProcessor<T, R>,
        args: TaskArgs<T>,
        result: TaskResult<R>
    }

    impl<T, R> Task<T, R>
    where
        T: Send + Clone +'static,
        R: Send + Clone + 'static,
        Box<T>: Send + Clone + 'static,
    {
        pub fn new(task_processor: TaskProcessor<T, R>, args: TaskArgs<T>) -> Task<T, R> {
            let result = Ok(TaskItems::with_capacity(5));
            Task{task_processor, args, result}
        }

        pub fn exec(&mut self) {
            let processor = &self.task_processor;
            self.result = processor(&self.args);
        }

        pub fn get_result(&self) -> &TaskResult<R> {
            &self.result
        }
    }

    type TaskQueueData<T, R> = VecDeque<Task<T, R>>;
    type AvailableTaskData<T, R> = VecDeque<Option<Task<T, R>>>;

    #[derive(Debug)]
    pub struct TaskQueue<T, R> {
        tasks: TaskQueueData<T, R>,
        queued: usize
    }

    impl<T, R> TaskQueue<T, R>
    where
        T: Send + Clone +'static,
        R: Send + Clone + 'static,
        Box<T>: Send + Clone + 'static,
    {
        pub fn new() -> TaskQueue<T, R> {
            Self::with_capacity(10)
        }

        pub fn with_capacity(initial_size: usize) -> TaskQueue<T, R> {
            TaskQueue{tasks: VecDeque::with_capacity(initial_size), queued: 0}
        }

        pub fn add_task(&mut self, processor: TaskProcessor<T, R>, args: TaskArgs<T>) {
            self.queue(Task::new(processor, args));
        }

        pub fn queue(&mut self, task: Task<T, R>) {
            self.tasks.push_back(task);
            self.queued += 1;
        }

        pub fn queue_batch(&mut self, tasks: AvailableTaskData<T, R>) {
            for task in tasks {
                match task {
                    Some(task) => self.queue(task),
                    None => ()
                }
            }
        }

        pub fn dequeue(&mut self) -> Option<Task<T, R>> {
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

        pub fn clear(&mut self) {
            self.tasks.clear();
        }
    }

    impl<T, R> Iterator for TaskQueue<T, R>
    where
        T: Send + Clone +'static,
        R: Send + Clone + 'static,
        Box<T>: Send + Clone + 'static,
    {
        type Item = Task<T, R>;

        fn next(&mut self) -> Option<Task<T, R>> {
            self.tasks.pop_front()
        }
    }

    pub type SafeTaskQueue<T, R> = Arc<Mutex<TaskQueue<T, R>>>;
    pub type SafeTaskResults<R> = Arc<Mutex<TaskResults<R>>>;
    pub type ThreadPool = Vec<ThreadHandle<()>>;
    pub type SafeThreadedTaskQueue<T, R> = Arc<Mutex<ThreadedTaskQueue<T, R>>>;

    #[derive(Clone)]
    pub struct ThreadedTaskQueue<T, R> {
        queue: SafeTaskQueue<T, R>,
        results: SafeTaskResults<R>,
        worker_num: usize,
        microtask_size: usize
    }

    impl<T, R> ThreadedTaskQueue<T, R>
    where
        T: Send + Clone +'static,
        R: Send + Clone + 'static,
        Box<T>: Send + Clone + 'static,
    {
        pub fn new(worker_num: usize, microtask_size: usize) -> SafeThreadedTaskQueue<T, R> {
            Self::with_capacity(worker_num, microtask_size, 10)
        }

        pub fn with_capacity(worker_num: usize, microtask_size: usize, capacity: usize) -> SafeThreadedTaskQueue<T, R> {
            let queue = SafeTaskQueue::new(Mutex::new(TaskQueue::new()));
            let results = SafeTaskResults::new(Mutex::new(TaskResults::with_capacity(capacity)));
            let ttq = ThreadedTaskQueue{worker_num, microtask_size, queue, results};
            let mut safe_ttq = SafeThreadedTaskQueue::new(Mutex::new(ttq));
            Self::start(&mut safe_ttq);
            safe_ttq
        }

        pub fn start(this: &mut SafeThreadedTaskQueue<T, R>) -> ThreadPool {
            let mut safe_self = this.lock().unwrap();
            let mut handles = ThreadPool::with_capacity(safe_self.worker_num);

            for _ in 0..safe_self.worker_num {
                let mut copy = this.clone();
                handles.push(thread_spawn(move || copy.lock().unwrap().worker()));
            }

            handles
        }

        fn worker(&mut self) {
            // Init worker
            let micro_runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            let microtask_size = self.microtask_size.clone();

            // Begin processing
            loop {
                let mut microtask_queue = TaskQueue::<T, R>::with_capacity(microtask_size);
                let mut async_handles = Vec::<AsyncHandle<()>>::with_capacity(microtask_size);
                microtask_queue.queue_batch(self.take_tasks(microtask_size));

                // Begin scheduling tasks
                for mut task in microtask_queue {
                    let future =
                        micro_runtime.spawn(
                            async move {
                                task.exec();
                            }
                        );

                    async_handles.push(future);
                }

                // Wait on all tasks to complete
                for handle in async_handles {
                    micro_runtime.block_on(handle).unwrap();
                }

                // Rest briefly
                sleep(SLEEP_DURATION);
            }
        }

        pub fn add_task(&mut self, processor: TaskProcessor<T, R>, args: TaskArgs<T>) {
            let mut queue  = self.queue.lock().unwrap();
            queue.add_task(processor, args);
        }

        pub fn wait(&mut self) -> TaskResults<R> {
            while !self.is_empty() {
                sleep(SLEEP_DURATION);
            }

            let results = self.collect_results();
            self.reset();
            results
        }

        pub fn is_empty(&self) -> bool {
            let queue  = self.queue.lock().unwrap();
            queue.is_empty()
        }
        pub fn reset(&mut self) {
            let mut queue  = self.queue.lock().unwrap();
            let mut results  = self.results.lock().unwrap();
            queue.clear();
            results.clear();
        }

        fn take_tasks(&mut self, n: usize) -> AvailableTaskData<T, R> {
            let mut tasks = self.queue.lock().unwrap();
            let mut taken = AvailableTaskData::with_capacity(n);
            for _ in 0..n {
                taken.push_back(tasks.dequeue());
            }
            taken
        }

        fn collect_results(&self) -> TaskResults<R> {
            let result_queue = self.results.lock().unwrap();
            (*result_queue).as_slice().to_vec()
        }
    }
}
