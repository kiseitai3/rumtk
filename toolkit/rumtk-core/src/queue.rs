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
    use tokio::task::JoinHandle as TokioHandle;
    use std::thread::JoinHandle as ThreadHandle;
    use std::thread::{spawn, Result, sleep, Duration};
    use crate::strings::RUMString;

    const SLEEP_DURATION: Duration = Duration::from_millis(1);

    pub type TaskProcessor<T> = fn(buf: T) -> Result<Vec<T>>;

    pub struct Task<T>
    {
        task_processor: TaskProcessor<T>,
        data: Vec<T>
    }

    impl<T> Task<T>{
        pub fn new(data: T, task_processor: TaskProcessor<T>) -> Task<T> {
            Task{task_processor, data}
        }
    }

    type TaskResult<T> = Result<Vec<T>>;

    pub struct TaskQueue<T> {
        tasks: VecDeque<Task<T>>,
        queued: usize
    }

    impl<T> TaskQueue<T> {
        pub fn new() -> TaskQueue<T> {
            TaskQueue{tasks: VecDeque::new(), queued: 0}
        }

        pub fn queue(&mut self, task: Task<T>) {
            self.tasks.push_back(task);
            self.queued += 1;
        }

        pub fn dequeue(&mut self) -> Option<Task<T>> {
            self.tasks.pop_front()
        }

        pub fn task_done(&mut self) {
            if self.queued == 0 {
                panic!("TaskQueue::task_done called more times than queued tasks!");
            }

            self.queued -= 1;
        }

        pub fn is_empty(&self) -> bool { self.queued > 0 }
    }

    pub struct ThreadedTaskQueue<T> {
        queue: TaskQueue<T>,
        results: VecDeque<TaskResult<T>>,
        workers: Vec<ThreadHandle<TaskProcessor<T>>>
    }

    impl<T> ThreadedTaskQueue<T> {
        pub fn new(worker_num: u8) -> ThreadedTaskQueue<T> {
            let mut workers = vec![];

            for i in 0..worker_num {
                workers.push(spawn(Self::worker))
            }

            ThreadedTaskQueue{workers, queue: TaskQueue::new(), results: VecDeque::new()}
        }

        fn worker(&mut self) {
            loop {
                let task = self.queue.dequeue();

                match task {
                    Some(task) => {
                        let processor = task.task_processor;
                        self.results.push_back(processor(task.data));
                        self.queue.task_done();
                    }
                    _ => {}
                }
                // Rest briefly
                sleep(SLEEP_DURATION);
            }
        }
        
        pub fn add_task(&mut self, task: Task<T>) {
            self.queue.queue(task);
        }

        pub fn wait(&mut self) -> VecDeque<TaskResult<T>> {

            while !self.queue.is_empty() {
                sleep(SLEEP_DURATION);
            }

            let results = self.results.clone();
            self.results.clear();
            results
        }
    }
}
