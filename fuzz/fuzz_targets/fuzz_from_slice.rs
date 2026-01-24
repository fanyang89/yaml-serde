#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() <= 10240 {
        _ = yaml_serde::from_slice::<yaml_serde::Value>(data);
    }
});
