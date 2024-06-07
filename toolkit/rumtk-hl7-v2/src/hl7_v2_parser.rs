//https://v2.hl7.org/conformance/HL7v2_Conformance_Methodology_R1_O1_Ballot_Revised_D9_-_September_2019_Introduction.html#:~:text=The%20base%20HL7%20v2%20standard,message%20definition%20is%20called%20profiling.
//https://www.hl7.org/implement/standards/product_brief.cfm?product_id=185

mod v2_parser {
    use std::collections::hash_map::{HashMap};
    use crate::hl7_v2_types::v2_types::{V2String, V2DateTime};

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
            let comp_vec: Vec<&String> = val.split(divider).collect();
            let mut component_list: FieldList = FieldList::new();
            for c in comp_vec {
                component_list.push(V2Component::from(c));
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

    struct V2Message {
        default_segment: V2SegmentGroup,
        segment_groups: SegmentMap
    }

    impl V2Message {
        fn new() -> V2Message {
            V2Message{default_segment: V2SegmentGroup::new(), segment_groups: SegmentMap::new()}
        }

        fn len(self) -> usize {
            self.segment_groups.len()
        }

        fn is_repeat_segment(self, segment_name: &String) -> bool {
            let _segment_group: &V2SegmentGroup = self.find_segment(segment_name);
            _segment_group.len() > 1
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
    }

    struct V2Parser {

    }
}