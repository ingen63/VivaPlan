//! #137: fuzz target for upgrading older plans (runs on every open). Any JSON goes through
//! `api::upgrade_plan` and then through `api::validate`; a crash is a finding. Seed corpus: `fuzz/seeds/upgrade_plan/`.
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(plan) = serde_json::from_slice::<serde_json::Value>(data) {
        let out = engine::api::upgrade_plan(&plan);
        let _ = engine::api::validate(&out.plan);
    }
});
