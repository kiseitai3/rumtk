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
    const HL7_V2_PDF_MESSAGE: &str =
        "MSH|^~\\&|SendingApp|SendingFac|ReceivingApp|ReceivingFac|201607060811||ORU^R03|5209141|D|2.3\n
        PID|1|123456|123456||Andrés^Ángel^||19600101|M|||123 BLACK PEARL^^DETROIT^MI^48217||3138363978|||||1036557|123456789\n
        PV1|||^^\n
        ORC|RE||161810162||||00001^ONCE^^^^||201607060811|||00007553^PHYSICIAN^आरवा^ ^^^|||||||||BIOTECH CLINICAL LAB INC^^23D0709666^^^CLIA|25775 MEADOWBROOK^^NOVI^MI^48375|^^^^^248^9121700|OFFICE^1234 MILE ROAD  SUITE # 2^ROYAL OAK^MI^48073\n
        OBR|8|455648^LA01|1618101622^GROUP951|GROUP951^CHROMOSOMAL ANALYSIS^L|||201606291253|||||||201606291631||00007553^PHYSICIAN^ひなた^ ^^^||||||201607060811|||F||1^^^^^9\n
        OBX|1|ED|00008510^INTELLIGENT FLOW PROFILE^L||^^^^JVBERi0xLjQKJeLjz9MKCjEgMCBvYmoKPDwvVHlwZSAvQ2F0YWxvZwovUGFnZXMgMiAwIFI+PgplbmRvYmoKCjIgMCBvYmoKPDwvVHlwZSAvUGFnZXMKL0tpZHMgWzMgMCBSXQovQ291bnQgMT4+CmVuZG9iagoKMTIgMCBvYmoKPDwvTGVuZ3RoIDEzIDAgUgovRmlsdGVyIC9GbGF0ZURlY29kZT4+CnN0cmVhbQp4nM1c3ZPUOJJ/569QxMRFMLtgrG+buBfoHmZ6BxgW+u5iN3hxV7m7feMqF3Y...T0YK||||||C|||201606291631\n
        NTE|1|L|Reference Lab: GENOPTIX|L\n
        NTE|2|L|2110 ROUTHERFORD RD|L\n
        NTE|3|L|CARLSBAD, CA  92008|L";
    const HL7_V2_REPEATING_FIELD_MESSAGE: &str =
        "MSH|^~\&#|NIST EHR^2.16.840.1.113883.3.72.5.22^ISO|NIST EHR Facility^2.16.840.1.113883.3.72.5.23^ISO|NIST Test Lab APP^2.16.840.1.113883.3.72.5.20^ISO|NIST Lab Facility^2.16.840.1.113883.3.72.5.21^ISO|20130211184101-0500||OML^O21^OML_O21|NIST-LOI_9.0_1.1-GU_PRU|T|2.5.1|||AL|AL|||||LOI_Common_Component^LOI BaseProfile^2.16.840.1.113883.9.66^ISO~LOI_GU_Component^LOI GU Profile^2.16.840.1.113883.9.78^ISO~LAB_PRU_Component^LOI PRU Profile^2.16.840.1.113883.9.82^ISO\n
        PID|1||PATID14567^^^NIST MPI&2.16.840.1.113883.3.72.5.30.2&ISO^MR||Hernandez^Maria^^^^^L||19880906|F||2054-5^Black or   African American^HL70005|3248 E  FlorenceAve^^Huntington Park^CA^90255^^H||^^PH^^^323^5825421|||||||||H^Hispanic or Latino^HL70189\n
        ORC|NW|ORD231-1^NIST EHR^2.16.840.1.113883.3.72.5.24^ISO|||||||20130116090021-0800|||134569827^Feller^Hans^^^^^^NPI&2.16.840.1.113883.4.6&ISO^L^^^NPI
        OBR|1|ORD231-1^NIST EHR^2.16.840.1.113883.3.72.5.24^ISO||34555-3^Creatinine 24H renal clearance panel^LN^^^^^^CreatinineClearance|||201301151130-0800|201301160912-0800||||||||134569827^Feller^Hans^^^^^^NPI&2.16.840.1.113883.4.6&ISO^L^^^NPI\n
        DG1|1||I10^Essential (primary) hypertension^I10C^^^^^^Hypertension, NOS|||F|||||||||2\n
        DG1|2||O10.93^Unspecified pre-existing hypertension complicating the puerperium^I10C^^^^^^Pregnancy with chronic hypertension|||W|||||||||1\n
        OBX|1|CWE|67471-3^Pregnancy status^LN^1903^Pregnancy status^99USL^2.44^^Isthe patient pregnant?||Y^Yes^HL70136^1^Yes, confirmed less than 12 weeks^99USL^2.5.1^^early pregnancy (pre 12 weeks)||||||O|||20130115|||||||||||||||SCI\n
        OBX|2|NM|3167-4^Volume of   24   hour Urine^LN^1904^Urine Volume of 24 hour collection^99USL^2.44^^Urine Volume 24hour collection||1250|mL^milliliter^UCUM^ml^mililiter^L^1.7^^ml|||||O|||20130116|||||||||||||||SCI\n
        OBX|3|NM|3141-9^Body weight Measured^LN^BWm^Body weight Measured^99USL^2.44^^patient weight measured in kg||59.5|kg^kilogram^UCUM|||||O|||20130116|||||||||||||||SCI\n
        SPM|1|S-2312987-1&NIST EHR&2.16.840.1.113883.3.72.5.24&ISO||276833005^24 hour urine sample (specimen)^SCT^UR24H^24hr Urine^99USL^^^24 hour urine|||||||||||||201301151130-0800^201301160912-0800\n
        SPM|2|S-2312987-2&NIST EHR&2.16.840.1.113883.3.72.5.24&ISO||119297000^Blood Specimen^SCT|||||||||||||201301160912-0800ORC|NW|ORD231-2^NIST EHR^2.16.840.1.113883.3.72.5.24^ISO|||||||20130115102146-0800|||134569827^Feller^Hans^^^^^^NPI&2.16.840.1.113883.4.6&ISO^L^^^NPI\n
        OBR|2|ORD231-2^NIST EHR^2.16.840.1.113883.3.72.5.24^ISO||21482-5^Protein [Mass/volume] in 24 hour Urine^LN^^^^^^24 hour Urine Protein|||201301151130-0800|201301160912-0800||||||||134569827^Feller^Hans^^^^^^NPI&2.16.840.1.113883.4.6&ISO^L^^^NPI\n
        DG1|1||I10^Essential (primary) hypertension^I10C^^^^^^Hypertension, NOS|||F|||||||||2";
    const SPANISH_NAME: &str = "Andrés";
    const SANSKRIT_NAME: &str = "आरवा";
    const HIRAGANA_NAME: &str = "ひなた";
    const repeate_field1: &str = "ISO";
    const repeate_field2: &str = "LOI_GU_Component";
    const repeate_field3: &str = "LAB_PRU_Component";
    const DEFAULT_HL7_V2_FIELD_STRING: &str = "2000^2012^01";

    /*********************************Test Cases**************************************/
    #[test]
    fn test_hl7_v2_field_parsing() {
        let field_str = tests::DEFAULT_HL7_V2_FIELD_STRING;
        let encode_chars = V2ParserCharacters::new();
        let field = V2Field::from_str(&field_str, &encode_chars);
        println!("{:#?}", &field);
        assert_eq!(field.len(), 3, "Wrong number of components in field");
        println!("Value in component {} => {}!", 0, field.get(1).unwrap().as_str());
        assert_eq!(field.get(1).unwrap().as_str(), "2000", "Wrong value in component!");
        println!("Value in component {} => {}!", 1, field.get(2).unwrap().as_str());
        assert_eq!(field.get(2).unwrap().as_str(), "2012", "Wrong value in component!");
        println!("Value in component {} => {}!", 2, field.get(3).unwrap().as_str());
        assert_eq!(field.get(3).unwrap().as_str(), "01", "Wrong value in component!");
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
        let message = V2Message::from_str(tests::DEFAULT_HL7_V2_MESSAGE).unwrap();
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
        let message = V2Message::from_str(tests::HL7_V2_MESSAGE).unwrap();
        assert!(message.segment_exists("MSH"), "Missing MSH segment!");
        assert!(message.segment_exists("FHS"), "Missing FHS segment!");
        assert!(message.segment_exists("NK1"), "Missing NK1 segment!");
        assert!(message.segment_exists("PV1"), "Missing PV1 segment!");
        assert!(message.segment_exists("FTS"), "Missing FTS segment!");
        assert!(message.segment_exists("BHS"), "Missing BHS segment!");
    }

    #[test]
    fn test_load_hl7_v2_utf8_message() {
        /*
        Testing for the proper parsing of message when presented with Unicode portions.
         */
        let message = V2Message::from_str(tests::HL7_V2_PDF_MESSAGE).unwrap();
        let pid = message.get("PID", 1).unwrap();
        let orc = message.get("ORC", 1).unwrap();
        let obr = message.get("OBR", 1).unwrap();
        let name1 = pid.get(5).unwrap().get(1).unwrap().as_str();
        let name2 = orc.get(12).unwrap().get(3).unwrap().as_str();
        let name3 = obr.get(16).unwrap().get(3).unwrap().as_str();
        println!("{}", name1);
        println!("{}", name2);
        println!("{}", name3);
        assert_eq!(name1, SPANISH_NAME, "Wrong name/string found in PID(1)5.1!");
        assert_eq!(name2, SANSKRIT_NAME, "Wrong name/string found in ORC(1)12.3!");
        assert_eq!(name3, HIRAGANA_NAME, "Wrong name/string found in OBR(1)16.3!");
    }

    #[test]
    fn test_handle_hl7_v2_message_with_repeating_fields() {
        /*
        Testing for the proper parsing of message when presented with Unicode portions.
         */
        let message = V2Message::from_str(tests::HL7_V2_REPEATING_FIELD_MESSAGE).unwrap();
        let msh = message.get("MSH", 1).unwrap();
        let field1 = msh.get(5).unwrap().get(1).unwrap().as_str();
        let field2 = msh.get(12).unwrap().get(3).unwrap().as_str();
        let field3 = msh.get(16).unwrap().get(3).unwrap().as_str();
        assert_eq!(field1, repeate_field1, "Wrong field contents found in MSH(1)5.1!");
        assert_eq!(field2, repeate_field2, "Wrong field contents found in MSH(1)12.3!");
        assert_eq!(field3, repeate_field3, "Wrong field contents found in MSH(1)16.3!");
    }
}
