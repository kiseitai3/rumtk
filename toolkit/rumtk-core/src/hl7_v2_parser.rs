//https://v2.hl7.org/conformance/HL7v2_Conformance_Methodology_R1_O1_Ballot_Revised_D9_-_September_2019_Introduction.html#:~:text=The%20base%20HL7%20v2%20standard,message%20definition%20is%20called%20profiling.
//https://www.hl7.org/implement/standards/product_brief.cfm?product_id=185
mod v2_parser {
    use std::collections::hash_map::{HashMap};

    type FieldList = Vec<String>;
    struct V2Field {
        components: FieldList
    }

    impl V2Field {
        fn new(self, val: String) -> V2Field {
            V2Field{components: FieldList::new()}
        }

        fn len(self) -> usize {
            self.components.len()
        }
    }

    struct V2Segment {
        fields: Vec<V2Field>
    }

    impl V2Segment {
        fn new(s: usize) -> V2Segment {
            V2Segment{fields: Vec::with_capacity(s)}
        }

        fn len(self) -> usize {
            self.fields.len()
        }
    }

    type SegmentList = Vec<V2Segment>;
    type SegmentMap = HashMap<String, SegmentList>;

    struct V2Message {
        segments: SegmentMap
    }

    impl V2Message {
        fn new(self) -> V2Message {
            V2Message{segments: SegmentMap::new()}
        }

        fn len(self) -> usize {
            self.segments.len()
        }

        fn is_repeat_segment(self, segment_name: &String) -> bool {
            let _segment_group: &SegmentList = self.find_segment(segment_name);
            _segment_group.len() > 1
        }

        fn segment_exists(self, segment_name: &String) -> bool {
            let _segment_group: &SegmentList = self.find_segment(segment_name);
            _segment_group.len() > 0
        }

        fn find_segment(self, segment_name: &String) -> &SegmentList {
            match self.segments.get(segment_name) {
                Ok(segment_list) => &segment_list,
                None() => &SegmentList::new()
            }
        }
    }
}