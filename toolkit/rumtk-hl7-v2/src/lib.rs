pub mod hl7_v2_parser;
pub mod hl7_v2_interpreter;
mod hl7_v2_constants;
pub mod hl7_v2_types;
extern crate rumtk_core;


/*****************************************Tests****************************************/
#[cfg(test)]
mod tests {
    use super::*;
    use hl7_v2_parser::v2_parser::*;

    /**********************************Constants**************************************/
    const DEFAULT_HL7_V2_MESSAGE: &str =
        "MSH|^~\\&|ADT1|GOOD HEALTH HOSPITAL|GHH LAB, INC.|GOOD HEALTH HOSPITAL|198808181126|SECURITY|ADT^A01^ADT_A01|MSG00001|P|2.8||\r\n\
         EVN|A01|200708181123||\n\
         PID|1||PATID1234^5^M11^ADT1^MR^GOOD HEALTH HOSPITAL~123456789^^^USSSA^SS||EVERYMAN^ADAM^A^III||19610615|M||C|2222 HOME STREET^^GREENSBORO^NC^27401-1020|GL|(555) 555-2004|(555)555-2004||S||PATID12345001^2^M10^ADT1^AN^A|444333333|987654^NC|\r\
         NK1|1|NUCLEAR^NELDA^W|SPO^SPOUSE||||NK^NEXT OF KIN\n\r\
         PV1|1|I|2000^2012^01||||004777^ATTEND^AARON^A|||SUR||||ADM|A0|";
    const DEFAULT_HL7_V2_FIELD_STRING: &str = "2000^2012^01";

    /*********************************Test Cases**************************************/
    #[test]
    fn test_hl7_v2_field_parsing() {
        let field_str = tests::DEFAULT_HL7_V2_FIELD_STRING;
        let encode_chars = V2ParserCharacters::new();
        let field = V2Field::from_string(&field_str, &encode_chars);
        println!("{:#?}", &field);
        assert_eq!(field.len(), 3, "Wrong number of components in field");
        println!("Value in component {} => {}!", 0, field.get(0).unwrap().as_str());
        assert_eq!(field.get(0).unwrap().as_str(), "2000", "Wrong value in component!");
        println!("Value in component {} => {}!", 1, field.get(1).unwrap().as_str());
        assert_eq!(field.get(1).unwrap().as_str(), "2012", "Wrong value in component!");
        println!("Value in component {} => {}!", 2, field.get(2).unwrap().as_str());
        assert_eq!(field.get(2).unwrap().as_str(), "01", "Wrong value in component!");
    }

    #[test]
    fn test_sanitize_hl7_v2_message() {
        let message = tests::DEFAULT_HL7_V2_MESSAGE;
        let sanitized_message = V2Message::sanitize(message);
        println!("{}",message);
        println!("{}",sanitized_message);
        assert!(message.contains('\n'), "Raw message has new line characters.");
        assert!(!sanitized_message.contains('\n'), "Sanitized message has new line characters.");
        assert!(!sanitized_message.contains("\r\r"), "Sanitizer failed to consolidate double carriage returns into a single carriage return per instance..");
    }

    #[test]
    fn test_tokenize_hl7_v2_message() {
        let message = tests::DEFAULT_HL7_V2_MESSAGE;
        let sanitized_message = V2Message::sanitize(message);
        let tokens = V2Message::tokenize_segments(&sanitized_message.as_str());
        println!("Token count {}", tokens.len());
        assert_eq!(tokens.len(), 5, "Tokenizer generated the wrong number of tokens! We expected 5 segment tokens.");
    }

    #[test]
    fn test_load_hl7_v2_encoding_characters() {
        let message = tests::DEFAULT_HL7_V2_MESSAGE;
        let sanitized_message = V2Message::sanitize(message);
        let tokens = V2Message::tokenize_segments(&sanitized_message.as_str());
        let encode_chars = V2ParserCharacters::from_msh(tokens[0]).unwrap();
        println!("{:#?}", encode_chars);
        assert!(encode_chars.segment_terminator.contains('\r'), "Wrong segment character!");
        assert!(encode_chars.field_separator.contains('|'), "Wrong field character!");
        assert!(encode_chars.component_separator.contains('^'), "Wrong component character!");
        assert!(encode_chars.repetition_separator.contains('~'), "Wrong repetition character!");
        assert!(encode_chars.escape_character.contains('\\'), "Wrong escape character!");
        assert!(encode_chars.subcomponent_separator.contains('&'), "Wrong subcomponent character!");
        assert!(encode_chars.truncation_character.contains('#'), "Wrong truncation character!");
    }

    #[test]
    fn test_extract_hl7_v2_message_segments() {
        let message = tests::DEFAULT_HL7_V2_MESSAGE;
        let sanitized_message = V2Message::sanitize(message);
        let tokens = V2Message::tokenize_segments(&sanitized_message.as_str());
        let encode_chars = V2ParserCharacters::from_msh(tokens[0]).unwrap();
        let parsed_segments = V2Message::extract_segments(&tokens, &encode_chars).unwrap();
        print!("Keys: ");
        for k in parsed_segments.keys() {
            print!("{} ", k);
        }
    }

    #[test]
    fn test_load_hl7_v2_message() {
        let message = V2Message::from(tests::DEFAULT_HL7_V2_MESSAGE).unwrap();
    }
}
