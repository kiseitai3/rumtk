#![feature(unboxed_closures)]
/*
 * rumtk attempts to implement HL7 and medical protocols for interoperability in medicine.
 * This toolkit aims to be reliable, simple, performant, and standards compliant.
 * Copyright (C) 2024  Luis M. Santos, M.D.
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


pub mod net;
pub mod log;
pub mod strings;
pub mod maths;
pub mod cache;
pub mod search;
pub mod queue;
pub mod core;
pub mod threading;

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::time::Duration;
    use compact_str::{format_compact, CompactString};
    use tokio::time::sleep;
    use crate::strings::{RUMString, RUMStringConversions, UTFStringExtensions};
    use crate::search::rumtk_search::*;
    use crate::cache::RUMCache;
    use super::*;

    /*
    #[test]
    fn test_server_listen() {
        let server = net::TCPSever::new();
        server.start();
        //assert_eq!(result, 4);
    }

    #[test]
    fn test_client_send() {
        let test_str = String::from("Hello World!");
        let server = net::TCPSever::new();
        let client = net::TCPClient::new();
        let port = 55555;
        server.start(port);
        client.connect(port);
        client.send(&test_str.as_bytes());
        let result = String::from(server.pop());
        assert_eq!(result, test_str);
    }

    #[test]
    fn test_log_to_file() {
        let logger_name = String::from("test_logger");
        let logger_path = String::from("logs");
        let logger = log::new_logger(logger_path, logger_name, log::LOGLEVEL::INFO);
        let test_str = String::from("Hello World!");
        log::log_info(test_str);
        assert_eq!(result, test_str);
    }
    */
    #[test]
    fn test_escaping_control() {
        let input = "\r\n\'\"";
        let expected = "\\r\\n\\'\\\"";
        let result = strings::escape(&input);
        println!("Input: {} Expected: {} Got: {}", input, expected, result.as_str());
        assert_eq!(expected, result, "Incorrect string escaping!");
        println!("Passed!")
    }

    #[test]
    fn test_escaping_unicode() {
        let input = "❤";
        let expected = "\\u2764";
        let result = strings::escape(&input);
        println!("Input: {} Expected: {} Got: {}", input, expected, result.as_str());
        assert_eq!(expected, result, "Incorrect string escaping!");
        println!("Passed!")
    }

    #[test]
    fn test_unescaping_unicode() {
        let input = "❤";
        let escaped = strings::escape(&input);
        let expected = "❤";
        let result = RUMString::from_utf8(strings::unescape(&escaped.as_str()).unwrap()).unwrap();
        println!("Input: {} Expected: {} Got: {}", input, expected, result.as_str());
        assert_eq!(expected, result.as_str(), "Incorrect string unescaping!");
        println!("Passed!")
    }

    #[test]
    fn test_unescaping_string() {
        let input = "I \\u2764 my wife!";
        let expected = "I ❤ my wife!";
        let result = strings::unescape_string(&input).unwrap();
        println!("Input: {} Expected: {} Got: {}", input, expected, result.as_str());
        assert_eq!(expected, result.as_str(), "Incorrect string unescaping!");
        println!("Passed!")
    }

    #[test]
    fn test_escaping_string() {
        let input = "I ❤ my wife!";
        let expected = "I \\u2764 my wife!";
        let result = strings::escape_str(&input);
        println!("Input: {} Expected: {} Got: {}", input, expected, result.as_str());
        assert_eq!(expected, result.as_str(), "Incorrect string escaping!");
        println!("Passed!")
    }

    #[test]
    fn test_autodecode_utf8() {
        let input = "I ❤ my wife!";
        let result = strings::try_decode(input.as_bytes());
        println!("Input: {} Expected: {} Got: {}", input, input, result.as_str());
        assert_eq!(input, result, "Incorrect string decoding!");
        println!("Passed!")
    }

    #[test]
    fn test_autodecode_other() {
        //TODO: Need an example of other encoding texts.
        let input = "I ❤ my wife!";
        let expected = "I ❤ my wife!";
        let result = input;
        println!("Input: {} Expected: {} Got: {}", input, input, result);
        assert_eq!(input, result, "Incorrect string decoding!");
        println!("Passed!")
    }

    #[test]
    fn test_decode() {
        let input = "I ❤ my wife!";
        let expected = "I ❤ my wife!";
        let result = strings::try_decode_with(input.as_bytes(), "utf-8");
        println!("Input: {} Expected: {} Got: {}", input, input, result.as_str());
        assert_eq!(input, result, "Incorrect string decoding!");
        println!("Passed!")
    }

    #[test]
    fn test_rumcache_insertion() {
        let mut cache: RUMCache<&str, CompactString> = RUMCache::with_capacity(5);
        cache.insert("❤", CompactString::from("I ❤ my wife!"));
        println!("Contents: {:#?}", &cache);
        assert_eq!(cache.len(), 1, "Incorrect number of items in cache!");
        println!("Passed!")
    }

    #[test]
    fn test_search_string_letters() {
        let input = "Hello World!";
        let expr = r"\w";
        let result = string_search(input, expr, "");
        let expected: RUMString = RUMString::from("HelloWorld");
        println!("Input: {:?} Expected: {:?} Got: {:?}", input, expected, result);
        assert_eq!(expected, result, "String search results mismatch");
        println!("Passed!")
    }

    #[test]
    fn test_search_string_words() {
        let input = "Hello World!";
        let expr = r"\w+";
        let result = string_search(input, expr, " ");
        let expected: RUMString = RUMString::from("Hello World");
        println!("Input: {:?} Expected: {:?} Got: {:?}", input, expected, result);
        assert_eq!(expected, result, "String search results mismatch");
        println!("Passed!")
    }

    #[test]
    fn test_search_string_named_groups() {
        let input = "Hello World!";
        let expr = r"(?<hello>\w{5}) (?<world>\w{5})";
        let result = string_search_named_captures(input, expr, "");
        let expected: RUMString = RUMString::from("World");
        println!("Input: {:?} Expected: {:?} Got: {:?}", input, expected, result);
        assert_eq!(expected, result["world"], "String search results mismatch");
        println!("Passed!")
    }

    #[test]
    fn test_search_string_all_groups() {
        let input = "Hello World!";
        let expr = r"(?<hello>\w{5}) (?<world>\w{5})";
        let result = string_search_all_captures(input, expr, "");
        let expected: Vec<&str> = vec!["Hello", "World"];
        println!("Input: {:?} Expected: {:?} Got: {:?}", input, expected, result);
        assert_eq!(expected, result, "String search results mismatch");
        println!("Passed!")
    }

    ///////////////////////////////////Threading Tests/////////////////////////////////////////////////
    #[test]
    fn test_default_num_threads() {
        use num_cpus;
        let threads = threading::threading_functions::get_default_system_thread_count();
        assert_eq!(threads >= num_cpus::get(), true, "Default thread count is incorrect! We got {}, but expected {}!", threads, num_cpus::get());
    }

    #[test]
    fn test_create_threadpool() {
        let pool = ThreadPool::<i32, i32>::new(4);
    }

    #[test]
    fn test_create_threadpool_default() {
        let pool = ThreadPool::<i32, i32>::default();
    }

    #[test]
    fn test_execute_job() {
        let expected = vec![
            1,
            2,
            3
        ];
        let task_processor = |args: &SafeTaskArgs<i32>| -> TaskResult<i32> {
            let owned_args = args.lock().unwrap();
            let mut results = TaskItems::<i32>::with_capacity(owned_args.len());
            print!("Contents: ");
            for arg in owned_args.iter() {
                results.push(arg.clone());
                println!("{} ", &arg);
            }
            Ok(results)
        };
        let task_args = SafeTaskArgs::<i32>::new(Mutex::new(expected.clone()));
        let task = SafeTask::<i32, i32>::new(Mutex::new(Task::new(task_processor, task_args)));
        let pool = ThreadPool::<i32, i32>::new(4);
        pool.execute(&task);
        // Let the pool init and threads come online. Otherwise, we end up poisoning the lock...
        std::thread::sleep(Duration::from_millis(1000));
        let result = task.lock().unwrap();
        let completed = &result.is_completed();
        let mut results = TaskItems::<i32>::with_capacity(expected.len());
        for r in result.get_result() {
            for v in r {
                results.push(v.clone());
            }
        }
        assert_eq!(completed, &true, "Task was not processed before finishing test!");
        assert_eq!(&results, &expected, "{}", format_compact!("Task processing returned a different result than expected! Expected {:?} \nResults {:?}", &expected, &results));
    }

    ///////////////////////////////////Queue Tests/////////////////////////////////////////////////
    use queue::queue::*;

    #[test]
    fn test_queue_data() {
        let expected = vec![
            RUMString::from("Hello"),
            RUMString::from("World!"),
            RUMString::from("Overcast"),
            RUMString::from("and"),
            RUMString::from("Sad")
        ];
        let mut queue = TaskQueue::<RUMString, RUMString>::new(5);
        let processor = |args: &SafeTaskArgs<RUMString>| -> TaskResult<RUMString> {
            let owned_args = args.lock().unwrap();
            let mut results = TaskItems::<RUMString>::with_capacity(owned_args.len());
            print!("Contents: ");
            for arg in owned_args.iter() {
                print!("{} ", &arg);
                results.push(RUMString::new(arg));
            }
            Ok(results)
        };
        let task_args = SafeTaskArgs::<RUMString>::new(Mutex::new(expected.clone()));
        queue.add_task(processor, task_args);
        let results = queue.wait();
        let mut result_data = Vec::<RUMString>::with_capacity(5);
        for r in results {
            for v in r.unwrap().iter() {
                result_data.push(v.clone());
            }
        }
        assert_eq!(result_data, expected, "Results do not match expected!");
    }

    //////////////////////////////////////////////////////////////////////////////////////////////
}
