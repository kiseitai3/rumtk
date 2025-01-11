#![no_main]

use libfuzzer_sys::fuzz_target;
use rumtk_hl7_v2::v2_parse_message;

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    let message = v2_parse_message!(data);
    match message {
        Ok(msg) => {
            println!("{:?}", &msg);
        },
        Err(e) => { println!("{}", e); }
    }
});
