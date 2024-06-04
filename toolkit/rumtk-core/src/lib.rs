mod net;
mod log;
mod strings;


#[cfg(test)]
mod tests {
    use rumtk_hl7_v2::hl7_v2_parser;
    use super::*;

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

    #[test]
    fn test_load_hl7_v2_message() {
        let message = hl7_v2_parser::new(tests::DEFAULT_HL7_V2_MESSAGE);
        let message_segment = String::from("MSH");
        let test_str = String::from("");
        assert_eq!(message.get(message_segment), test_str);
    }

    #[test]
    fn test_load_hl7_v2_message_segment() {
        let message = hl7_v2_parser::new(tests::DEFAULT_HL7_V2_MESSAGE);
        let message_segment = String::from("MSH");
        let test_str = String::from("");
        assert_eq!(message.get(message_segment), test_str);
    }

    #[test]
    fn test_load_hl7_v2_message_segment_field() {
        let message = hl7_v2_parser::new(tests::DEFAULT_HL7_V2_MESSAGE);
        let message_segment = String::from("MSH-1");
        let test_str = String::from("");
        assert_eq!(message.get(message_segment), test_str);
    }

    #[test]
    fn test_load_hl7_v2_message_segment_field_node() {
        let message = hl7_v2_parser::new(tests::DEFAULT_HL7_V2_MESSAGE);
        let message_segment = String::from("PID-3.2");
        let test_str = String::from("");
        assert_eq!(message.get(message_segment), test_str);
    }
}
