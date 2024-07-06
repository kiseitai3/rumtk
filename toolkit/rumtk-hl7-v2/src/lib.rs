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
    const VXU_HL7_V2_MESSAGE: &str =
        "MSH|^~\\&|NISTEHRAPP|NISTEHRFAC|NISTIISAPP|NISTIISFAC|20150625072816.601-0500||VXU^V04^VXU_V04|NIST-IZ-AD-10.1_Send_V04_Z22|P|2.5.1|||ER|AL|||||Z22^CDCPHINVS|NISTEHRFAC|NISTIISFAC\n
        PID|1||21142^^^NIST-MPI-1^MR||Vasquez^Manuel^Diego^^^^L||19470215|M||2106-3^White^CDCREC|227 Park Ave^^Bozeman^MT^59715^USA^P||^PRN^PH^^^406^5555815~^NET^^Manuel.Vasquez@isp.com|||||||||2135-2^Hispanic or Latino^CDCREC||N|1|||||N\n
        PD1|||||||||||01^No reminder/recall^HL70215|N|20150625|||A|20150625|20150625ORC|RE||31165^NIST-AA-IZ-2|||||||7824^Jackson^Lily^Suzanne^^^^^NIST-PI-1^L^^^PRN|||||||NISTEHRFAC^NISTEHRFacility^HL70362\n
        RXA|0|1|20141021||152^Pneumococcal Conjugate, unspecified formulation^CVX|999|||01^Historical Administration^NIP001|||||||||||CP|A";
    const HL7_V2_MESSAGE: &str =
        "FHS|^~\\&|WIR11.3.2|WIR|||20200514||1219274.update|||\n
        BHS|^~\\&|WIR11.3.2|WIR|||20200514|||||\n
        MSH|^~\\&|WIR11.3.2^^|WIR^^||WIRPH^^|20200514||VXU^V04^VXU_V04|2020051412382900|P^|2.5.1^^|||ER||||||^CDCPHINVS\n
        PID|||3064985^^^^SR^~ML288^^^^PI^||CHILD^BABEE^^^^^^||20180911|F||2106-3^^^^^|22 YOUNGER LAND^^JUNEAU^WI^53039^^^^WI027^^||(920)386-5555^PRN^PH^^^920^3865555^^|||||||||2186-5^^^^^|||||||\n
        PD1|||||||||||02^^^^^|U||||A\n
        NK1|1|CHILD^MARY^^^^^^|PAR^PARENT^HL70063^^^^^|22 YOUNGER LAND^^JUNEAU^WI^53039^^^^^^|(920)386-5555^PRN^PH^^^920^3865555^^\n
        NK1|2|CHILD^BABEE^^^^^^|SEL^SELF^HL70063^^^^^|22 YOUNGER LAND^^JUNEAU^WI^53039^^^^^^|(920)386-5555^PRN^PH^^^920^3865555^^\n
        PV1||R||||||||||||||||||\n
        ORC|RE||0|\n
        RXA|0|999|20190503|20190503|110^DTAP/Polio/Hep B^CVX^Pediarix^Pediarix^WVTN|1.0|||01^^^^^~38230637^WIR immunization id^IMM_ID^^^||^^^WIR Physicians^^^^^^^^^^^|||||||||\n
        BTS|1|\n
        FTS|1|";
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
        let msh = V2Message::find_msh(&tokens).unwrap();
        let encode_chars = V2ParserCharacters::from_msh(&msh.as_str()).unwrap();
        let parsed_segments = V2Message::extract_segments(&tokens, &encode_chars).unwrap();
        let keys = parsed_segments.keys();
        print!("Keys: ");
        for k in keys {
            print!("{} ", k);
        }
        assert_eq!(parsed_segments.len(), 5, "Number of segments mismatching what was expected!");
        assert!(parsed_segments.contains_key("MSH"), "Missing MSH segment!");
        assert!(parsed_segments.contains_key("PID"), "Missing PID segment!");
        assert!(parsed_segments.contains_key("PV1"), "Missing PV1 segment!");
        assert!(parsed_segments.contains_key("EVN"), "Missing EVN segment!");
        assert!(parsed_segments.contains_key("NK1"), "Missing NK1 segment!");
    }

    #[test]
    fn test_load_hl7_v2_message() {
        let message = V2Message::from(tests::DEFAULT_HL7_V2_MESSAGE).unwrap();
        assert!(message.segment_exists("MSH"), "Missing MSH segment!");
        assert!(message.segment_exists("PID"), "Missing PID segment!");
        assert!(message.segment_exists("PV1"), "Missing PV1 segment!");
        assert!(message.segment_exists("EVN"), "Missing EVN segment!");
        assert!(message.segment_exists("NK1"), "Missing NK1 segment!");
    }

    #[test]
    fn test_load_hl7_v2_message_wir_iis() {
        /*
        Per examples in https://confluence.hl7.org/display/OO/v2+Sample+Messages you can have
        messages that have other header segments before the standard MSH header.
        As a result, I have made the logic a bit more permissive of the position of the msh segment.
        I also made sure segments were trimmed to avoid issues with white space padding
         */
        let message = V2Message::from(tests::HL7_V2_MESSAGE).unwrap();
        assert!(message.segment_exists("MSH"), "Missing MSH segment!");
        assert!(message.segment_exists("FHS"), "Missing FHS segment!");
        assert!(message.segment_exists("NK1"), "Missing NK1 segment!");
        assert!(message.segment_exists("PV1"), "Missing PV1 segment!");
        assert!(message.segment_exists("FTS"), "Missing FTS segment!");
        assert!(message.segment_exists("BHS"), "Missing BHS segment!");
    }
}
