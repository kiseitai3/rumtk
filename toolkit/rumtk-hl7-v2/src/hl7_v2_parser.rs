/*
 * rumtk attempts to implement HL7 and medical protocols for interoperability in medicine.
 * This toolkit aims to be reliable, simple, performant, and standards compliant.
 * Copyright (C) 2024  Luis M. Santos, M.D.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
 */

///
/// The V2 Parser module will contain a simple and lightweight message parser that will generate a
/// structure following the message structure in the HL7 Specifications.
/// The V2Message type here will provide a basic interface for navigating through the mapped
/// segments and fields.
/// From here, we will then write a schema driven interpreter module (see other source files in
/// crate). That interpreter will try to generate a message structure using the specified HL7
/// types. That structure will be exportable to JSON and (maybe) XML.
///
/// [Conformance](https://v2.hl7.org/conformance/HL7v2_Conformance_Methodology_R1_O1_Ballot_Revised_D9_-_September_2019_Introduction.html#:~:text=The%20base%20HL7%20v2%20standard,message%20definition%20is%20called%20profiling.)
///
/// [Product Brief](https://www.hl7.org/implement/standards/product_brief.cfm?product_id=185)
///

pub mod v2_parser {
    pub use crate::hl7_v2_base_types::v2_primitives::{
        V2DateTime, V2ParserCharacters, V2PrimitiveCasting, V2Result, V2SearchIndex, V2String,
    };
    pub use crate::hl7_v2_constants::{
        V2_DELETE_FIELD, V2_EMPTY_STRING, V2_MSHEADER_PATTERN, V2_SEGMENT_DESC, V2_SEGMENT_IDS,
        V2_SEGMENT_TERMINATOR,
    };
    pub use rumtk_core::cache::{get_or_set_from_cache, new_cache, AHashMap, LazyRUMCache};
    use rumtk_core::core::clamp_index;
    use rumtk_core::json::serialization::{Deserialize, Serialize};
    use rumtk_core::rumtk_cache_fetch;
    use rumtk_core::strings::CompactStringExt;
    pub use rumtk_core::strings::{
        format_compact, try_decode_with, unescape_string, AsStr, RUMString, RUMStringConversions,
    };
    use std::ops::{Index, IndexMut};
    /**************************** Globals ***************************************/

    static mut search_cache: LazyRUMCache<RUMString, V2SearchIndex> = new_cache();

    /**************************** Helpers ***************************************/
    fn compile_search_index(search_pattern: &RUMString) -> V2SearchIndex {
        V2SearchIndex::from(search_pattern)
    }

    /**************************** Types *****************************************/
    ///
    /// V2Component.
    /// All V2Components contain the field's component data as a UTF-8 string.
    /// You can request a conversion to an atomic type via the as_* family of methods.
    ///
    /// ## Per Section 2.5.3.1
    ///
    /// ```text
    /// A field SHALL exist in one of three population states in an HL7 message:
    ///
    /// **Populated.** (Synonyms: valued, non-blank, not blank, not empty.) The sending system sends a value
    /// in the field. For example, if a sending system includes medical record number, that would be
    /// communicated as |1234567^^^MR^KP-CA|.
    ///
    /// **Not populated.** (Synonyms: unpopulated, not valued, unvalued, blank, empty, not present, missing.)
    /// The sending system does not supply a value for the field. The Sender might or might not have a value
    /// for the field. The receiving system can make no inference regarding the absence of an element value if
    /// there is not a conformance profile governing the implementation. However, if there is a Conformance
    /// Message Profile in effect, then special rules apply; see section 2.B, "Conformance Using Message
    /// Profiles".
    ///
    /// **Null. HL7 v2.x does not have an explicit concept for null values.**
    ///
    /// **Populated with Delete Indicator:** Any existing value for the corresponding data base element in the
    /// receiving application SHOULD be deleted. This is symbolically communicated as two double-quotes
    /// between the delimiters (i.e., |""|).Employing consecutive double quote characters as the only content of
    /// a field for other purposes is prohibited.
    /// ```
    ///
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub struct V2Component {
        component: V2String,
    }

