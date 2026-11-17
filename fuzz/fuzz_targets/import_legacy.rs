//! #137: Fuzz-Ziel für den Import alter `config.json`. Beliebiger Text geht durch `api::import_legacy`; ein Absturz ist
//! ein Fund. Datenpfade der alten Konfiguration gelten ab einem leeren Ordner, gelesen wird nur. Startkorpus:
//! `fuzz/seeds/import_legacy/`.
#![no_main]
use std::path::Path;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        let base = std::env::temp_dir().join("vivaplan-fuzz-import-legacy-leer");
        let _ = engine::api::import_legacy(text, 2030, Path::new(&base), "fuzz");
    }
});
