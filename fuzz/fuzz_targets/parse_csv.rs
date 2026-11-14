#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|text: &str| {
    let _ = engine::market::DataSet::parse_csv("fuzz", text);
});
