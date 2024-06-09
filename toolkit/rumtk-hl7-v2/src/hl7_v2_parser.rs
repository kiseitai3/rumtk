/*
    The V2 Parser module will contain a simple and lightweight message parser that will generate a
    structure following the message structure in the HL7 Specifications.
    The V2Message type here will provide a basic interface for navigating through the mapped
    segments and fields.
    From here, we will then write a schema driven interpreter module (see other source files in
    crate). That interpreter will try to generate a message structure using the specified HL7
    types. That structure will be exportable to JSON and (maybe) XML.
 */
//https://v2.hl7.org/conformance/HL7v2_Conformance_Methodology_R1_O1_Ballot_Revised_D9_-_September_2019_Introduction.html#:~:text=The%20base%20HL7%20v2%20standard,message%20definition%20is%20called%20profiling.
//https://www.hl7.org/implement/standards/product_brief.cfm?product_id=185

mod v2_parser {
    use std::collections::hash_map::{HashMap};
    use unicode_segmentation::UnicodeSegmentation;
    use crate::hl7_v2_types::v2_types::{V2String, V2DateTime};
    use crate::hl7_v2_constants::MSHEADER_PATTERN;

    const V2_DELETE_FIELD: &str = "\"\"";
    struct V2Component {
        component: V2String,
        delete_data: bool
    }

    impl V2Component {
        fn new() -> V2Component {
            V2Component{component: V2String::new(), delete_data: false}
        }

        fn from(item: &String) -> V2Component {
            V2Component{component: V2String::from(item), delete_data: item == V2_DELETE_FIELD}
        }

        fn is_empty(self) -> bool {
            self.component == ""
        }

        fn as_datetime(self) -> V2DateTime {
            V2DateTime::from_v2_string(self.component)
        }

        fn as_bool(self) -> bool {
            self.component.parse::<bool>().unwrap()
        }

        fn as_integer(self) -> i64 {
            self.component.parse::<i64>().unwrap()
        }

        fn as_float(self) -> f64 {
            self.component.parse::<f64>().unwrap()
        }
    }

    type FieldList = Vec<V2Component>;
    struct V2Field {
        components: FieldList
    }

    impl V2Field {
        fn new() -> V2Field {
            V2Field{components: FieldList::new()}
        }

        fn from_string(val: &String, divider: &String) -> V2Field {
            let comp_vec: Vec<&str> = val.split(divider).collect();
            let mut component_list: FieldList = FieldList::new();
            for c in comp_vec {
                component_list.push(V2Component::from(&String::from(c)));
            }
            V2Field{components: component_list}
        }

        fn len(self) -> usize {
            self.components.len()
        }
    }

    type V2Segment = Vec<V2Field>;
    type V2SegmentGroup = Vec<V2Segment>;
    type SegmentMap = HashMap<String, V2SegmentGroup>;

    struct V2ParseCharacters {
        segment_terminator: char,
        field_separator: char,
        component_separator: char,
        repetition_separator: char,
        escape_character: char,
        subcomponent_separator: char,
        truncation_character: char
    }

    const SEGMENT_TERMINATOR: char = '\r';

    impl V2ParseCharacters {
        fn from(msg_key_chars: &str) -> V2ParseCharacters {
            let mut parse_chars = msg_key_chars.chars();

            match parse_chars.count() {
                6 => V2ParseCharacters {
                    segment_terminator: SEGMENT_TERMINATOR,
                    field_separator: parse_chars.next().unwrap(),
                    component_separator: parse_chars.next().unwrap(),
                    repetition_separator: parse_chars.next().unwrap(),
                    escape_character: parse_chars.next().unwrap(),
                    subcomponent_separator: parse_chars.next().unwrap(),
                    truncation_character: parse_chars.next().unwrap(),
                },
                _ => V2ParseCharacters {
                    segment_terminator: SEGMENT_TERMINATOR,
                    field_separator: parse_chars.next().unwrap(),
                    component_separator: parse_chars.next().unwrap(),
                    repetition_separator: parse_chars.next().unwrap(),
                    escape_character: parse_chars.next().unwrap(),
                    subcomponent_separator: parse_chars.next().unwrap(),
                    truncation_character: '#'
                }
            }
        }
    }

    struct V2Message {
        separators: V2ParseCharacters,
        default_segment: V2SegmentGroup,
        segment_groups: SegmentMap
    }

    impl V2Message {
        fn from(raw_msg: &String) -> Result<V2Message, Err> {
            let segments = Self::tokenize_segments(&raw_msg);
            let parse_characters = Self::extract_parse_chars(&segments[0]);



            V2Message{default_segment: V2SegmentGroup::new(), segment_groups: SegmentMap::new()}
        }

        fn len(self) -> usize {
            self.segment_groups.len()
        }

        fn is_repeat_segment(self, segment_name: &String) -> bool {
            let _segment_group: &V2SegmentGroup = self.find_segment(segment_name);
            _segment_group.len() > 1
        }

        fn is_msh(msh_segment_token: &str) -> bool {
            match msh_segment_token[0..4] {
                MSHEADER_PATTERN => true,
                _ => false
            }
        }

        fn segment_exists(self, segment_name: &String) -> bool {
            let _segment_group: &V2SegmentGroup = self.find_segment(segment_name);
            _segment_group.len() > 0
        }

        fn find_segment(self, segment_name: &String) -> &V2SegmentGroup {
            match self.segment_groups.get(segment_name) {
                Some(segment_groups) => &segment_groups,
                None => &self.default_segment
            }
        }

        // Message parsing operations

        fn tokenize_segments(raw_message: &String) -> Vec<&str> {
            //Per Figure 2-1. Delimiter values of the HL7 v2 2.9 standard, each segment is separated
            // by a carriage return <cr>. The value cannot be changed by implementers.
            raw_message.split(SEGMENT_TERMINATOR).collect()
        }

        fn extract_parse_chars(msh_segment: &str) -> V2ParseCharacters {
            assert!(Self::is_msh(msh_segment), "The first segment is not an MSH segment! This message is malformed!");
            V2ParseCharacters::from(&msh_segment[4..])
        }

        fn extract_segment(segment: &str, parse_chars: &V2ParseCharacters) -> V2Segment {

        }
    }

    struct V2Parser {

    }

    impl V2Parser {
        fn parse(raw_message: &String) -> V2Message {
            let msg_lines: Vec<&str> = raw_message.split().collect();
        }

        fn tokenize_segments(raw_message: &String) -> Vec<&str> {
            //Per Figure 2-1. Delimiter values of the HL7 v2 2.9 standard, each segment is separated
            // by a carriage return <cr>. The value cannot be changed by implementers.
            raw_message.split('\r').collect()
        }
    }
}