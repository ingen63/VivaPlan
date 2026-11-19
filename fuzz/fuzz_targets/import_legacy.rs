//! #137: fuzz target for the import of old `config.json` files. Any text goes through `api::import_legacy`; a crash is
//! a finding. Data paths of the old configuration start in an empty folder; files are only read. Seed corpus:
//! `fuzz/seeds/import_legacy/`.
#![no_main]
use std::path::Path;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        let base = std::env::temp_dir().join("vivaplan-fuzz-import-legacy-empty");
        let _ = engine::api::import_legacy(text, 2030, Path::new(&base), "fuzz");
    }
});
