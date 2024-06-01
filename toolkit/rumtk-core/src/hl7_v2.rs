//https://v2.hl7.org/conformance/HL7v2_Conformance_Methodology_R1_O1_Ballot_Revised_D9_-_September_2019_Introduction.html#:~:text=The%20base%20HL7%20v2%20standard,message%20definition%20is%20called%20profiling.

mod HL7V2_STRUCTS {
    use std::collections::hash_map::{HashMap};
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

    type SegmentMap = HashMap<String, V2Segment>;
    type SegmentDefinition = HashMap<String, String>;
    const V2SEGMENTKEYS = SegmentDefinition::from([
        ("MSH", "Message Header"),
        ("EVN", "Event"),
        ("PID", "Patient"),
        ("PD1", "Patient Demographics Extended"),
        ("PV1", "Visit/Encounter"),
        ("PV2", "Visit/Encounter Additional"),
        ("ROL", "Role"),
        ("DG1", "Diagnosis"),
        ("PR1", "Procedure"),
        ("MRG", "Merge Patient Information"),
        ("GT1", "Guarantor"),
        ("IN1", "Insurance"),
        ("IN2", "Insurance Additional Information"),
        ("ORC", "Order Control"),
        ("OBR", "Observation Request"),
        ("OBX", "Observation"),
        ("NK1", "Next of Kin / Patient Contact"),
        ("NTE", "Note"),
        ("FT1", "Financial Transaction"),
        ("RXA", "Pharmacy Administration"),
        ("RXC", "Pharmacy Component"),
        ("ZXC", "Pharmacy Component"),
        ("RXE", "Pharmacy Encoded Order"),
        ("RXR", "Pharmacy Route"),
        ("AL1", "Allergy Information"),
        ("IAM", "Patient Adverse Reaction"),
        ("SPM", "Specimen"),
        ("SCH", "Scheduling"),
        ("RGS", "Resource Group Segment"),
        ("AIL", "Location Resource"),
        ("AIP", "Personnel Resource"),
        ("ZXT", "Non-Standard")
    ]);

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

        fn from_raw_message(self, raw_msg: &String) -> V2Message {
            let mut message: V2Message = self.new();

            message
        }
    }
}