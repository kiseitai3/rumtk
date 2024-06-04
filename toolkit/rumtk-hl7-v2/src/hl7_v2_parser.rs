//https://v2.hl7.org/conformance/HL7v2_Conformance_Methodology_R1_O1_Ballot_Revised_D9_-_September_2019_Introduction.html#:~:text=The%20base%20HL7%20v2%20standard,message%20definition%20is%20called%20profiling.
//https://www.hl7.org/implement/standards/product_brief.cfm?product_id=185

mod v2_parser {
    use std::collections::hash_map::{HashMap};

    const V2_DELETE_FIELD: &str = "\"\"";
    struct V2Component {
        component: String,
        delete_data: bool
    }

    impl V2Component {
        fn new(self) -> V2Component {
            V2Component{component: String::new(), delete_data: false}
        }

        fn from(self, item: &String) -> V2Component {
            V2Component{component: String::from(item), delete_data: item == V2_DELETE_FIELD}
        }

        fn is_empty(self) -> bool {
            self.component == ""
        }

        fn as_datetime(self) -> DateTime<Utc> {
            let dt = self.component.split('+)
            let date_time: NaiveDateTime = NaiveDateTime::from_ymd(2017, 11, 12).and_hms(17, 33, 44)
            self.component.parse().unwrap()
        }

        fn as_bool(self) -> bool {

        }

        fn as_integer(self) -> i64 {

        }

        fn as_float(self) -> f64 {

        }
    }

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

        fn find_component(self, component_name: &String) ->
    }

    type V2Segment = Vec<V2Field>;
    type V2SegmentGroup = Vec<V2Segment>;
    type SegmentMap = HashMap<String, V2SegmentGroup>;

    struct V2Message {
        default_segment: V2SegmentGroup,
        segment_groups: SegmentMap
    }

    impl V2Message {
        fn new(self) -> V2Message {
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