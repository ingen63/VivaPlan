//! #137: Fuzz-Ziel für das Nachführen älterer Planungen (läuft bei jedem Öffnen). Beliebiges JSON geht durch
//! `api::upgrade_plan` und danach durch `api::validate`; ein Absturz ist ein Fund. Startkorpus: `fuzz/seeds/upgrade_plan/`.
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(plan) = serde_json::from_slice::<serde_json::Value>(data) {
        let out = engine::api::upgrade_plan(&plan);
        let _ = engine::api::validate(&out.plan);
    }
});
