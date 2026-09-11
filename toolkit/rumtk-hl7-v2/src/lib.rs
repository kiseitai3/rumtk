/*
 * rumtk attempts to implement HL7 and medical protocols for interoperability in medicine.
 * This toolkit aims to be reliable, simple, performant, and standards compliant.
 * Copyright (C) 2024  Luis M. Santos, M.D. <lsantos@medicalmasses.com>
 * Copyright (C) 2025  MedicalMasses L.L.C. <contact@medicalmasses.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
//#![feature(inherent_associated_types)]
#![feature(rustc_private)]
#![feature(str_as_str)]
#![feature(allocator_api)]

extern crate rumtk_core;
pub mod hl7_v2_base_types;
pub mod hl7_v2_complex_types;
pub mod hl7_v2_constants;
pub mod hl7_v2_datasets;
pub mod hl7_v2_field_descriptors;
pub mod hl7_v2_interpreter;
pub mod hl7_v2_mllp;
pub mod hl7_v2_optionality_rules;
pub mod hl7_v2_parser;
pub mod hl7_v2_scripting;
pub mod hl7_v2_search;
pub mod hl7_v2_types;
pub mod hl7_v2_python_types;
/*****************************************Tests****************************************/
#[cfg(test)]
mod tests {
    use crate::hl7_v2_base_types::v2_base_types::{
        V2DateTime, V2ParserCharacters, V2SearchIndex, V2String,
    };
    use crate::hl7_v2_base_types::v2_primitives::{
        V2PrimitiveCasting, V2PrimitiveType, TRUNCATE_FT,
    };
    use crate::hl7_v2_complex_types::hl7_v2_complex_types::{cast_component, V2Type};
    use crate::hl7_v2_constants::{V2_SEGMENT_IDS, V2_SEGMENT_IDS_USIZE};
    use crate::hl7_v2_field_descriptors::v2_field_descriptor::{
        V2ComponentType, V2ComponentTypeDescriptor,
    };
    use crate::hl7_v2_mllp::mllp_v2::{
        mllp_decode, mllp_encode, MLLPClientMessages, CR, EB, MLLP_FILTER_POLICY, SB,
    };
    use crate::hl7_v2_optionality_rules::Optionality;
    use crate::hl7_v2_parser::v2_parser::{V2Field, V2Message};
    use crate::hl7_v2_search::REGEX_V2_SEARCH_DEFAULT;
    use crate::{
        rumtk_v2_find_component, rumtk_v2_generate_message, rumtk_v2_mllp_connect,
        rumtk_v2_mllp_get_client_ids, rumtk_v2_mllp_get_ip_port, rumtk_v2_mllp_iter_channels,
        rumtk_v2_mllp_listen, rumtk_v2_mllp_receive, rumtk_v2_mllp_send, rumtk_v2_parse_message,
    };
    use rumtk_core::base::{RUMResult, RUMVec};
    use rumtk_core::buffers::*;
    use rumtk_core::buffers::{buffer_find, buffer_find_instances, buffer_replace, buffer_replace_in_place, buffer_to_str, RUMBufferIteratorExt};
    use rumtk_core::cpu::{cpu_collect, CPUTokenIndexCollection, CPU_SEARCH_WINDOW_256_SIZE};
    use rumtk_core::search::rumtk_search::{string_search_named_captures, SearchGroups};
    use rumtk_core::serde::{from_json, to_json, RUMDeJson, RUMSerJson};
    use rumtk_core::strings::{basic_escape, rumtk_format, AsStr, RUMArrayConversions, RUMString, StringUtils};
    use rumtk_core::{rumtk_benchmark_snippet, rumtk_create_task, rumtk_deserialize, rumtk_exec_task, rumtk_resolve_task, rumtk_serialize, rumtk_sleep};
    use std::thread::spawn;
    use std::time::Instant;
    /**********************************Constants**************************************/
    use crate::hl7_v2_datasets::{hl7_v2_messages::*, hl7_v2_test_fragments::*};
    /*********************************Test Cases**************************************/
    #[test]
    fn test_hl7_v2_field_parsing() {
        let field_str = RUMBuffer::from(DEFAULT_HL7_V2_FIELD_STRING.as_bytes());
        let encode_chars = V2ParserCharacters::new();
        let field = V2Field::from(field_str.freeze(), &encode_chars);
        println!("{}", DEFAULT_HL7_V2_FIELD_STRING);
        println!("{:#?}", &field);
        assert_eq!(field.len(), 3, "Wrong number of components in field");
        println!(
            "Value in component {} => {}!",
            0,
            field.get(1).unwrap().as_str()
        );
        assert_eq!(
            field.get(1).unwrap().as_str(),
            "2000",
            "Wrong value in component!"
        );
        println!(
            "Value in component {} => {}!",
            1,
            field.get(2).unwrap().as_str()
        );
        assert_eq!(
            field.get(2).unwrap().as_str(),
            "2012",
            "Wrong value in component!"
        );
        println!(
            "Value in component {} => {}!",
            2,
            field.get(3).unwrap().as_str()
        );
        assert_eq!(
            field.get(3).unwrap().as_str(),
            "01",
            "Wrong value in component!"
        );
    }

    #[test]
    fn test_sanitize_hl7_v2_message() {
        let message = RUMBuffer::from(DEFAULT_HL7_V2_MESSAGE.clone().as_bytes());
        let mut raw = RUMBuffer::from(DEFAULT_HL7_V2_MESSAGE.clone().as_bytes());
        let sanitized_message = V2Message::sanitize(&mut raw);
        println!("{:?}", buffer_to_str(message.as_slice()).unwrap());
        println!("{:?}", buffer_to_str(sanitized_message.as_slice()).unwrap());
        
        assert!(
            message.contains(&('\n' as u8)),
            "Raw message has new line characters."
        );
        assert!(
            !sanitized_message.contains(&('\n' as u8)),
            "Sanitized message has new line characters."
        );
    }

    /*
    #[test]
    fn test_tokenize_hl7_v2_message() {
        let encode_chars = V2ParserCharacters::new();
        let message = RUMBuffer::from(DEFAULT_HL7_V2_MESSAGE.as_bytes());
        let sanitized_message = V2Message::sanitize(message);
        println!("Input => {:?}", &sanitized_message);
        println!("Parse chars => {:#?}", &encode_chars);

        let tokens = vec![];//V2Message::tokenize_segments(sanitized_message, &encode_chars);
        println!("Token count {}", tokens.len());
        assert_eq!(
            tokens.len(),
            5,
            "Tokenizer generated the wrong number of tokens! We expected 5 segment tokens."
        );
    }*/

    #[test]
    fn test_load_hl7_v2_encoding_characters() {
        let encode_chars = V2ParserCharacters::new();
        let mut message = RUMBuffer::from(DEFAULT_HL7_V2_MESSAGE.as_bytes());
        let sanitized_message = V2Message::sanitize(&mut message);
        let encode_chars = V2ParserCharacters::from(&sanitized_message).unwrap();
        println!("{:#?}", encode_chars);
        assert!(
            encode_chars.segment_terminator == '\r' as u8,
            "Wrong segment character!"
        );
        assert!(
            encode_chars.field_separator == '|' as u8,
            "Wrong field character!"
        );
        assert!(
            encode_chars.component_separator == '^' as u8,
            "Wrong component character!"
        );
        assert!(
            encode_chars.repetition_separator == '~' as u8,
            "Wrong repetition character!"
        );
        assert!(
            encode_chars.escape_character == '\\' as u8,
            "Wrong escape character!"
        );
        assert!(
            encode_chars.subcomponent_separator == '&' as u8,
            "Wrong subcomponent character!"
        );
        assert!(
            encode_chars.truncation_character == '#' as u8,
            "Wrong truncation character!"
        );
    }

