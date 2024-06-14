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
    use std::collections::VecDeque;
    use unicode_segmentation::UnicodeSegmentation;
    use crate::hl7_v2_types::v2_types::{V2String, V2DateTime};
    use crate::hl7_v2_constants::{MSHEADER_PATTERN, V2_SEGMENT_TYPES, V2_DELETE_FIELD,
                                  V2_SEGMENT_TERMINATOR, V2_TRUNCATION_CHARACTER};

    type V2Result<T, E> = Result<T, E>;

    struct V2Component {
        component: V2String,
        delete_data: bool
    }

    impl V2Component {
        fn new() -> V2Component {
            V2Component{component: V2String::new(), delete_data: false}
        }

        fn from(item: &str) -> V2Component {
            V2Component{component: V2String::from(item), delete_data: item == V2_DELETE_FIELD}
        }

        fn from_string(item: &str, parser_chars: &V2ParserCharacters) -> V2Component {
            V2Component{component: V2String::from(item), delete_data: item == V2_DELETE_FIELD}
        }

        fn is_empty(&self) -> bool {
            self.component == ""
        }

        fn as_datetime(&self) -> V2DateTime {
            V2DateTime::from_v2_string(self.component)
        }

        fn as_bool(&self) -> bool {
            self.component.parse::<bool>().unwrap()
        }

        fn as_integer(&self) -> i64 {
            self.component.parse::<i64>().unwrap()
        }

        fn as_float(&self) -> f64 {
            self.component.parse::<f64>().unwrap()
        }

        fn as_str(&self) -> &str {
            self.component.as_str()
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

        fn from_string(val: &str, parser_chars: &V2ParserCharacters) -> V2Field {
            let comp_vec: Vec<&str> = val.split(parser_chars.component_separator).collect();
            let mut component_list: FieldList = FieldList::new();
            for c in comp_vec {
                component_list.push(V2Component::from_string(c, parser_chars));
            }
            V2Field{components: component_list}
        }

        fn len(&self) -> usize {
            self.components.len()
        }
    }

    type V2Fields = Vec<V2Field>;
    struct V2Segment {
        name: String,
        description: String,
        fields: V2Fields
    }

    impl V2Segment {
        fn from_string(raw_segment: &str, parser_chars: &V2ParserCharacters) -> V2Result<V2Segment, String> {
            let raw_fields: Vec<&str> = raw_segment.split(parser_chars.field_separator).collect();
            let raw_field_count = raw_fields.len();

            if raw_field_count <= 0 {
                return Err(format!("Error splitting segments into fields!\nRaw segment: {}\nField separator: {}", &raw_segment, &parser_chars.field_separator))
            }

            let mut field_list: VecDeque<V2Field> = VecDeque::with_capacity(raw_fields.len());

            for raw_field in raw_fields {
                field_list.push(V2Field::from_string(raw_field, parser_chars))
            }

            let field_name = match field_list.pop_front() {
                Some(field) => match field.components.get(0) {
                    Some(name) => name.component.to_uppercase(),
                    None => return Err(format!("Expected at least one component in field but got None!\nRaw segment: {}", &raw_segment))
                },
                None => return Err(format!("Expected field but got None!\nRaw segment: {}", &raw_segment))
            };
            let field_description = String::from(
                match V2_SEGMENT_TYPES.get(&field_name){
                    Some(description) => description,
                    None => return Err(format!("Field description not found! Field name: {}", &field_name))
                });

            Ok(V2Segment { name: field_name, description: field_description, fields: field_list })
        }
    }

    type V2SegmentGroup = Vec<V2Segment>;
    type SegmentMap = HashMap<String, V2SegmentGroup>;

    struct V2ParserCharacters {
        segment_terminator: char,
        field_separator: char,
        component_separator: char,
        repetition_separator: char,
        escape_character: char,
        subcomponent_separator: char,
        truncation_character: char
    }

    impl V2ParserCharacters {
        fn from(msg_key_chars: &str) -> V2Result<V2ParserCharacters, String> {
            let encoding_field: Vec<&str> = msg_key_chars.split(&msg_key_chars[0..1]).collect();
            let parser_chars: Vec<char> = encoding_field[0].chars().collect();


            match parser_chars.count() {
                6 => Ok(V2ParserCharacters {
                    segment_terminator: V2_SEGMENT_TERMINATOR,
                    field_separator: parser_chars.next().unwrap(),
                    component_separator: parser_chars.next().unwrap(),
                    repetition_separator: parser_chars.next().unwrap(),
                    escape_character: parser_chars.next().unwrap(),
                    subcomponent_separator: parser_chars.next().unwrap(),
                    truncation_character: parser_chars.next().unwrap(),
                }),
                5 => Ok(V2ParserCharacters {
                    segment_terminator: V2_SEGMENT_TERMINATOR,
                    field_separator: parser_chars.next().unwrap(),
                    component_separator: parser_chars.next().unwrap(),
                    repetition_separator: parser_chars.next().unwrap(),
                    escape_character: parser_chars.next().unwrap(),
                    subcomponent_separator: parser_chars.next().unwrap(),
                    truncation_character: V2_TRUNCATION_CHARACTER
                }),
                _ => Err(String::from("Wrong count of parsing characters in message header!"))
            }
        }

        fn from_msh(msh_segment: &str) -> V2Result<V2ParserCharacters, String> {
            if V2ParserCharacters::is_msh(msh_segment) {
                Ok(V2ParserCharacters::from(&msh_segment[4..]).unwrap())
            } else {
                Err(String::from("The segment is not an MSH segment! This message is malformed!"))
            }
        }

        fn is_msh(msh_segment_token: &str) -> bool {
            match msh_segment_token[0..4] {
                MSHEADER_PATTERN => true,
                _ => false
            }
        }
    }

    struct V2Message {
        separators: V2ParserCharacters,
        default_segment: V2SegmentGroup,
        segment_groups: SegmentMap
    }

    impl V2Message {
        fn from(raw_msg: &String) -> V2Result<V2Message, String> {
            let segment_tokens = V2Message::tokenize_segments(&raw_msg);
            let parse_characters = match V2ParserCharacters::extract_parser_chars(&segment_tokens[0]){
                Ok(parser_chars) => parser_chars,
                Err(why) => Err(why)
            };
            let segments = match V2Message::extract_segments(&segment_tokens, &parse_characters){
                Ok(segments) => segments,
                Err(e) => return e
            };


            Ok(V2Message {
                separators: parse_characters,
                default_segment: V2SegmentGroup::new(),
                segment_groups: segments
            })
        }

        fn len(&self) -> usize {
            self.segment_groups.len()
        }

        fn is_repeat_segment(&self, segment_name: &String) -> bool {
            let _segment_group: &V2SegmentGroup = self.find_segment(segment_name);
            _segment_group.len() > 1
        }

        fn segment_exists(&self, segment_name: &String) -> bool {
            let _segment_group: &V2SegmentGroup = self.find_segment(segment_name);
            _segment_group.len() > 0
        }

        fn find_segment(&self, segment_name: &String) -> &V2SegmentGroup {
            match self.segment_groups.get(segment_name) {
                Some(segment_groups) => &segment_groups,
                None => &self.default_segment
            }
        }

        // Message parsing operations

        fn tokenize_segments(raw_message: &String) -> Vec<&str> {
            //Per Figure 2-1. Delimiter values of the HL7 v2 2.9 standard, each segment is separated
            // by a carriage return <cr>. The value cannot be changed by implementers.
            raw_message.split(V2_SEGMENT_TERMINATOR).collect()
        }

        fn extract_segments(raw_segments: &Vec<&str>, parser_chars: &V2ParserCharacters) -> V2Result<SegmentMap, String> {
            let mut segments: SegmentMap = SegmentMap::new();

            for segment_str in raw_segments {
                let segment: V2Segment = match V2Segment::from_string(segment_str, parser_chars){
                    Ok(segment_value) => segment_value,
                    Err(why) => return Err(why)
                };
                segments[&segment.name].push(segment);
            }

            Ok(segments)
        }
    }
}