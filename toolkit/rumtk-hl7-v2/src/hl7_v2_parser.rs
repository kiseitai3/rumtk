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

pub mod v2_parser {
    use std::collections::hash_map::{HashMap};
    use std::collections::VecDeque;
    use rumtk_core::strings::{UTFStringExtensions};
    use crate::hl7_v2_types::v2_types::{V2String, V2DateTime};
    use crate::hl7_v2_constants::{V2_MSHEADER_PATTERN, V2_SEGMENT_TYPES, V2_DELETE_FIELD,
                                  V2_SEGMENT_TERMINATOR, V2_TRUNCATION_CHARACTER, V2_EMPTY_STRING};

    pub type V2Result<T> = Result<T, String>;

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
            V2DateTime::from_v2_string(&self.component)
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

    type ComponentList = Vec<V2Component>;
    struct V2Field {
        components: ComponentList
    }

    impl V2Field {
        fn new() -> V2Field {
            V2Field{components: ComponentList::new()}
        }

        fn from_string(val: &str, parser_chars: &V2ParserCharacters) -> V2Field {
            let comp_vec: Vec<&str> = val.split(parser_chars.component_separator.as_str()).collect();
            let mut component_list: ComponentList = ComponentList::new();
            for c in comp_vec {
                component_list.push(V2Component::from_string(c, parser_chars));
            }
            V2Field{components: component_list}
        }

        fn len(&self) -> usize {
            self.components.len()
        }
    }

    type V2FieldList = Vec<V2Field>;
    struct V2Segment {
        name: String,
        description: String,
        fields: V2FieldList
    }

    impl V2Segment {
        fn from_string(raw_segment: &str, parser_chars: &V2ParserCharacters) -> V2Result<Self> {
            let raw_fields: Vec<&str> = raw_segment.split(parser_chars.field_separator.as_str()).collect();
            let raw_field_count = raw_fields.len();

            if raw_field_count <= 0 {
                return Err(format!("Error splitting segments into fields!\nRaw segment: {}\nField separator: {}", &raw_segment, &parser_chars.field_separator))
            }

            let mut fields: VecDeque<V2Field> = VecDeque::with_capacity(raw_fields.len());
            let mut field_list = V2FieldList::with_capacity(raw_fields.len() - 1);

            for raw_field in raw_fields {
                fields.push_back(V2Field::from_string(&raw_field, &parser_chars))
            }

            let field_name = match fields.pop_front() {
                Some(field) => match field.components.get(0) {
                    Some(name) => name.component.to_uppercase(),
                    None => return Err(format!("Expected at least one component in field but got None!\nRaw segment: {}", &raw_segment))
                },
                None => return Err(format!("Expected field but got None!\nRaw segment: {}", &raw_segment))
            };
            let field_description = String::from(
                match V2_SEGMENT_TYPES.get(&field_name){
                    Some(description) => &description,
                    None => V2_EMPTY_STRING
                });

            for field in fields {
                field_list.push(field);
            }

            Ok(V2Segment { name: field_name, description: field_description, fields: field_list })
        }
    }

    type V2SegmentGroup = Vec<V2Segment>;
    type SegmentMap = HashMap<String, V2SegmentGroup>;

    struct V2ParserCharacters {
        segment_terminator: String,
        field_separator: String,
        component_separator: String,
        repetition_separator: String,
        escape_character: String,
        subcomponent_separator: String,
        truncation_character: String
    }

    impl V2ParserCharacters {
        fn from(msg_key_chars: &str) -> V2Result<Self> {
            let field_separator: &str = msg_key_chars.get_grapheme(0);
            let encoding_field: Vec<&str> = msg_key_chars.split(&field_separator).collect();
            let parser_chars: &str = encoding_field[1];


            match parser_chars.count_graphemes() {
                5 => Ok(V2ParserCharacters {
                    segment_terminator: V2_SEGMENT_TERMINATOR.to_string(),
                    field_separator: field_separator.to_string(),
                    component_separator: parser_chars.get_grapheme(0).to_string(),
                    repetition_separator: parser_chars.get_grapheme(1).to_string(),
                    escape_character: parser_chars.get_grapheme(2).to_string(),
                    subcomponent_separator: parser_chars.get_grapheme(3).to_string(),
                    truncation_character: parser_chars.get_grapheme(4).to_string(),
                }),
                4 => Ok(V2ParserCharacters {
                    segment_terminator: V2_SEGMENT_TERMINATOR.to_string(),
                    field_separator: field_separator.to_string(),
                    component_separator: parser_chars.get_grapheme(0).to_string(),
                    repetition_separator: parser_chars.get_grapheme(1).to_string(),
                    escape_character: parser_chars.get_grapheme(2).to_string(),
                    subcomponent_separator: parser_chars.get_grapheme(3).to_string(),
                    truncation_character: V2_TRUNCATION_CHARACTER.to_string()
                }),
                _ => Err("Wrong count of parsing characters in message header!".to_string())
            }
        }

        fn from_msh(msh_segment: &str) -> V2Result<Self> {
            if V2ParserCharacters::is_msh(msh_segment) {
                Ok(V2ParserCharacters::from(&msh_segment[4..]).unwrap())
            } else {
                Err("The segment is not an MSH segment! This message is malformed!".to_string())
            }
        }

        fn is_msh(msh_segment_token: &str) -> bool {
            match &msh_segment_token[0..4] {
                V2_MSHEADER_PATTERN => true,
                _ => false
            }
        }
    }

    pub struct V2Message {
        separators: V2ParserCharacters,
        default_segment: V2SegmentGroup,
        segment_groups: SegmentMap
    }

    impl V2Message {
        pub fn from(raw_msg: &str) -> V2Result<Self> {
            let clean_msg = V2Message::sanitize(&raw_msg);
            let segment_tokens = V2Message::tokenize_segments(&clean_msg.as_str());
            let parse_characters = match V2ParserCharacters::from(&segment_tokens[0]){
                Ok(parser_chars) => parser_chars,
                Err(why) => return Err(why)
            };
            let segments = match V2Message::extract_segments(&segment_tokens, &parse_characters){
                Ok(segments) => segments,
                Err(e) => return Err(e)
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

        fn sanitize(raw_message: &str) -> String {
            let rr_string = raw_message.replace("\n", "\r");
            let mut n_string = rr_string.replace("\r\r", "\r");
            while(n_string.contains("\r\r")) {
                n_string = n_string.replace("\r\r", "\n");
            }
            n_string
        }

        fn tokenize_segments(raw_message: &str) -> Vec<&str> {
            //Per Figure 2-1. Delimiter values of the HL7 v2 2.9 standard, each segment is separated
            // by a carriage return <cr>. The value cannot be changed by implementers.
            raw_message.split(V2_SEGMENT_TERMINATOR).collect()
        }

        fn extract_segments(raw_segments: &Vec<&str>, parser_chars: &V2ParserCharacters) -> V2Result<SegmentMap> {
            let mut segments: SegmentMap = SegmentMap::new();

            for segment_str in raw_segments {
                let segment: V2Segment = match V2Segment::from_string(segment_str, parser_chars){
                    Ok(segment_value) => segment_value,
                    Err(why) => return Err(why)
                };

                let key = String::from(&segment.name);
                if segments.contains_key(&segment.name) == false {
                    segments.insert(key, V2SegmentGroup::new());
                }
                segments.get_mut(&segment.name).unwrap().push(segment);
            }

            Ok(segments)
        }
    }
}