    #[test]
    fn test_extract_hl7_v2_message_segments() {
        let mut message = RUMBuffer::from(DEFAULT_HL7_V2_MESSAGE.as_bytes());
        let sanitized_message = V2Message::sanitize(&mut message);
        let encode_chars = V2ParserCharacters::from(&sanitized_message).unwrap();
        let parsed_segments = V2Message::extract_segments(sanitized_message.freeze(), &encode_chars).unwrap();

        assert_eq!(
            parsed_segments.iter().filter(|&x| x.is_some()).collect::<Vec<_>>().len(),
            5,
            "Number of segments mismatching what was expected!"
        );
        assert!(
            parsed_segments.get(V2_SEGMENT_IDS_USIZE(b"MSH")).is_some(),
            "Missing MSH segment!"
        );
        assert!(
            parsed_segments.get(V2_SEGMENT_IDS_USIZE(b"PID")).is_some(),
            "Missing PID segment!"
        );
        assert!(
            parsed_segments.get(V2_SEGMENT_IDS_USIZE(b"PV1")).is_some(),
            "Missing PV1 segment!"
        );
        assert!(
            parsed_segments.get(V2_SEGMENT_IDS_USIZE(b"EVN")).is_some(),
            "Missing EVN segment!"
        );
        assert!(
            parsed_segments.get(V2_SEGMENT_IDS_USIZE(b"NK1")).is_some(),
            "Missing NK1 segment!"
        );
    }

    #[test]
    fn test_extract_hl7_v2_message_scrambled_segments() {
        let mut message = RUMBuffer::from(HL7_V2_SCRAMBLED.as_bytes());
        let sanitized_message = V2Message::sanitize(&mut message);
        let encode_chars = V2ParserCharacters::from(&sanitized_message).unwrap();
        println!("{}", buffer_to_str(&sanitized_message.as_slice()).unwrap());
        let parsed_segments = V2Message::extract_segments(sanitized_message.freeze(), &encode_chars).unwrap();

        assert_eq!(
            parsed_segments.iter().filter(|&x| x.is_some()).collect::<Vec<_>>().len(),
            4,
            "Number of segments mismatching what was expected!"
        );
        assert!(
            parsed_segments.get(V2_SEGMENT_IDS_USIZE(b"MSH")).is_some(),
            "Missing MSH segment!"
        );
        assert!(
            parsed_segments.get(V2_SEGMENT_IDS_USIZE(b"PID")).is_some(),
            "Missing PID segment!"
        );
        assert!(
            parsed_segments.get(V2_SEGMENT_IDS_USIZE(b"PD1")).is_some(),
            "Missing PV1 segment!"
        );
        assert!(
            parsed_segments.get(V2_SEGMENT_IDS_USIZE(b"RXA")).is_some(),
            "Missing EVN segment!"
        );
    }