    impl V2Component {
        fn new() -> V2Component {
            V2Component {
                component: V2String::from(""),
            }
        }

        ///
        /// Constructs HL7 V2 Component.
        /// ### Per Section 2.7
        /// Added support for unescaping escaped strings.
        /// Support is limited to control sequences and hex/unicode character sequences.
        /// Advanced ANSI Escape sequences are not supported at this layer.
        /// We let the receiving application further handle the advanced ANSI escape sequences as
        /// it best sees fit.
        ///
        /// ## Section 2.7.3
        ///
        /// Note => People have already created the conversion tables for the different encodings
        /// but auto detection of encoding is not 100% reliable. Care should be taken when using
        /// the resulting string.
        ///
        /// ## Single-byte character sets:
        /// ```text
        ///-      \C2842\ISO-IR6 G0 (ISO 646 : ASCII)
        ///-      \C2D41\ISO-IR100 (ISO 8859 : Latin Alphabet 1)
        ///-      \C2D42\ISO-IR101 (ISO 8859 : Latin Alphabet 2)
        ///-      \C2D43\ISO-IR109 (ISO 8859 : Latin Alphabet 3)
        ///-      \C2D44\ISO-IR110 (ISO 8859 : Latin Alphabet 4)
        ///-      \C2D4C\ISO-IR144 (ISO 8859 : Cyrillic)
        ///-      \C2D47\ISO-IR127 (ISO 8859 : Arabic)
        ///-      \C2D46\ISO-IR126 (ISO 8859 : Greek)
        ///-      \C2D48\ISO-IR138 (ISO 8859 : Hebrew)
        ///-      \C2D4D\ISO-IR148 (ISO 8859 : Latin Alphabet 5)
        ///-      \C284A\ISO-IR14 (JIS X 0201 -1976: Romaji)
        ///-      \C2949\ISO-IR13 (JIS X 0201 : Katakana)
        /// ```
        /// ## Multi-byte codes:
        /// ```text
        ///-      \M2442\ISO-IR87 (JIS X 0208 : Kanji, hiragana and katakana)
        ///-      \M242844\ISO-IR159 (JIS X 0212 : Supplementary Kanji)
        /// ```
        /// We grab the ASCII string.
        /// Cast it to bytes while unescaping any escape sequences.
        /// Guess the encoding of the bytes.
        /// Decode back to UTF-8.
        /// If all things go right, the UTF-8 string should be a faithful representative of the
        /// intended string per section 2.7 of the standard.
        ///
        /// Will not support 2.7.8 Local encodings (\Zxxyy) until needed in the wild.
        ///
        pub fn from_str(item: &str) -> V2Component {
            V2Component {
                component: V2String::from(item),
            }
        }

        pub fn to_string(&self) -> V2String {
            self.component.clone()
        }

        pub fn is_empty(&self) -> bool {
            self.component == ""
        }

        pub fn is_delete(&self) -> bool {
            self.component == V2_DELETE_FIELD
        }

        pub fn as_datetime(&self) -> V2DateTime {
            V2DateTime::from_str(&self.component)
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
    }

    impl AsStr for V2Component {
        fn as_str(&self) -> &str {
            self.component.as_str()
        }
    }

    impl V2PrimitiveCasting for V2Component {}

    pub type ComponentList = Vec<V2Component>;

    ///
    /// A field is a collection of items separated by the field separation character.
    ///
    /// ## Example
    ///
    /// PID5 in
    /// `PID|||3064985^^^^SR^~ML288^^^^PI^||CHILD^BABEE^^^^^^||20180911|F||2106-3^^^^^|22 YOUNGER LAND^^JUNEAU^WI^53039^^^^WI027^^||(920)386-5555^PRN^PH^^^920^3865555^^|||||||||2186-5^^^^^|||||||`
    /// is `CHILD^BABEE^^^^^^`
    ///
    /// ## Per Section 2.5.3
    ///
    /// ```text
    /// A field is a string of characters. Fields for use within HL7 segments are defined by HL7. A
    /// comprehensive data dictionary of all HL7 fields is provided in Appendix A.
    ///```
    ///
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub struct V2Field {
        components: ComponentList,
    }
    impl V2Field {
        pub fn new() -> V2Field {
            V2Field {
                components: ComponentList::new(),
            }
        }

