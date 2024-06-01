use std::collections::HashMap;


type ElementType = HashMap<String, String>;
const V2_SEGMENT_TYPES = ElementType::from([
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
    ("ZXT", "Non-Standard"),
    ("Z*", "Non-Standard")
]);

const V2_MESSAGE_TYPES = ElementType::from([
    "ACK",

]);

