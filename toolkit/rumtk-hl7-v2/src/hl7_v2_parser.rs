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
    use std::any::TypeId;
    use std::ops::{Index, IndexMut};
    use std::collections::hash_map::{HashMap};
    use std::collections::VecDeque;
    use rumtk_core::strings::{UTFStringExtensions};
    use crate::hl7_v2_types::v2_types::{V2String, V2DateTime};
    use crate::hl7_v2_constants::{V2_MSHEADER_PATTERN, V2_SEGMENT_TYPES, V2_DELETE_FIELD,
                                  V2_SEGMENT_TERMINATOR, V2_TRUNCATION_CHARACTER, V2_EMPTY_STRING};

    pub type V2Result<T> = Result<T, String>;

    #[derive(Debug)]
    pub struct V2Component {
        component: V2String,
        delete_data: bool
    }

    impl V2Component {
        fn new() -> V2Component {
            V2Component{component: V2String::new(), delete_data: false}
        }

        pub fn from_string(item: &str) -> V2Component {
            V2Component{component: V2String::from(item), delete_data: item == V2_DELETE_FIELD}
        }

        pub fn is_empty(&self) -> bool {
            self.component == ""
        }

        pub fn as_datetime(&self) -> V2DateTime {
            V2DateTime::from_v2_string(&self.component)
        }

        pub fn as_bool(&self) -> bool {
            self.component.parse::<bool>().unwrap()
        }

        pub fn as_integer(&self) -> i64 {
            self.component.parse::<i64>().unwrap()
        }

        pub fn as_float(&self) -> f64 {
            self.component.parse::<f64>().unwrap()
        }

        pub fn as_str(&self) -> &str {
            self.component.as_str()
        }

        pub fn value<T>(&self) -> T {
            match TypeId::of::<T>() {
                TypeId::of::<V2DateTime>() => self.as_datetime(),
                TypeId::of::<bool>() => self.as_bool(),
                TypeId::of::<i8>() => self.as_integer() as i8,
                TypeId::of::<u8>() => self.as_integer() as u8,
                TypeId::of::<i16>() => self.as_integer() as i16,
                TypeId::of::<u16>() => self.as_integer() as u16,
                TypeId::of::<i32>() => self.as_integer() as i32,
                TypeId::of::<u32>() => self.as_integer() as u32,
                TypeId::of::<i64>() => self.as_integer(),
                TypeId::of::<usize>() => self.as_integer(),
                TypeId::of::<f32>() => self.as_float() as f32,
                TypeId::of::<f64>() => self.as_float(),
                TypeId::of::<str>() => self.as_str(),
                TypeId::of::<String>() => self.as_str().to_string(),
                _ => self.as_str()
            }
        }
    }

    pub type ComponentList = Vec<V2Component>;
    #[derive(Debug)]
    pub struct V2Field {
        components: ComponentList
    }

    impl V2Field {
        pub fn new() -> V2Field {
            V2Field{components: ComponentList::new()}
        }

        pub fn from_string(val: &str, parser_chars: &V2ParserCharacters) -> V2Field {
            let comp_vec: Vec<&str> = val.split(parser_chars.component_separator.as_str()).collect();
            let mut component_list: ComponentList = ComponentList::new();
            for c in comp_vec {
                component_list.push(V2Component::from_string(c));
            }
            V2Field{components: component_list}
        }

        pub fn len(&self) -> usize {
            self.components.len()
        }

        pub fn get(&self, indx: usize) -> V2Result<&V2Component> {
            match self.components.get(indx) {
                Some(component) => Ok(component),
                None => Err(format!("Component at index {} not found!", indx))
            }
        }

        pub fn get_mut(&mut self, indx: usize) -> V2Result<&mut V2Component> {
            match self.components.get_mut(indx) {
                Some(component) => Ok(component),
                None => Err(format!("Component at index {} not found!", indx))
            }
        }
    }

    impl Index<usize> for V2Field {
        type Output = V2Component;
        fn index(&self, indx: usize) -> V2Result<&Self::Output> {
            self.get(indx)
        }
    }

    impl IndexMut<usize> for V2Field {
        fn index_mut(&mut self, indx: usize) -> V2Result<&mut V2Component> {
            self.get_mut(indx)
        }
    }

    pub type V2FieldList = Vec<V2Field>;
    #[derive(Debug)]
    pub struct V2Segment {
        name: String,
        description: String,
        fields: V2FieldList
    }

    impl V2Segment {
        pub fn from_string(raw_segment: &str, parser_chars: &V2ParserCharacters) -> V2Result<Self> {
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

        pub fn get(&self, indx: usize) -> V2Result<&V2Field> {
            match self.fields.get(indx) {
                Some(field) => Ok(field),
                None => Err(format!("Field number {} not found!", indx))
            }
        }

        pub fn get_mut(&mut self, indx: usize) -> V2Result<&mut V2Field> {
            match self.fields.get_mut(indx) {
                Some(field) => Ok(field),
                None => Err(format!("Field number {} not found!", indx))
            }
        }
    }

    impl Index<usize> for V2Segment {
        type Output = V2Field;
        fn index(&self, indx: usize) -> V2Result<&Self::Output> {
            self.get(indx)
        }
    }

    impl IndexMut<usize> for V2Segment {
        fn index_mut(&mut self, indx: usize) -> V2Result<&mut V2Field> {
            self.get_mut(indx)
        }
    }

    pub type V2SegmentGroup = Vec<V2Segment>;
    pub type SegmentMap = HashMap<String, V2SegmentGroup>;

    #[derive(Debug)]
    pub struct V2ParserCharacters {
        pub segment_terminator: String,
        pub field_separator: String,
        pub component_separator: String,
        pub repetition_separator: String,
        pub escape_character: String,
        pub subcomponent_separator: String,
        pub truncation_character: String
    }

    impl V2ParserCharacters {
        pub fn new() -> V2ParserCharacters {
            V2ParserCharacters {
                segment_terminator: V2_SEGMENT_TERMINATOR.to_string(),
                field_separator: String::from("|"),
                component_separator: String::from("^"),
                repetition_separator: String::from("~"),
                escape_character: String::from("\\"),
                subcomponent_separator: String::from("&"),
                truncation_character: String::from("#"),
            }
        }
        pub fn from(msg_key_chars: &str) -> V2Result<Self> {
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

        pub fn from_msh(msh_segment: &str) -> V2Result<Self> {
            if V2ParserCharacters::is_msh(msh_segment) {
                V2ParserCharacters::from(&msh_segment[3..])
            } else {
                Err("The segment is not an MSH segment! This message is malformed!".to_string())
            }
        }

        fn is_msh(msh_segment_token: &str) -> bool {
            &msh_segment_token[0..3] == V2_MSHEADER_PATTERN
        }
    }

    pub struct V2Message {
        separators: V2ParserCharacters,
        segment_groups: SegmentMap
    }

    impl V2Message {
        pub fn from(raw_msg: &str) -> V2Result<Self> {
            let clean_msg = V2Message::sanitize(&raw_msg);
            let segment_tokens = V2Message::tokenize_segments(&clean_msg.as_str());
            let parse_characters = match V2ParserCharacters::from_msh(&segment_tokens[0]){
                Ok(parser_chars) => parser_chars,
                Err(why) => return Err(why)
            };
            let segments = match V2Message::extract_segments(&segment_tokens, &parse_characters){
                Ok(segments) => segments,
                Err(e) => return Err(e)
            };


            Ok(V2Message {
                separators: parse_characters,
                segment_groups: segments
            })
        }

        pub fn len(&self) -> usize {
            self.segment_groups.len()
        }

        pub fn get(&self, segment_name: &str, sub_segment: usize) -> V2Result<&V2Segment> {
            let segment_group = self.get_group(segment_name).unwrap();
            match segment_group.get(sub_segment) {
                Some(segment) => Ok(segment),
                None => Err(format!("Subsegment {} was not found in segment group {}!", sub_segment, segment_name))
            }
        }

        pub fn get_mut(&mut self, segment_name: &str, sub_segment: usize) -> V2Result<&mut V2Segment> {
            let mut segment_group = self.get_mut_group(segment_name).unwrap();
            match segment_group.get_mut(sub_segment) {
                Some(segment) => Ok(segment),
                None => Err(format!("Subsegment {} was not found in segment group {}!", sub_segment, segment_name))
            }
        }

        pub fn get_group(&self, segment_name: &str) -> V2Result<&V2SegmentGroup> {
            match self.segment_groups.get(segment_name) {
                Some(segment_group) => Ok(segment_group),
                None => Err(format!("Segment {} not found inm message!", segment_name))
            }
        }

        pub fn get_mut_group(&mut self, segment_name: &str) -> V2Result<&mut V2SegmentGroup> {
            match self.segment_groups.get_mut(segment_name) {
                Some(segment_group) => Ok(segment_group),
                None => Err(format!("Segment {} not found inm message!", segment_name))
            }
        }

        pub fn is_repeat_segment(&self, segment_name: &String) -> bool {
            let _segment_group: &V2SegmentGroup = self.get_group(segment_name).unwrap();
            _segment_group.len() > 1
        }

        pub fn segment_exists(&self, segment_name: &String) -> bool {
            let _segment_group: &V2SegmentGroup = self.get_group(segment_name).unwrap();
            _segment_group.len() > 0
        }

        // Message parsing operations

        pub fn sanitize(raw_message: &str) -> String {
            let rr_string = raw_message.replace("\n", "\r");
            let mut n_string = rr_string.replace("\r\r", "\r");
            while(n_string.contains("\r\r")) {
                n_string = n_string.replace("\r\r", "\n");
            }
            n_string
        }

        pub fn tokenize_segments(raw_message: &str) -> Vec<&str> {
            //Per Figure 2-1. Delimiter values of the HL7 v2 2.9 standard, each segment is separated
            // by a carriage return <cr>. The value cannot be changed by implementers.
            raw_message.split(V2_SEGMENT_TERMINATOR).collect()
        }

        pub fn extract_segments(raw_segments: &Vec<&str>, parser_chars: &V2ParserCharacters) -> V2Result<SegmentMap> {
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

    impl Index<&'_ str> for V2Message {
        type Output = V2SegmentGroup;
        fn index(&self, segment_name: &str) -> V2Result<&Self::Output> {
            self.get_group(segment_name)
        }
    }

    impl IndexMut<&'_ str> for V2Message {
        fn index_mut(&mut self, segment_name: &str) -> V2Result<&mut V2SegmentGroup> {
            self.get_mut_group(segment_name)
        }
    }
}