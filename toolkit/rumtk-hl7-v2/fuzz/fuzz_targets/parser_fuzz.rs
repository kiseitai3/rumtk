#![no_main]

use libfuzzer_sys::fuzz_target;
use rumtk_hl7_v2::rumtk_v2_parse_message;

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    let message = rumtk_v2_parse_message!(data);
    match message {
        Ok(msg) => {
            println!("{:?}", &msg);
        }
        Err(e) => {
            println!("{}", e);
        }
    }
});