    #[test]
    fn test_load_hl7_v2_two_segments() {
        let message = V2Message::try_from(DEFAULT_HL7_V2_TWO_SEGMENTS).unwrap();
        println!("{}", rumtk_serialize!(&message).unwrap_or_default());
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"MSH")),
            "Missing MSH segment!"
        );
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"EVN")),
            "Missing EVN segment!"
        );
    }

    #[test]
    fn test_load_hl7_v2_two_segments_parsed_correctly() {
        let mut message = RUMBuffer::from(DEFAULT_HL7_V2_TWO_SEGMENTS.as_bytes());
        let sanitized_message = V2Message::sanitize(&mut message);
        let message = V2Message::try_from(sanitized_message.clone()).unwrap();
        let generated = rumtk_v2_generate_message!(message);

        assert_eq!(
            &generated,
            EXPECTED_PARSED_TWO_SEGMENTS,
            "Failed to parse properly or something broke!"
        );
    }

    #[test]
    fn test_load_hl7_v2_message() {
        let message = V2Message::try_from(DEFAULT_HL7_V2_MESSAGE).unwrap();
        println!("{}", rumtk_serialize!(&message).unwrap_or_default());
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"MSH")),
            "Missing MSH segment!"
        );
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"PID")),
            "Missing PID segment!"
        );
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"PV1")),
            "Missing PV1 segment!"
        );
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"EVN")),
            "Missing EVN segment!"
        );
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"NK1")),
            "Missing NK1 segment!"
        );
    }

    ///
    /// Per examples in https://confluence.hl7.org/display/OO/v2+Sample+Messages you can have
    ///  messages that have other header segments before the standard MSH header.
    ///  As a result, I have made the logic a bit more permissive of the position of the msh segment.
    ///  I also made sure segments were trimmed to avoid issues with white space padding
    ///
    #[test]
    fn test_load_hl7_v2_message_wir_iis() {
        let message = V2Message::try_from(HL7_V2_MESSAGE).unwrap();
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"MSH")),
            "Missing MSH segment!"
        );
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"FHS")),
            "Missing FHS segment!"
        );
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"NK1")),
            "Missing NK1 segment!"
        );
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"PV1")),
            "Missing PV1 segment!"
        );
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"FTS")),
            "Missing FTS segment!"
        );
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"BHS")),
            "Missing BHS segment!"
        );
    }
    #[test]
    fn test_load_hl7_v2_message_scrambled_vec() {
        let input = Vec::from(HL7_V2_SCRAMBLED);
        let message = V2Message::try_from(input).unwrap();
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"MSH")),
            "Missing MSH segment!"
        );
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"PID")),
            "Missing PID segment!"
        );
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"PD1")),
            "Missing PV1 segment!"
        );
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"RXA")),
            "Missing RXA segment!"
        );
    }
    #[test]
    fn test_load_hl7_v2_message_scrambled() {
        let message = V2Message::try_from(HL7_V2_SCRAMBLED).unwrap();
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"MSH")),
            "Missing MSH segment!"
        );
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"PID")),
            "Missing PID segment!"
        );
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"PD1")),
            "Missing PV1 segment!"
        );
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"RXA")),
            "Missing RXA segment!"
        );
    }

    ///
    /// Testing for the proper parsing of message when presented with Unicode portions.
    ///
    #[test]
    fn test_load_hl7_v2_utf8_message() {
        let message = V2Message::try_from(HL7_V2_PDF_MESSAGE).unwrap();
        let pid = message.get(V2_SEGMENT_IDS(b"PID"), 1).unwrap();
        let orc = message.get(V2_SEGMENT_IDS(b"ORC"), 1).unwrap();
        let obr = message.get(V2_SEGMENT_IDS(b"OBR"), 1).unwrap();
        let binding = pid
            .get(5)
            .unwrap()
            .get(0)
            .unwrap()
            .get(1)
            .unwrap();
        let name1 = binding
            .as_str();
        let binding = orc
            .get(12)
            .unwrap()
            .get(0)
            .unwrap()
            .get(3)
            .unwrap();
        let name2 = binding
            .as_str();
        let binding = obr
            .get(16)
            .unwrap()
            .get(0)
            .unwrap()
            .get(3)
            .unwrap();
        let name3 = binding
            .as_str();
        println!("{}", name1);
        println!("{}", name2);
        println!("{}", name3);
        assert_eq!(name1, SPANISH_NAME, "Wrong name/string found in PID(1)5.1!");
        assert_eq!(
            name2, SANSKRIT_NAME,
            "Wrong name/string found in ORC(1)12.3!"
        );
        assert_eq!(
            name3, HIRAGANA_NAME,
            "Wrong name/string found in OBR(1)16.3!"
        );
    }

    ///
    /// Testing for the proper parsing of message when presented with repeating fields.
    ///
    #[test]
    fn test_handle_hl7_v2_message_with_repeating_fields() {
        let message = V2Message::try_from(HL7_V2_REPEATING_FIELD_MESSAGE).unwrap();
        let msh = message.get(V2_SEGMENT_IDS(b"MSH"), 1).unwrap();
        let binding = msh
            .get(-1)
            .unwrap()
            .get(0)
            .unwrap()
            .get(4)
            .unwrap();
        let field1 = binding
            .as_str();
        let binding = msh
            .get(-1)
            .unwrap()
            .get(1)
            .unwrap()
            .get(1)
            .unwrap();
        let field2 = binding
            .as_str();
        let binding = msh
            .get(-1)
            .unwrap()
            .get(2)
            .unwrap()
            .get(1)
            .unwrap();
        let field3 = binding
            .as_str();
        assert_eq!(
            msh.get(-1).unwrap().len(),
            3,
            "Wrong number of subfields in group in MSH(1)-1!"
        );
        assert_eq!(
            field1, repeate_field1,
            "Wrong field contents found in MSH(1)-1(0).4!"
        );
        assert_eq!(
            field2, repeate_field2,
            "Wrong field contents found in MSH(1)-1(1).1!"
        );
        assert_eq!(
            field3, repeate_field3,
            "Wrong field contents found in MSH(1)-1(2).1!"
        );
    }

    #[test]
    fn test_generating_v2_message() {
        let message = rumtk_v2_parse_message!(&DEFAULT_HL7_V2_MESSAGE).unwrap();
        let generated_message_string = rumtk_v2_generate_message!(&message);
        let generated_message = rumtk_v2_parse_message!(&generated_message_string).unwrap();
        assert_eq!(
            &message, &generated_message,
            "Messages are not equal! Expected: {:?} Got: {:?}",
            &message, &generated_message
        );
    }

    #[test]
    fn test_generating_v2_message_wir() {
        let message = rumtk_v2_parse_message!(&HL7_V2_MESSAGE).unwrap();
        let generated_message_string = rumtk_v2_generate_message!(&message);
        let generated_message = rumtk_v2_parse_message!(&generated_message_string).unwrap();
        assert_eq!(
            &message, &generated_message,
            "Messages are not equal! Expected: {:?} Got: {:?}",
            &message, &generated_message
        );
    }

    #[test]
    fn test_generating_v2_message_pdf() {
        let message = rumtk_v2_parse_message!(&HL7_V2_PDF_MESSAGE).unwrap();
        let generated_message_string = rumtk_v2_generate_message!(&message);
        let generated_message = rumtk_v2_parse_message!(&generated_message_string).unwrap();
        assert_eq!(
            &message, &generated_message,
            "Messages are not equal! Expected: {:?} Got: {:?}",
            &message, &generated_message
        );
    }

    #[test]
    fn test_generating_v2_message_repeated_fields() {
        let message = rumtk_v2_parse_message!(&HL7_V2_REPEATING_FIELD_MESSAGE).unwrap();
        let generated_message_string = rumtk_v2_generate_message!(&message);
        let generated_message = rumtk_v2_parse_message!(&generated_message_string).unwrap();
        assert_eq!(
            &message, &generated_message,
            "Messages are not equal! Expected: {:?} Got: {:?}",
            &message, &generated_message
        );
    }

    #[test]
    fn test_handle_hl7_v2_search_pattern_parsing_full() {
        let pattern = "MSH(1)-1[5].4";
        let groups = string_search_named_captures(pattern, REGEX_V2_SEARCH_DEFAULT, "1").unwrap();
        let expected = SearchGroups::from([
            (RUMString::from("segment_group"), RUMString::from("1")),
            (RUMString::from("sub_field"), RUMString::from("5")),
            (RUMString::from("segment"), RUMString::from("MSH")),
            (RUMString::from("field"), RUMString::from("-1")),
            (RUMString::from("component"), RUMString::from("4")),
        ]);
        println!(
            "Input: {:?} Expected: {:?} Got: {:?}",
            pattern, expected, groups
        );
        assert_eq!(
            groups, expected,
            "Misparsed search expression MSH(1)-1[5].4!"
        );
    }

    #[test]
    fn test_handle_hl7_v2_search_pattern_parsing_simple() {
        let pattern = "MSH1.4";
        let groups = string_search_named_captures(pattern, REGEX_V2_SEARCH_DEFAULT, "1").unwrap();
        let expected = SearchGroups::from([
            (RUMString::from("segment_group"), RUMString::from("1")),
            (RUMString::from("sub_field"), RUMString::from("1")),
            (RUMString::from("segment"), RUMString::from("MSH")),
            (RUMString::from("field"), RUMString::from("1")),
            (RUMString::from("component"), RUMString::from("4")),
        ]);
        println!(
            "Input: {:?} Expected: {:?} Got: {:?}",
            pattern, expected, groups
        );
        assert_eq!(groups, expected, "Misparsed search expression MSH1.4!");
    }

    #[test]
    fn test_v2_search_index() {
        let expr = "MSH(1)-1[5].4";
        let v2_search_index = V2SearchIndex::from(expr);
        let expected = V2SearchIndex::new("MSH", 1, -1, 5, 4);
        println!(
            "Input: {:?} Expected: {:?} Got: {:?}",
            expr, expected, v2_search_index
        );
        assert_eq!(
            v2_search_index, expected,
            "Failed to parse expression into correct SearchIndex object."
        );
    }

    #[test]
    fn test_load_hl7_v2_message_macro() {
        let message = rumtk_v2_parse_message!(DEFAULT_HL7_V2_MESSAGE).unwrap();
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"MSH")),
            "Missing MSH segment!"
        );
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"PID")),
            "Missing PID segment!"
        );
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"PV1")),
            "Missing PV1 segment!"
        );
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"EVN")),
            "Missing EVN segment!"
        );
        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"NK1")),
            "Missing NK1 segment!"
        );
    }

    #[test]
    fn test_load_msh() {
        let message = rumtk_v2_parse_message!(EXPECTED_MSH_SEGMENT).unwrap();
        let msg_string = rumtk_v2_generate_message!(&message);

        assert!(
            message.segment_exists(V2_SEGMENT_IDS(b"MSH")),
            "Missing MSH segment!"
        );
        assert_eq!(
            EXPECTED_MSH_SEGMENT, msg_string,
            "MSH misparsed!"
        );
    }

    #[test]
    fn test_load_large_message_check_all_segments_present() {
        let message = rumtk_v2_parse_message!(V2_TEST_LARGE_MESSAGE).unwrap();
        let all_segments = vec![b"MSH",b"PID",b"ORC",b"OBR",b"DG1",b"OBX",b"SPM"];

        for segment_k in all_segments {
            assert!(
                message.segment_exists(V2_SEGMENT_IDS(segment_k)),
                "Missing {} segment!", buffer_to_str(segment_k).unwrap()
            );
        }
    }

    #[test]
    fn test_load_large_message_check_correct_count_of_duplicate_dg1_segment() {
        let message = rumtk_v2_parse_message!(V2_TEST_LARGE_MESSAGE).unwrap();
        let all_segments = vec![(b"MSH",1),(b"PID",1),(b"ORC",1),(b"OBR",2),(b"DG1",3),(b"OBX",2048),(b"SPM",2)];

        for (segment_k, expected_count) in all_segments {
            let count = message.segment_group_count(V2_SEGMENT_IDS(segment_k));
            assert!(
                count == expected_count,
                "Segment {} has wrong count! Expected: {} Got: {}",
                buffer_to_str(segment_k).unwrap(), expected_count, count
            );
        }
    }

    #[test]
    fn test_load_hl7_v2_message_macro_failure() {
        let input = "Hello World!";
        let err_msg = rumtk_format!(
            "Parsing did not fail as expected. Input {} => parsed?",
            input
        );
        match rumtk_v2_parse_message!(input) {
            Ok(v) => panic!("{}", err_msg.as_str()),
            Err(e) => {
                println!("{}", rumtk_format!("Got error => {}", e).as_str());
                println!("Passed failed case!");
            }
        };
    }

    #[test]
    fn test_find_hl7_v2_message_component_macro() {
        let pattern = "PID(1)5.4";
        let message = rumtk_v2_parse_message!(DEFAULT_HL7_V2_MESSAGE).unwrap();
        let component = rumtk_v2_find_component!(message, pattern).unwrap();
        let expected = "III";
        assert_eq!(
            component.as_str(),
            expected,
            "Wrong component found! Looked for {} expecting {}, but got {}",
            pattern,
            expected,
            component.as_str()
        );
    }

    #[test]
    fn test_find_hl7_v2_message_component_simple_macro() {
        let pattern = "PID5.4";
        let message = rumtk_v2_parse_message!(DEFAULT_HL7_V2_MESSAGE).unwrap();
        let component = rumtk_v2_find_component!(message, pattern).unwrap();
        let expected = "III";
        assert_eq!(
            component.as_str(),
            expected,
            "Wrong component found! Looked for {} expecting {}, but got {}",
            pattern,
            expected,
            component.as_str()
        );
    }

    #[test]
    fn test_find_hl7_v2_message_msh_field() {
        let pattern = "MSH1.1";
        let message = rumtk_v2_parse_message!(HL7_V2_MSH_ONLY).unwrap();
        let component = rumtk_v2_find_component!(message, pattern).unwrap();
        let expected = "^~\\&"; // We do not need to include the truncation character.
        assert_eq!(
            component.as_str(),
            expected,
            "Wrong component found! Looked for {} expecting {}, but got {}",
            pattern,
            expected,
            component.as_str()
        );
    }

    #[test]
    fn test_find_hl7_v2_message_component_macro_failure() {
        let pattern = "PID(1)15.4";
        let err_msg = rumtk_format!(
            "Search did not fail as expected. Input {} => found component?",
            pattern
        );
        let message = rumtk_v2_parse_message!(DEFAULT_HL7_V2_MESSAGE).unwrap();
        match rumtk_v2_find_component!(message, pattern) {
            Ok(v) => panic!("{}", err_msg.as_str()),
            Err(e) => {
                println!("{}", rumtk_format!("Got error => {}", e).as_str());
                println!("Passed failed case!");
            }
        }
    }

    #[test]
    fn test_cast_component_to_datetime_expected_functionality() {
        let inputs = [
            "2007",
            "200708",
            "20070818",
            "200708181123",
            "20070818112355",
            "20070818112355.55",
            "20070818112355.5555-5000",
            "20070818112355-5000",
        ];
        let expected_outputs = [
            "2007-01-01T00:00:00.0000",
            "2007-08-01T00:00:00.0000",
            "2007-08-18T00:00:00.0000",
            "2007-08-18T11:23:00.0000",
            "2007-08-18T11:23:55.0000",
            "2007-08-18T11:23:55.5500",
            "2007-08-18T11:23:55.5555-5000",
            "2007-08-18T11:23:55.0000-5000",
        ];
        for i in 0..inputs.len() {
            let input = inputs[i];
            let expected_utc = expected_outputs[i];
            print!(
                "Testing input #{} \"{}\". Expected output is \"{}\". Casting to datetime type.",
                i, input, expected_utc
            );
            let date = input.to_v2datetime().unwrap();
            let err_msg = rumtk_format!("The expected date time string does not match the date time string generated from the input [In: {}, Got: {}]", input, date.as_utc_string());
            assert_eq!(expected_utc, date.as_utc_string().as_str(), "{}", &err_msg);
            println!(" ... Got: {} ✅ ", date.as_utc_string());
        }
    }

    #[test]
    fn test_cast_component_to_datetime_validation() {
        let inputs = ["200"];
        for input in inputs {
            match input.to_v2datetime() {
                Ok(date) => {
                    panic!(
                        "Validation failed [In: {} Got: {} Expected: None] ... ✕",
                        input,
                        date.as_utc_string()
                    );
                }
                Err(e) => println!(
                    "Validation correctly identified malformed input with message => [{}] ✅",
                    e.as_str()
                ),
            }
        }
    }

    #[test]
    fn test_cast_component_to_datetime_base_example() {
        let location = "EVN2"; //EVN|A01|200708181123||\n\r; EVN2 => segment = EVN, field = 2
        let expected_component = "200708181123";
        let message = rumtk_v2_parse_message!(DEFAULT_HL7_V2_MESSAGE).unwrap();
        let component = rumtk_v2_find_component!(message, location).unwrap();
        assert_eq!(expected_component, component.as_str(), "We are not using the correct component for this test. Check that the original test message has not changed and update the location string appropriately!");
        let date = component.to_v2datetime().unwrap();
        let expected_utc = "2007-08-18T11:23:00.0000";
        let err_msg = rumtk_format!("The expected date time string does not match the date time string generated from the input [{}]", component.as_str());
        assert_eq!(expected_utc, date.as_utc_string().as_str(), "{}", &err_msg)
    }

    #[test]
    fn test_datetime_default() {
        let input = V2DateTime::default().as_utc_string();
        let expected_val = V2String::from("1970-01-01T00:00:00.00000");
        let err_msg = rumtk_format!("The expected formatted string does not match the formatted string generated from the input [In: {}, Got: {}]", input, input);
        assert_eq!(expected_val, input, "{}", &err_msg);
    }

    #[test]
    fn test_cast_component_to_date_expected_functionality() {
        let inputs = ["2007", "200708", "20070818"];
        let expected_outputs = [
            "2007-01-01T00:00:00.0000",
            "2007-08-01T00:00:00.0000",
            "2007-08-18T00:00:00.0000",
        ];
        for i in 0..inputs.len() {
            let input = inputs[i];
            let expected_utc = expected_outputs[i];
            print!(
                "Testing input #{} \"{}\". Expected output is \"{}\". Casting to datetime type.",
                i, input, expected_utc
            );
            let date = input.to_v2date().unwrap();
            let err_msg = rumtk_format!("The expected date time string does not match the date time string generated from the input [In: {}, Got: {}]", input, date.as_utc_string());
            assert_eq!(expected_utc, date.as_utc_string().as_str(), "{}", &err_msg);
            println!(" ... Got: {} ✅ ", date.as_utc_string());
        }
    }

    #[test]
    fn test_cast_component_to_date_validation() {
        let inputs = ["200"];
        for input in inputs {
            match input.to_v2date() {
                Ok(date) => {
                    panic!(
                        "Validation failed [In: {} Got: {} Expected: None] ... ✕",
                        input,
                        date.as_utc_string()
                    );
                }
                Err(e) => println!(
                    "Validation correctly identified malformed input with message => [{}] ✅",
                    e.as_str()
                ),
            }
        }
    }

    #[test]
    fn test_cast_component_to_date_base_example() {
        let location = "PD113"; //EVN|A01|200708181123||\n\r; PD113 => segment = PD1, field = 13
        let expected_component = "20150625";
        let message = rumtk_v2_parse_message!(VXU_HL7_V2_MESSAGE).unwrap();
        let component = rumtk_v2_find_component!(message, location).unwrap();
        assert_eq!(expected_component, component.as_str(), "We are not using the correct component for this test. Check that the original test message has not changed and update the location string appropriately!");
        let date = component.to_v2date().unwrap();
        let expected_utc = "2015-06-25T00:00:00.0000";
        let err_msg = rumtk_format!(
            "The expected date string does not match the date string generated from the input [{}]",
            component.as_str()
        );
        assert_eq!(expected_utc, date.as_utc_string().as_str(), "{}", &err_msg)
    }

    #[test]
    fn test_cast_component_to_time_expected_functionality() {
        let inputs = ["1123", "112355", "112355.5555", "112355.5555-5000"];
        let expected_outputs = [
            "1970-01-01T11:23:00.0000",
            "1970-01-01T11:23:55.0000",
            "1970-01-01T11:23:55.5555",
            "1970-01-01T11:23:55.5555-5000",
        ];
        for i in 0..inputs.len() {
            let input = inputs[i];
            let expected_utc = expected_outputs[i];
            print!(
                "Testing input #{} \"{}\". Expected output is \"{}\". Casting to datetime type.",
                i, input, expected_utc
            );
            let date = input.to_v2time().unwrap();
            let err_msg = rumtk_format!("The expected date time string does not match the date time string generated from the input [In: {}, Got: {}]", input, date.as_utc_string());
            assert_eq!(expected_utc, date.as_utc_string().as_str(), "{}", &err_msg);
            println!(" ... Got: {} ✅ ", date.as_utc_string());
        }
    }

    #[test]
    fn test_cast_component_to_time_validation() {
        let inputs = ["2"];
        for input in inputs {
            match input.to_v2time() {
                Ok(date) => {
                    panic!(
                        "Validation failed [In: {} Got: {} Expected: None] ... ✕",
                        input,
                        date.as_utc_string()
                    );
                }
                Err(e) => println!(
                    "Validation correctly identified malformed input with message => [{}] ✅",
                    e.as_str()
                ),
            }
        }
    }

    #[test]
    fn test_cast_component_to_number_expected_functionality() {
        let inputs = [
            "5e3",
            "5E3",
            "112355.5555",
            "5F",
            "5.5F",
            "5f",
            "5.5e2",
            "-5f",
            "-05e1",
        ];
        let expected_outputs = [
            5000.0,
            5000.0,
            112355.5555,
            5.0,
            5.5,
            5.0,
            550.0,
            -5.0,
            -50.0,
        ];
        for i in 0..inputs.len() {
            let input = inputs[i];
            let expected_val = expected_outputs[i];
            print!(
                "Testing input #{} \"{}\". Expected output is \"{}\". Casting to NM type.",
                i, input, expected_val
            );
            let val = input.to_v2number().unwrap();
            let err_msg = rumtk_format!("The expected date time string does not match the date time string generated from the input [In: {}, Got: {}]", input, val);
            assert_eq!(expected_val, val, "{}", &err_msg);
            println!(" ... Got: {} ✅ ", val);
        }
    }

    #[test]
    fn test_cast_component_to_number_validation() {
        let inputs = [".2"];
        for input in inputs {
            match input.to_v2number() {
                Ok(val) => {
                    panic!(
                        "Validation failed [In: {} Got: {} Expected: None] ... ✕",
                        input, val
                    );
                }
                Err(e) => println!(
                    "Validation correctly identified malformed input with message => [{}] ✅",
                    e.as_str()
                ),
            }
        }
    }

    #[test]
    fn test_cast_component_to_st_expected_functionality() {
        let inputs = [" Hello World!"];
        let expected_outputs = ["Hello World!"];
        for i in 0..inputs.len() {
            let input = inputs[i];
            let expected_val = expected_outputs[i];
            print!(
                "Testing input #{} \"{}\". Expected output is \"{}\". Casting to ST type.",
                i, input, expected_val
            );
            let val = input.to_v2stringdata().unwrap();
            let err_msg = rumtk_format!("The expected date time string does not match the date time string generated from the input [In: {}, Got: {}]", input, val);
            assert_eq!(expected_val, val, "{}", &err_msg);
            println!(" ... Got: {} ✅ ", val);
        }
    }

    #[test]
    fn test_cast_component_to_st_validation() {
        let input = "2".duplicate(1001);
        println!("{}", input);
        match input.to_v2stringdata() {
            Ok(val) => {
                panic!(
                    "Validation failed [In: {} Got: {} Expected: None] ... ✕",
                    input, val
                );
            }
            Err(e) => println!(
                "Validation correctly identified malformed input with message => [{}] ✅",
                e.as_str()
            ),
        }
    }

    #[test]
    fn test_cast_component_to_ft_expected_functionality() {
        let inputs = ["H", &"e".duplicate(120000)];
        let expected_outputs = ["H", &"e".duplicate(TRUNCATE_FT as usize)];
        for i in 0..inputs.len() {
            let input = inputs[i];
            let expected_val = expected_outputs[i];
            print!(
                "Testing input #{} \"{}\". Expected output is \"{}\". Casting to FT type.",
                i, input, expected_val
            );
            let val = input.to_v2formattedtext('~').unwrap();
            println!("{}", val.len());
            let err_msg = rumtk_format!("The expected formatted string does not match the formatted string generated from the input [In: {}, Got: {}]", input, val);
            assert_eq!(expected_val, val, "{}", &err_msg);
            println!(" ... Got: {} ✅ ", val);
        }
    }

    #[test]
    fn test_validated_cast_component_to_type() {
        let mut message = RUMBuffer::from(DEFAULT_HL7_V2_MESSAGE.as_bytes());
        let sanitized_message = V2Message::sanitize(&mut message);
        let encode_chars = V2ParserCharacters::from(&sanitized_message).unwrap();
        let v2_component = V2ComponentTypeDescriptor::new(
            "date",
            "Date",
            V2ComponentType::Primitive(V2PrimitiveType::Date),
            4,
            1,
            1,
            Optionality::O,
            true,
        );
        let input = "2007";
        let val = cast_component(vec![&input], &v2_component, &encode_chars);
        let expected = "2007-01-01T00:00:00.0000";
        let err_msg = rumtk_format!("The expected formatted string does not match the formatted string generated from the input [In: {}, Got: {}]", input, expected);

        match val {
            V2Type::V2Date(result) => {
                assert_eq!(expected, result.unwrap().as_utc_string(), "{}", &err_msg)
            }
            _ => panic!("Wrong type received!"),
        }
    }

    // TODO: Add tests for sequenceid and telephonestring
    // TODO: Add fuzzing test for to_datetime().

    #[test]
    fn test_mllp_encode() {
        let expected_message = RUMString::from("I ❤ my wife!");
        let encoded = mllp_encode(&expected_message);
        let payload = &encoded[1..encoded.len() - 2];

        assert_eq!(encoded[0], SB, "Incorrect start byte in MLLP message!");

        assert_eq!(
            encoded[encoded.len() - 2],
            EB,
            "Incorrect end byte in MLLP message!"
        );

        assert_eq!(
            encoded[encoded.len() - 1],
            CR,
            "Missing mandatory carriage return in MLLP message!"
        );

        assert_eq!(
            expected_message,
            payload.to_string().unwrap(),
            "{}",
            rumtk_format!(
                "Malformed payload! Expected: {} Found: {}",
                expected_message,
                payload.to_string().unwrap()
            )
        );
    }

    #[test]
    fn test_mllp_decode() {
        let expected_message = RUMString::from("I ❤ my wife!");
        let message_size = expected_message.len();
        let encoded = mllp_encode(&expected_message);
        let encoded_size = encoded.len();

        assert_eq!(
            encoded_size,
            message_size + 3,
            "Incorrect encoded message size!"
        );

        let decoded = mllp_decode(&encoded).unwrap();
        let decoded_size = decoded.len();

        assert_eq!(
            decoded_size, message_size,
            "Incorrect decoded message size! Expected: {} Got: {}",
            expected_message, decoded
        );

        assert_eq!(
            expected_message,
            decoded,
            "{}",
            rumtk_format!(
                "Malformed decoded message! Expected: {} Found: {}",
                expected_message,
                decoded
            )
        );
    }

    #[test]
    fn test_mllp_listen() {
        let mllp_layer = match rumtk_v2_mllp_listen!(0, MLLP_FILTER_POLICY::NONE, true) {
            Ok(mllp_layer) => mllp_layer,
            Err(e) => panic!("{}", e),
        };
        let (ip, port) = rumtk_v2_mllp_get_ip_port!(mllp_layer).unwrap_or_default();
        let client_id = rumtk_exec_task!(async || -> RUMResult<RUMString> {
            Ok(mllp_layer.lock().await.get_address_info().await.unwrap())
        })
        .unwrap();
        assert_eq!(
            client_id,
            rumtk_format!("127.0.0.1:{}", &port),
            "Failed to bind local port!"
        )
    }

    #[test]
    fn test_mllp_get_ids() {
        let mllp = rumtk_v2_mllp_listen!(MLLP_FILTER_POLICY::NONE, true).unwrap();
        let (ip, port) = rumtk_v2_mllp_get_ip_port!(mllp).unwrap();

        let safe_client = rumtk_v2_mllp_connect!(port, MLLP_FILTER_POLICY::NONE).unwrap();


        rumtk_sleep!(1);
        let mut results = rumtk_v2_mllp_get_client_ids!(mllp).unwrap();

        while results.is_empty() {
            rumtk_sleep!(1);
            results = rumtk_v2_mllp_get_client_ids!(mllp).unwrap();
        }

        let client_id = results.get(0).unwrap();
        let (client_ip, client_port) = rumtk_v2_mllp_get_ip_port!(safe_client).unwrap();
        let expected = rumtk_format!("{}:{}", client_ip, client_port);
        assert_eq!(
            &expected, client_id,
            "Expected to see client with ID: {}",
            expected
        );
    }

    #[test]
    fn test_mllp_get_ip() {
        let mllp_layer = match rumtk_v2_mllp_listen!(0, MLLP_FILTER_POLICY::NONE, true) {
            Ok(mllp_layer) => mllp_layer,
            Err(e) => panic!("{}", e),
        };
        let (ip, port) = rumtk_v2_mllp_get_ip_port!(&mllp_layer).unwrap();
    }

    #[test]
    fn test_mllp_echo() {
        static PORT: u16 = 55550;
        static EXPECTED_MESSAGE: &str = "Hello World";

        let safe_listener = rumtk_v2_mllp_listen!(PORT, MLLP_FILTER_POLICY::NONE, true).unwrap();

        let send_h = spawn(|| -> RUMResult<()> {
            let message = RUMString::from("Hello World");
            let safe_client = rumtk_v2_mllp_connect!(PORT, MLLP_FILTER_POLICY::NONE)?;
            let (ip, port) = rumtk_v2_mllp_get_ip_port!(&safe_client)?;
            let endpoint = rumtk_format!("{}:{}", ip, port);
            rumtk_v2_mllp_send!(&safe_client, &endpoint, &message)
        });

        let mut client_ids = rumtk_v2_mllp_get_client_ids!(safe_listener).unwrap();
        while client_ids.is_empty() {
            rumtk_sleep!(1);
            client_ids = rumtk_v2_mllp_get_client_ids!(safe_listener).unwrap();
        }
        let client_id = client_ids.get(0).unwrap().clone();

        println!("{}", &client_id);
        let results = rumtk_v2_mllp_receive!(&safe_listener, &client_id).unwrap();
        let result = results.get(0).unwrap();
        println!("Send thread completed!");

        assert_eq!(
            result, EXPECTED_MESSAGE,
            "Message received does not match the expected message. Got {}",
            &result
        );
    }

    #[test]
    fn test_mllp_connect() {
        let mllp_layer = match rumtk_v2_mllp_listen!(0, MLLP_FILTER_POLICY::NONE, true) {
            Ok(mllp_layer) => mllp_layer,
            Err(e) => panic!("{}", e),
        };
        let (ip, port) = rumtk_v2_mllp_get_ip_port!(mllp_layer).unwrap();
        let client = match rumtk_v2_mllp_connect!(port, MLLP_FILTER_POLICY::NONE) {
            Ok(client) => client,
            Err(e) => panic!("{}", e),
        };
        rumtk_sleep!(1);
        let mut connected_clients = rumtk_v2_mllp_get_client_ids!(&mllp_layer).unwrap();
        for i in 0..10 {
            if connected_clients.is_empty() {
                rumtk_sleep!(1);
                connected_clients = rumtk_v2_mllp_get_client_ids!(&mllp_layer).unwrap();
            }
        }
        let connected_address = connected_clients.get(0).unwrap();
        let client_ids = rumtk_v2_mllp_get_client_ids!(&client).unwrap();
        let client_id = client_ids.get(0).unwrap();
        assert_eq!(connected_address, client_id, "Failed to bind local port!")
    }

    #[test]
    fn test_mllp_channel() {
        let empty_string = |s: RUMString| Ok::<RUMString, RUMString>(RUMString::from(""));
        let safe_listener = match rumtk_v2_mllp_listen!(0, MLLP_FILTER_POLICY::NONE, true) {
            Ok(mllp_layer) => mllp_layer,
            Err(e) => panic!("{}", e),
        };
        let (ip, port) = rumtk_v2_mllp_get_ip_port!(&safe_listener).unwrap();
        let safe_client = match rumtk_v2_mllp_connect!(port, MLLP_FILTER_POLICY::NONE) {
            Ok(client) => client,
            Err(e) => panic!("{}", e),
        };
        rumtk_sleep!(1);
        let client_ids = rumtk_v2_mllp_get_client_ids!(&safe_listener).unwrap();
        let client_id = client_ids.get(0).unwrap();
        let mut server_channels = rumtk_v2_mllp_iter_channels!(safe_client).unwrap();
        let mut server_channel = server_channels.get_mut(0).unwrap().clone();
        let channel_address = server_channel.lock().unwrap().get_address_info().unwrap();
        assert_eq!(
            client_id,
            &channel_address,
            "{}",
            rumtk_format!(
                "Issue stablishing MLLP communication channel! Expected: {} Received: {}",
                &client_id,
                &channel_address
            )
        )
    }

    #[test]
    fn test_mllp_channel_async_communication() {
        let mut safe_listener = match rumtk_v2_mllp_listen!(0, MLLP_FILTER_POLICY::NONE, true) {
            Ok(mllp_layer) => mllp_layer,
            Err(e) => panic!("{}", e),
        };
        let (ip, port) = rumtk_v2_mllp_get_ip_port!(safe_listener).unwrap();
        let safe_client = match rumtk_v2_mllp_connect!(port, MLLP_FILTER_POLICY::NONE) {
            Ok(client) => client,
            Err(e) => panic!("{}", e),
        };
        rumtk_sleep!(1);
        let client_ids = rumtk_v2_mllp_get_client_ids!(safe_listener).unwrap();
        let client_id = client_ids.get(0).unwrap().clone();
        let mut server_channels = rumtk_v2_mllp_iter_channels!(safe_client).unwrap();
        let mut server_channel = server_channels.get_mut(0).unwrap().clone();
        let expected_message = RUMString::from("I ❤ my wife!");
        let message_copy = expected_message.clone();
        let send_thread = spawn(move || -> RUMResult<()> {
            Ok(server_channel
                .lock()
                .unwrap()
                .send_message(&message_copy)
                .unwrap())
        });
        //rumtk_sleep!(1);
        let received_messages = rumtk_exec_task!(async || -> RUMResult<MLLPClientMessages> {
            let mut received_message = safe_listener
                .lock()
                .await
                .receive_client_messages(&client_id)
                .await?;
            while received_message.len() == 0 {
                received_message = safe_listener
                    .lock()
                    .await
                    .receive_client_messages(&client_id)
                    .await?;
            }
            Ok(received_message)
        })
        .unwrap();
        let received_message = received_messages.get(0).unwrap();

        assert_eq!(
            &expected_message,
            received_message,
            "{}",
            rumtk_format!(
                "Issue sending message through channel! Expected: {} Received: {}",
                &expected_message,
                &received_message
            )
        )
    }

    #[test]
    fn test_mllp_hl7_echo() {
        let empty_string = |s: RUMString| Ok::<RUMString, RUMString>(RUMString::from(""));
        let mut safe_listener = match rumtk_v2_mllp_listen!(0, MLLP_FILTER_POLICY::NONE, true) {
            Ok(mllp_listener) => mllp_listener,
            Err(e) => panic!("{}", e),
        };
        let (ip, port) = rumtk_v2_mllp_get_ip_port!(safe_listener).unwrap();
        let safe_client = match rumtk_v2_mllp_connect!(port, MLLP_FILTER_POLICY::NONE) {
            Ok(client) => client,
            Err(e) => panic!("{}", e),
        };
        rumtk_sleep!(1);
        let client_ids = rumtk_v2_mllp_get_client_ids!(safe_listener).unwrap();
        let client_id = client_ids.get(0).unwrap().clone();
        let client_id_copy = client_id.clone();
        let mut server_channels = rumtk_v2_mllp_iter_channels!(safe_client.clone()).unwrap();
        let mut server_channel = server_channels.get_mut(0).unwrap().clone();
        let server_channel_copy = server_channel.clone();
        let send_thread = spawn(move || -> RUMResult<()> {
            Ok(server_channel
                .lock()
                .unwrap()
                .send_message(HL7_V2_PDF_MESSAGE)
                .unwrap())
        });
        let safe_listener_copy = safe_listener.clone();
        let received_messages = rumtk_exec_task!(async || -> RUMResult<MLLPClientMessages> {
            let mut received_message = safe_listener_copy
                .lock()
                .await
                .receive_client_messages(&client_id)
                .await?;
            while received_message.len() == 0 {
                received_message = safe_listener_copy
                    .lock()
                    .await
                    .receive_client_messages(&client_id)
                    .await?;
            }
            Ok(received_message)
        })
        .unwrap();
        let received_message = received_messages.get(0).unwrap();

        assert_eq!(
            &HL7_V2_PDF_MESSAGE,
            &received_message,
            "{}",
            rumtk_format!(
                "Issue sending message through channel! Expected: {} Received: {}",
                &HL7_V2_PDF_MESSAGE,
                &received_message
            )
        );
        let safe_listener_copy2 = safe_listener.clone();
        println!("Echoing message back to client!");
        let value = client_id_copy.clone();
        let echo_thread = spawn(move || {
            println!("Sending echo message!");
            rumtk_v2_mllp_send!(safe_listener_copy2, &value, HL7_V2_PDF_MESSAGE).unwrap();
            println!("Sent echo message!");
        });
        rumtk_sleep!(1);
        let echoed_messages = rumtk_exec_task!(async || -> RUMResult<MLLPClientMessages> {
            println!("Echoing message back to client!");
            let mut echoed_messages = safe_client
                .lock()
                .await
                .receive_client_messages(&client_id_copy)
                .await?;
            while echoed_messages.len() == 0 {
                echoed_messages = safe_client
                    .lock()
                    .await
                    .receive_client_messages(&client_id_copy)
                    .await?;
            }
            println!("Echoed message: {}", &echoed_messages.first().unwrap());
            Ok(echoed_messages)
        })
        .unwrap();
        let echoed_message = echoed_messages.get(0).unwrap();

        assert_eq!(
            &HL7_V2_PDF_MESSAGE,
            &echoed_message,
            "{}",
            rumtk_format!(
                "Issue echoing message through channel! Expected: {} Received: {}",
                &HL7_V2_PDF_MESSAGE,
                &echoed_message
            )
        )
    }

    ////////////////////////////JSON Tests/////////////////////////////////

    #[test]
    fn test_deserialize_escaped_v2_message() {
        let message = rumtk_v2_parse_message!(V2_JSON_MESSAGE).unwrap();
        let serialized = rumtk_serialize!(&message).unwrap();
        let escaped = basic_escape(&serialized, Some(&vec![]));
        let deserialized = rumtk_deserialize!(&escaped).unwrap();

        assert_eq!(
            message, deserialized,
            "Deserialized JSON does not match the expected value!"
        );
    }

    #[test]
    fn test_deserialize_v2_message() {
        let message = rumtk_v2_parse_message!(DEFAULT_HL7_V2_MESSAGE).unwrap();
        let message_str = rumtk_serialize!(&message).unwrap();
        let deserialized: V2Message = rumtk_deserialize!(&message_str).unwrap();

        assert_eq!(
            message, deserialized,
            "Deserialized JSON does not match the expected value!"
        );
    }

    #[test]
    fn test_parse_serialize_deserialize_serialize_v2_message() {
        let message = rumtk_v2_parse_message!(DEFAULT_HL7_V2_MESSAGE).unwrap();
        let message_str = rumtk_serialize!(&message).unwrap();
        let deserialized: V2Message = rumtk_deserialize!(&message_str).unwrap();
        let deserialized_str = rumtk_serialize!(&deserialized).unwrap();

        assert_eq!(
            message_str, deserialized_str,
            "Deserialized JSON does not match the expected value!"
        );
    }

    #[test]
    fn test_deserialize_large_v2_message() {
        let message = rumtk_v2_parse_message!(V2_TEST_LARGE_MESSAGE).unwrap();
        let message_str = rumtk_serialize!(&message).unwrap();
        let deserialized: V2Message = rumtk_deserialize!(&message_str).unwrap();

        assert_eq!(
            message, deserialized,
            "Deserialized JSON does not match the expected value!"
        );
    }

    #[test]
    fn test_deserialize_stdin_v2_message_basic() {
        println!("{:?}", V2_JSON_MESSAGE_BASIC);
        let expected_message = rumtk_v2_parse_message!(V2_JSON_MESSAGE_BASIC).unwrap();
        println!("{:?}", expected_message.to_string());
        println!("{:?}", rumtk_serialize!(&expected_message).unwrap());

        let deserialized = rumtk_deserialize!(&ESCAPED_V2_JSON_MESSAGE_BASIC).unwrap();

        assert_eq!(
            expected_message, deserialized,
            "Deserialized Escaped JSON does not match the expected value!"
        );
    }


    ////////////////////////////Benchmark Tests/////////////////////////////////
    #[test]
    fn test_buffer_find_segments() {
        let buffer = V2_TEST_LARGE_MESSAGE.as_bytes();

        let (r, time) = rumtk_benchmark_snippet!(|| buffer_find(buffer, &['\n' as u8]));

        assert!(time <= 1000, "buffer find of segments in large message took {} microseconds [> 1000000 us]!", time);
        assert_eq!(r, 465, "buffer find return the wrong first index of \n!");
    }

    #[test]
    fn test_buffer_find_all_segments() {
        let buffer = V2_TEST_LARGE_MESSAGE.as_bytes();

        let (r, time) = rumtk_benchmark_snippet!(|| buffer_find_instances(buffer, &['\n' as u8]));

        assert!(time <= 20000, "buffer find of segments in large message took {} microseconds [> 10000 us]!", time);
    }

    ///
    /// This micro benchmark exists to validate that splitting a Bytes buffer is cheap and that sources of
    /// slowness come from somewhere else.
    ///
    #[test]
    fn test_buffer_basic_split_segments() {
        let mut buffer = RUMBuffer::from(V2_TEST_LARGE_MESSAGE.as_bytes());

        let (r, time) = rumtk_benchmark_snippet!(|| {
            let split_count = buffer.len() / CPU_SEARCH_WINDOW_256_SIZE;
            let mut splits = RUMVec::<RUMBuffer>::with_capacity(split_count);
            for i in 0..splits.len() {
                splits.push(buffer.split_to(CPU_SEARCH_WINDOW_256_SIZE).unwrap());
            }

            splits
        });

        assert!(time <= 3000, "basic buffer splits of large message took {} microseconds [> 3000 us]!", time);
    }

    #[test]
    fn test_buffer_split_fast_segments() {
        let buffer = RUMBuffer::from(V2_TEST_LARGE_MESSAGE.as_bytes());

        let (r, time) = rumtk_benchmark_snippet!(|| for b in buffer.split_fast('\r' as u8) {});
        
        assert!(time <= 5000, "buffer split of segments in large message took {} microseconds [> 5000 us]!", time);
    }

    #[test]
    fn test_buffer_replace_fragment() {
        let pattern = "4050097";
        let replacement = "405009789";
        let buffer = RUMBuffer::from(V2_TEST_LARGE_MESSAGE.as_bytes());

        let (r, time) = rumtk_benchmark_snippet!(|| buffer_replace(&buffer, pattern.as_bytes(), replacement.as_bytes()));

        assert!(time <= 100000, "buffer replace of segments in large message took {} microseconds [> 100000 us]!", time);
    }

    #[test]
    fn test_buffer_replace_in_place() {
        let pattern = "4050097";
        let replacement = "4050098";
        let mut buffer = RUMBuffer::from(V2_TEST_LARGE_MESSAGE.as_bytes());

        let (r, time) = rumtk_benchmark_snippet!(|| {
            buffer_replace_in_place(&mut buffer, pattern.as_bytes(), replacement.as_bytes());
        });

        assert!(time <= 20000, "buffer replace of segments in large message took {} microseconds [> 20000 us]!", time);
    }

    #[test]
    fn test_vec_serialization() {
        #[derive(RUMSerJson, RUMDeJson, PartialEq, Debug)]
        struct Points {
            x: usize,
            y: usize,
        }

        impl Points{
            pub fn new() -> Self {
                Self {
                    x: 0,
                    y: 0
                }
            }

            pub fn from(x: usize) -> Self {
                Self {
                    x: x.clone(),
                    y: x
                }
            }
        }
        let data = vec![Some(Points::from(1)), None, Some(Points::from(2)), Some(Points::from(3))];
        let expected = vec![Some(Points::from(1)), None, Some(Points::from(2)), Some(Points::from(3))];

        let serialized = to_json(&data).unwrap();
        let deserialized: RUMVec<Option<Points>> = from_json(&serialized).unwrap();

        assert_eq!(expected, deserialized);
    }

    #[test]
    fn test_parser_benchmark() {
        let buffer = RUMBuffer::from(V2_TEST_LARGE_MESSAGE.as_bytes());

        let (r, time) = rumtk_benchmark_snippet!(|| V2Message::try_from_buffer(buffer));

        println!("Parsed message in {} us", &time);

        assert!(time <= 100000, "V2Message parsing took {} microseconds [> 100000 us]!", time);
    }

    #[test]
    fn test_parser_including_message_drop_benchmark() {
        let buffer = RUMBuffer::from(V2_TEST_LARGE_MESSAGE.as_bytes());

        let (r, time) = rumtk_benchmark_snippet!(|| {
            let message = V2Message::try_from_buffer(buffer).unwrap();
            drop(message);
        });

        println!("Parsed message in {} us", &time);

        assert!(time <= 100000, "V2Message parsing took {} microseconds [> 100000 us]!", time);
    }

    #[test]
    fn test_parser_including_small_message_drop_benchmark() {
        let buffer = RUMBuffer::from(HL7_V2_REPEATING_FIELD_MESSAGE.as_bytes());

        let (r, time) = rumtk_benchmark_snippet!(|| {
            let message = V2Message::try_from_buffer(buffer).unwrap();
            drop(message);
        });

        println!("Parsed message in {} us", &time);

        assert!(time <= 2000, "V2Message parsing took {} microseconds [> 2000 us]!", time);
    }

    ////////////////////////////Message Parse Speed Tests/////////////////////////////////

    #[test]
    fn test_scan_msh_segment() {
        let input = HL7_V2_MSH_ONLY;
        let (tok, segment_indices) = cpu_collect(input.as_bytes(), b'|', 0);
        let expected: CPUTokenIndexCollection = vec![3, 8, 19, 30, 41, 52, 76, 77, 93, 122, 124, 130, 131, 132, 135, 138, 139, 140, 141, 142, 156, 167];

        println!("{}", input);

        assert_eq!(
            segment_indices, expected,
            "MSH Segment lookahead mismatch!"
        );
    }

    #[test]
    fn test_scan_msh_segment_benchmark() {
        let input = HL7_V2_MSH_ONLY;

        let (r, time) = rumtk_benchmark_snippet!(|| cpu_collect(input.as_bytes(), b'|', 0));

        println!("Parsed message in {} us", &time);

        assert!(time <= 300, "MSH segment scanning took {} microseconds [> 300 us]!", time);
    }

    #[test]
    fn test_scan_msh_segment2() {
        let input = EXPECTED_MSH_SEGMENT;
        let segment_indices = cpu_collect(input.as_bytes(), b'|', 0);

        println!("{}", input);

        assert_eq!(
            segment_indices.1.len(), 13,
            "MSH Segment lookahead result length mismatch!"
        );
    }

    #[test]
    fn test_scan_large_message() {
        let input = V2_TEST_LARGE_MESSAGE;
        let segment_indices = cpu_collect(input.as_bytes(), b'|', 0);
        let expected: CPUTokenIndexCollection = HL7_V2_MESSAGE_TOKEN_POSITIONS.clone();

        assert_eq!(
            segment_indices.1, expected,
            "Failed to scan large message!"
        );
    }

    #[test]
    fn test_scan_large_message_benchmark() {
        let input = V2_TEST_LARGE_MESSAGE;

        let (r, time) = rumtk_benchmark_snippet!(|| cpu_collect(input.as_bytes(), b'|', 0));

        println!("Parsed message in {} us", &time);

        assert!(time <= 10000, "V2Message scanning took {} microseconds [> 10000 us]!", time);
    }

    ////////////////////////////Fuzzed Tests/////////////////////////////////

    #[test]
    fn test_fuzzed_no_msh() {
        let input = RUMBuffer::from(&[136u8]);
        match rumtk_v2_parse_message!(&input) {
            Err(e) => println!("Correctly identified input as garbage! => {}", &e),
            Ok(message) => {
                println!("Test input [{:?}] Result => {:?}", &input, message);
                panic!("Message parsed without errors despite being malformed!")
            }
        }
    }

    #[test]
    fn test_fuzzed_garbage_parsing() {
        let input = "MSH@~��MS";
        match rumtk_v2_parse_message!(&input) {
            Err(e) => println!("Correctly identified input as garbage! => {}", &e),
            Ok(message) => {
                println!("Test input [{:?}] Result => {:?}", &input, message);
                panic!("Message parsed without errors despite being malformed!")
            }
        }
    }

    #[test]
    fn test_fuzzed_bad_index() {
        let input = "MSH\u{226}\u{000}\u{202}:\u{250}\u{226}\u{204}\u{226}\u{000}\u{000}\u{000}\u{000}\u{000}?SHl\u{000}\u{000}=\u{000}\u{000}\u{000}\u{000}\u{000}H\u{000}\u{224}@Hl\u{000}\u{000}\u{000}\u{000}\u{000}\u{000}\u{000}\u{226}\u{012}\u{366}\u{250}\u{226}\u{204}\u{366}";
        match rumtk_v2_parse_message!(&input) {
            Err(e) => println!("Correctly identified input as garbage! => {}", &e),
            Ok(message) => {
                println!("Test input [{:?}] Result => {:?}", &input, message);
                panic!("Message parsed without errors despite being malformed!")
            }
        }
    }
}
