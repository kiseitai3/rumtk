mod net;
mod log;
pub mod strings;
pub mod maths;
pub mod cache;
pub mod search;

#[cfg(test)]
mod tests {
    use compact_str::CompactString;
    use crate::strings::{RUMString, UTFStringExtensions};
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
        let result = input;
        println!("Input: {} Expected: {} Got: {}", input, input, result);
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
    fn test_search_string_groups() {
        let input = "Hello World!";
        let expr = r"(?<hello>\w{5}) (?<world>\w{5})";
        let result = string_search_captures(input, expr, "");
        let expected: RUMString = RUMString::from("World");
        println!("Input: {:?} Expected: {:?} Got: {:?}", input, expected, result);
        assert_eq!(expected, result["world"], "String search results mismatch");
        println!("Passed!")
    }
}