        pub fn from_str(val: &str, parser_chars: &V2ParserCharacters) -> V2Field {
            let comp_vec: Vec<&str> = val
                .split(parser_chars.component_separator.as_str())
                .collect();
            let mut component_list: ComponentList = ComponentList::new();
            for c in comp_vec {
                component_list.push(V2Component::from_str(c));
            }
            V2Field {
                components: component_list,
            }
        }

        pub fn to_string(&self, parser_chars: &V2ParserCharacters) -> V2String {
            let mut components: Vec<&str> = Vec::with_capacity(self.components.len());
            for component in self.components.iter() {
                components.push(component.as_str())
            }
            components.join_compact(parser_chars.component_separator.as_str())
        }

        pub fn with_raw_str(val: &str) -> V2Field {
            let mut component_list: ComponentList = vec![V2Component::from_str(val)];
            V2Field {
                components: component_list,
            }
        }

        pub fn len(&self) -> usize {
            self.components.len()
        }

        pub fn get(&self, indx: isize) -> V2Result<&V2Component> {
            let component_indx = clamp_index(&indx, &(self.components.len() as isize))? - 1;
            match self.components.get(component_indx) {
                Some(component) => Ok(component),
                None => Err(format_compact!("Component at index {} not found!", indx)),
            }
        }

        pub fn get_mut(&mut self, indx: isize) -> V2Result<&mut V2Component> {
            let component_indx = clamp_index(&indx, &(self.components.len() as isize))? - 1;
            match self.components.get_mut(component_indx) {
                Some(component) => Ok(component),
                None => Err(format_compact!("Component at index {} not found!", indx)),
            }
        }
    }

    impl Index<isize> for V2Field {
        type Output = V2Component;
        fn index(&self, indx: isize) -> &V2Component {
            self.get(indx).unwrap()
        }
    }

    impl IndexMut<isize> for V2Field {
        fn index_mut(&mut self, indx: isize) -> &mut V2Component {
            self.get_mut(indx).unwrap()
        }
    }

    pub type V2FieldGroup = Vec<V2Field>;
    pub type V2FieldList = Vec<V2FieldGroup>;

    ///
    /// A segment comprises of a collection of items separated by the segment separator character.
    /// A segment is one line.
    ///
    /// ## Example
    ///
    /// - MSH|^~\\&|WIR11.3.2^^|WIR^^||WIRPH^^|20200514||VXU^V04^VXU_V04|2020051412382900|P^|2.5.1^^|||ER||||||^CDCPHINVS
    ///
    /// ## Per Section 2.5.2
    /// ```text
    /// A segment is a logical grouping of data fields. Segments of a message MAY be required or optional.
    /// They MAY occur only once in a message or they MAY be allowed to repeat. Each segment is given a
    /// name. For example, the ADT message MAY contain the following segments: Message Header (MSH),
    /// Event Type (EVN), Patient ID (PID), and Patient Visit (PV1).
    /// ```
    ///
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub struct V2Segment {
        name: RUMString,
        description: RUMString,
        fields: V2FieldList,
    }

