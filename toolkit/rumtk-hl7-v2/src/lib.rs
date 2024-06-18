pub mod hl7_v2_parser;
pub mod hl7_v2_interpreter;
mod hl7_v2_constants;
pub mod hl7_v2_types;
extern crate rumtk_core;


/*****************************************Tests****************************************/
#[cfg(test)]
mod tests {
    use super::*;
    use hl7_v2_parser;

    /**********************************Constants**************************************/
    const DEFAULT_HL7_V2_MESSAGE: &str =
        "MSH|^~\\&|ADT1|GOOD HEALTH HOSPITAL|GHH LAB, INC.|GOOD HEALTH HOSPITAL|198808181126|SECURITY|ADT^A01^ADT_A01|MSG00001|P|2.8||\r\n
         EVN|A01|200708181123||\n
         PID|1||PATID1234^5^M11^ADT1^MR^GOOD HEALTH HOSPITAL~123456789^^^USSSA^SS||EVERYMAN^ADAM^A^III||19610615|M||C|2222 HOME STREET^^GREENSBORO^NC^27401-1020|GL|(555) 555-2004|(555)555-2004||S||PATID12345001^2^M10^ADT1^AN^A|444333333|987654^NC|\r
         NK1|1|NUCLEAR^NELDA^W|SPO^SPOUSE||||NK^NEXT OF KIN\n\r
         PV1|1|I|2000^2012^01||||004777^ATTEND^AARON^A|||SUR||||ADM|A0|";

    /*********************************Test Cases**************************************/

    #[test]
    fn test_load_hl7_v2_message() {
        let message = hl7_v2_parser::v2_parser::V2Message::from(tests::DEFAULT_HL7_V2_MESSAGE).unwrap();
        let message_segment = String::from("MSH");
        let test_str = String::from("");
        //assert_eq!(message.get(message_segment), test_str);
    }
}
