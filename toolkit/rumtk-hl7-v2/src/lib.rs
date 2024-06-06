pub mod hl7_v2_parser;
mod hl7_v2_interpreter;
mod hl7_v2_constants;
mod hl7_v2_types;
extern crate rumtk_core;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
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