    impl V2Segment {
        pub fn from_str(raw_segment: &str, parser_chars: &V2ParserCharacters) -> V2Result<Self> {
            let raw_fields: Vec<&str> = raw_segment
                .split(parser_chars.field_separator.as_str())
                .collect();
            let raw_field_count = raw_fields.len();

            if raw_field_count <= 0 {
                return Err(format_compact!(
                    "Error splitting segments into fields!\nRaw segment: {}\nField separator: {}",
                    &raw_segment,
                    &parser_chars.field_separator
                ));
            }

            let mut field_list = V2FieldList::with_capacity(raw_fields.len() - 1);
            let field_name = raw_fields[0].to_rumstring().to_uppercase();

            if raw_field_count > 1 {
                if field_name == "MSH" {
                    field_list.push(vec![V2Field::with_raw_str(raw_fields[1])]);
                    for i in 2..raw_field_count {
                        let raw_field = raw_fields[i];
                        field_list.push(Self::generate_subfields(raw_field, parser_chars));
                    }
                } else {
                    for i in 1..raw_field_count {
                        let raw_field = raw_fields[i];
                        field_list.push(Self::generate_subfields(raw_field, parser_chars));
                    }
                }
            }

            let field_description = RUMString::from(match V2_SEGMENT_DESC.get(&field_name) {
                Some(description) => description,
                None => V2_EMPTY_STRING,
            });

            Ok(V2Segment {
                name: field_name,
                description: field_description,
                fields: field_list,
            })
        }

        pub fn to_string(&self, parser_chars: &V2ParserCharacters) -> V2String {
            let mut segment: Vec<V2String> = Vec::with_capacity(self.fields.len());
            for field_group in self.fields.iter() {
                let mut fields: Vec<V2String> = Vec::with_capacity(field_group.len());
                for field in field_group {
                    fields.push(field.to_string(parser_chars));
                }
                segment.push(fields.join_compact(parser_chars.repetition_separator.as_str()));
            }
            format_compact!(
                "{}{}{}",
                self.name,
                parser_chars.field_separator.as_str(),
                segment.join_compact(parser_chars.field_separator.as_str())
            )
        }

        pub fn get(&self, indx: isize) -> V2Result<&V2FieldGroup> {
            let field_indx = clamp_index(&indx, &(self.fields.len() as isize))? - 1;
            match self.fields.get(field_indx) {
                Some(field) => Ok(field),
                None => Err(format_compact!("Field number {} not found!", indx)),
            }
        }

        pub fn get_mut(&mut self, indx: isize) -> V2Result<&mut V2FieldGroup> {
            let field_indx = clamp_index(&indx, &(self.fields.len() as isize))? - 1;
            match self.fields.get_mut(field_indx) {
                Some(field) => Ok(field),
                None => Err(format_compact!("Field number {} not found!", indx)),
            }
        }

        pub fn len(&self) -> usize {
            self.fields.len()
        }

        fn generate_subfields(field: &str, parser_chars: &V2ParserCharacters) -> Vec<V2Field> {
            let repetition_char = parser_chars.repetition_separator.as_str();
            let subfields: Vec<&str> = field.split(&repetition_char).collect();
            let mut field_group = V2FieldGroup::with_capacity(subfields.len());
            for subfield in subfields {
                field_group.push(V2Field::from_str(subfield, parser_chars))
            }
            field_group
        }
    }

    impl Index<isize> for V2Segment {
        type Output = V2FieldGroup;
        fn index(&self, indx: isize) -> &V2FieldGroup {
            self.get(indx).unwrap()
        }
    }

    impl IndexMut<isize> for V2Segment {
        fn index_mut(&mut self, indx: isize) -> &mut V2FieldGroup {
            self.get_mut(indx).unwrap()
        }
    }

    ///
    /// Segments can be repeating. As such we contain them in groups.
    ///
    /// ## Per Section 2.5.2
    /// ```text
    /// Two or more segments MAY be organized as a logical unit called a segment group. A segment group
    /// MAY be required or optional and might or might not repeat. As of v 2.5, the first segment in a newly
    /// defined segment group will be required to help ensure that unparsable messages will not be
    /// inadvertently defined. This required first segment is known as the anchor segment.
    /// ```
    ///
    pub type V2SegmentGroup = Vec<V2Segment>;

    ///
    /// We collect segment groups in a map thus yielding the core of a message.
    ///
    pub type SegmentMap = AHashMap<u8, V2SegmentGroup>;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub struct V2Message {
        separators: V2ParserCharacters,
        segment_groups: SegmentMap,
    }

