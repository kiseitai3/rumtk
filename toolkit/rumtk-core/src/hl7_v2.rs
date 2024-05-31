//https://v2.hl7.org/conformance/HL7v2_Conformance_Methodology_R1_O1_Ballot_Revised_D9_-_September_2019_Introduction.html#:~:text=The%20base%20HL7%20v2%20standard,message%20definition%20is%20called%20profiling.

mod HL7V2_STRUCTS {
    struct V2Item {
        raw: String
    }

    impl V2Item {
        fn new(self, val: String) -> V2Item {
            V2Item{raw: val}
        }

        fn len(self) -> usize {
            self.raw.len()
        }
    }

    struct V2Segment {
        nodes: Vec<V2Item>
    }

    impl V2Segment {
        fn new(s: usize) -> V2Segment {
            V2Segment{nodes: Vec::with_capacity(s)}
        }

        fn len(self) -> usize {
            self.nodes.len()
        }
    }

    struct V2Message {
        segments: Vec<V2Segment>
    }

    impl V2Message {
        fn new(self, raw: &String) -> V2Message {
            V2Message{segments: self.find_segments(&raw)}
        }

        fn len(self) -> usize {
            self.segments.len()
        }

        fn find_segments(self, raw_msg: &String) -> Vec<V2Segment> {
            Vec<V2Segment>::new()
        }
    }
}