    impl V2Message {
        pub fn from_str(raw_msg: &str) -> Self {
            Self::try_from_str(raw_msg).expect("If calls to from_str are failing for V2Message, consider using try_from_str or the TryFrom trait! You should not see this message.")
        }
        pub fn try_from_str(raw_msg: &str) -> V2Result<Self> {
            let clean_msg = V2Message::sanitize(raw_msg);
            let segment_tokens = V2Message::tokenize_segments(clean_msg.as_str());
            let msh_segment = V2Message::find_msh(&segment_tokens)?;
            let parse_characters = V2ParserCharacters::from_msh(msh_segment)?;
            let segments = V2Message::extract_segments(&segment_tokens, &parse_characters)?;

            Ok(V2Message {
                separators: parse_characters,
                segment_groups: segments,
            })
        }

        pub fn to_string(&self) -> V2String {
            let mut msg: Vec<V2String> = Vec::with_capacity(self.segment_groups.len());
            for segment_key in self.segment_groups.keys() {
                let segment_group = &self.segment_groups[segment_key];
                for segment in segment_group {
                    msg.push(segment.to_string(&self.separators));
                }
            }
            msg.join_compact(self.separators.segment_terminator.as_str())
        }

        pub fn len(&self) -> usize {
            self.segment_groups.len()
        }

        pub fn is_empty(&self) -> bool {
            self.segment_groups.is_empty()
        }

        pub fn get(&self, segment_index: &u8, sub_segment: usize) -> V2Result<&V2Segment> {
            let segment_group = self.get_group(segment_index)?;
            let subsegment_indx = sub_segment - 1;
            match segment_group.get(subsegment_indx) {
                Some(segment) => Ok(segment),
                None => Err(format_compact!(
                    "Subsegment {} was not found in segment group {}!",
                    subsegment_indx,
                    segment_index
                )),
            }
        }

        pub fn get_mut(
            &mut self,
            segment_index: &u8,
            sub_segment: usize,
        ) -> V2Result<&mut V2Segment> {
            let segment_group = self.get_mut_group(segment_index)?;
            let subsegment_indx = sub_segment - 1;
            match segment_group.get_mut(subsegment_indx) {
                Some(segment) => Ok(segment),
                None => Err(format_compact!(
                    "Subsegment {} was not found in segment group {}!",
                    subsegment_indx,
                    segment_index
                )),
            }
        }

        pub fn get_group(&self, segment_index: &u8) -> V2Result<&V2SegmentGroup> {
            match self.segment_groups.get(segment_index) {
                Some(segment_group) => Ok(segment_group),
                None => Err(format_compact!(
                    "Segment id {} not found in message!",
                    segment_index
                )),
            }
        }

        pub fn get_mut_group(&mut self, segment_index: &u8) -> V2Result<&mut V2SegmentGroup> {
            match self.segment_groups.get_mut(segment_index) {
                Some(segment_group) => Ok(segment_group),
                None => Err(format_compact!(
                    "Segment id {} not found in message!",
                    segment_index
                )),
            }
        }

        pub fn find_component(&self, search_pattern: &RUMString) -> V2Result<&V2Component> {
            let index = rumtk_cache_fetch!(&mut search_cache, search_pattern, compile_search_index);
            let segment = self.get(&index.segment, index.segment_group as usize)?;
            let field = match segment.get((index.field) as isize)?.get((index.field_group - 1) as usize) {
                Some(field) => field,
                None => return Err(format_compact!("Subfield provided is not 1 indexed or out of bounds. Did you give us a 0 when you meant 1? Got {}!", index.field_group))
            };
            field.get(index.component as isize)
        }

        pub fn is_repeat_segment(&self, segment_index: &u8) -> bool {
            let _segment_group: &V2SegmentGroup = self.get_group(segment_index).unwrap();
            _segment_group.len() > 1
        }

        pub fn segment_exists(&self, segment_index: &u8) -> bool {
            self.segment_groups.contains_key(segment_index)
        }

        // Message parsing operations
        pub fn find_msh<'a>(segments: &Vec<&'a str>) -> V2Result<&'a str> {
            for segment in segments {
                if segment.starts_with(V2_MSHEADER_PATTERN) {
                    return Ok(segment);
                }
            }
            Err("No MSH segment found! The message is malformed or incomplete!".to_rumstring())
        }

        pub fn sanitize(raw_message: &str) -> RUMString {
            let rr_string = raw_message.replace("\n", "\r");
            let mut n_string = rr_string.replace("\r\r", "\r");
            while n_string.contains("\r\r") {
                n_string = n_string.replace("\r\r", "\r");
            }
            n_string.to_rumstring()
        }

        pub fn tokenize_segments(raw_message: &str) -> Vec<&str> {
            //Per Figure 2-1. Delimiter values of the HL7 v2 2.9 standard, each segment is separated
            // by a carriage return <cr>. The value cannot be changed by implementers.
            let tokens: Vec<&str> = raw_message.split(V2_SEGMENT_TERMINATOR).collect();
            let mut trimmed_tokens: Vec<&str> = Vec::new();
            for tok in tokens {
                trimmed_tokens.push(tok.trim());
            }
            trimmed_tokens
        }

        pub fn extract_segments(
            raw_segments: &Vec<&str>,
            parser_chars: &V2ParserCharacters,
        ) -> V2Result<SegmentMap> {
            let mut segments: SegmentMap = SegmentMap::new();

            for segment_str in raw_segments {
                if segment_str.is_empty() {
                    continue;
                }

                let segment: V2Segment = V2Segment::from_str(segment_str, parser_chars)?;

                let key = match V2_SEGMENT_IDS.get(&segment.name) {
                    Some(k) => k,
                    None => return Err(format_compact!("Segment name is not a valid segment!")),
                };
                if !segments.contains_key(key) {
                    segments.insert(*key, V2SegmentGroup::new());
                }
                segments.get_mut(key).unwrap().push(segment);
            }

            Ok(segments)
        }
    }

    impl Index<&'_ u8> for V2Message {
        type Output = V2SegmentGroup;
        fn index(&self, segment_index: &u8) -> &V2SegmentGroup {
            self.get_group(segment_index).unwrap()
        }
    }

    impl IndexMut<&'_ u8> for V2Message {
        fn index_mut(&mut self, segment_index: &u8) -> &mut V2SegmentGroup {
            self.get_mut_group(segment_index).unwrap()
        }
    }

    impl TryFrom<&str> for V2Message {
        type Error = V2String;
        fn try_from(input: &str) -> V2Result<Self> {
            V2Message::try_from_str(input)
        }
    }

    impl TryFrom<&&str> for V2Message {
        type Error = V2String;
        fn try_from(input: &&str) -> V2Result<Self> {
            V2Message::try_from_str(input)
        }
    }

    impl TryFrom<&String> for V2Message {
        type Error = V2String;
        fn try_from(input: &String) -> V2Result<Self> {
            V2Message::try_from_str(input.as_str())
        }
    }

    impl TryFrom<&RUMString> for V2Message {
        type Error = V2String;
        fn try_from(input: &RUMString) -> V2Result<Self> {
            V2Message::try_from_str(input.as_str())
        }
    }

    impl TryFrom<&[u8]> for V2Message {
        type Error = V2String;
        fn try_from(input: &[u8]) -> V2Result<Self> {
            V2Message::try_from_str(try_decode_with(input, "ascii").as_str())
        }
    }
}

pub mod v2_parser_interface {
    /**************************** Macros ***************************************/
    use crate::hl7_v2_parser;

    ///
    /// Simple interface for creating an instance of V2Message!
    /// You can pass a string view, a String, a RUMString, or a byte slice as input.
    ///
    /// ## Example
    ///
    /// ```
    ///     use rumtk_hl7_v2::{rumtk_v2_parse_message};
    ///     let pattern = "MSH1.1";
    ///     let hl7_v2_message = "MSH|^~\\&|NISTEHRAPP|NISTEHRFAC|NISTIISAPP|NISTIISFAC|20150625072816.601-0500||VXU^V04^VXU_V04|NIST-IZ-AD-10.1_Send_V04_Z22|P|2.5.1|||ER|AL|||||Z22^CDCPHINVS|NISTEHRFAC|NISTIISFAC\n";
    ///     let message = rumtk_v2_parse_message!(&hl7_v2_message).unwrap();
    /// ```
    ///
    #[macro_export]
    macro_rules! rumtk_v2_parse_message {
        ( $msg:expr ) => {{
            use $crate::hl7_v2_parser::v2_parser::{V2Message, V2Result};
            V2Message::try_from($msg)
        }};
    }

    ///
    /// Simple interface for searching for a component inside a V2Message.
    /// This macro takes a borrow of a V2Message instance and a string search pattern.
    /// The only search pattern supported at the moment takes the form
    /// **<3-letter segment>(optional, segment_group)<field>\[optional, field_group\].<component>**.
    /// For example, you can search with **PID5.1** or **PID(1)5.1** or **PID(1)5\[1\].1**.
    ///
    /// The optional portions are for when you need to select a specific repeated segment or field.
    ///
    /// All of these indices must be 1-indexed.
    ///
    /// For the main indices, you can use negative values. For example, a -1 means you want to select
    /// the last item. This is applicable for the field and component indices.
    ///
    /// ## Example
    ///
    /// ```
    ///     use rumtk_hl7_v2::{rumtk_v2_parse_message, rumtk_v2_find_component};
    ///     let pattern = "MSH1.1";
    ///     let hl7_v2_message = "MSH|^~\\&|NISTEHRAPP|NISTEHRFAC|NISTIISAPP|NISTIISFAC|20150625072816.601-0500||VXU^V04^VXU_V04|NIST-IZ-AD-10.1_Send_V04_Z22|P|2.5.1|||ER|AL|||||Z22^CDCPHINVS|NISTEHRFAC|NISTIISFAC\n";
    ///     let message = rumtk_v2_parse_message!(&hl7_v2_message).unwrap();
    ///     let component = rumtk_v2_find_component!(message, pattern).unwrap();
    /// ```
    ///
    #[macro_export]
    macro_rules! rumtk_v2_find_component {
        ( $v2_msg:expr, $v2_search_pattern:expr ) => {{
            use rumtk_core::strings::RUMString;
            use $crate::hl7_v2_parser::v2_parser::{V2Component, V2Result};
            $v2_msg.find_component(&RUMString::from($v2_search_pattern))
        }};
    }

    ///
    /// Macro for generating V2 string message out of an instance of [hl7_v2_parser::v2_parser::V2Message].
    /// Basically, this is the opposite operation to [crate::rumtk_v2_parse_message].
    ///
    /// # Example
    /// ```
    ///     use rumtk_hl7_v2::{rumtk_v2_generate_message, rumtk_v2_parse_message};
    ///     let pattern = "MSH1.1";
    ///     let hl7_v2_message = "MSH|^~\\&|NISTEHRAPP|NISTEHRFAC|NISTIISAPP|NISTIISFAC|20150625072816.601-0500||VXU^V04^VXU_V04|NIST-IZ-AD-10.1_Send_V04_Z22|P|2.5.1|||ER|AL|||||Z22^CDCPHINVS|NISTEHRFAC|NISTIISFAC\n";
    ///     let message = rumtk_v2_parse_message!(&hl7_v2_message).unwrap();
    ///     let generated_message_str = rumtk_v2_generate_message!(&message);
    ///     let generated_message = rumtk_v2_parse_message!(&generated_message_str).unwrap();
    ///     assert_eq!(
    ///             &message, &generated_message,
    ///             "Messages are not equal! Expected: {:?} Got: {:?}",
    ///             &message, &generated_message
    ///         );
    /// ```
    ///
    #[macro_export]
    macro_rules! rumtk_v2_generate_message {
        ( $v2_msg:expr ) => {{
            $v2_msg.to_string()
        }};
    }
}
