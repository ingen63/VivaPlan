//! Einzige Testdatei der Integrationstests (#43). Jede Datei in `crates/engine/tests/` ist hier ein Modul.
//!
//! Grund: Cargo baut sonst je Datei ein eigenes Programm. Bei einem Mutationslauf wurden so pro Mutant 25 Programme
//! neu übersetzt, gebunden und zum Auflisten der Tests gestartet; das Binden und der erste Start (macOS prüft jedes
//! neue Programm) kosteten ein Vielfaches der eigentlichen Tests. Mit einem Programm bleiben dieselben Tests, nur
//! heissen sie `<datei>::<test>` (Beispiel: `cargo nextest run -p vivaplan-engine -E 'test(/^golden::/)'`).
//!
//! Eine neue Testdatei muss hier eingetragen werden. Der Test am Ende schlägt fehl, wenn eine Datei fehlt.

// #116: Speichergrenze je Testprozess, damit ein Mutant mit endloser Schleife den Rechner nicht lahmlegt.
#[global_allocator]
static TEST_ALLOC: engine::test_alloc::Limited = engine::test_alloc::Limited;

mod bench;
mod buckets_guardrails;
mod buckets_need;
mod cancel;
mod cash_housing;
mod data;
mod example_neu;
mod files;
mod golden;
mod household;
mod income_indexed;
mod inflow_strategy;
mod invariants;
mod jst;
mod legacy_format;
mod legacy_import;
mod library;
mod mandatory_inputs;
mod milestones_limits;
mod ods_reference;
mod optimization_settings;
mod optimize_goal;
mod optimize_ranges;
mod parametric_fit;
mod pillar3a;
mod pillar3a_plan;
mod proptest_parsers;
mod proptest_plans;
mod proptest_tax;
mod proptest_upgrade;
mod qa_matrix;
mod reference;
mod rerun_paths;
mod reserve_standard_only;
mod robustness;
mod sim_rules;
mod simulation;
mod strategy;
mod upgrade_api;
mod validate_extra;

/// Testplanungen, die nicht #51 prüfen, verwenden teils Alter ausserhalb der Grenzen für «AHV-Bezug» und «Bezug
/// Pensionskasse» (zum Beispiel ein Bezug mit 75 Jahren). Für sie gelten offene Grenzen; die Grenzen selbst prüfen die
/// Tests in `validate_extra.rs` und `settings.rs`.
pub fn any_age(mut v: serde_json::Value) -> serde_json::Value {
    v["limits"] = serde_json::json!({ "ahvMin": 0, "ahvMax": 130, "pkMin": 0, "pkMax": 130 });
    v
}

/// #43: Keine Testdatei darf unbemerkt aus dem Lauf fallen (`autotests = false` in `crates/engine/Cargo.toml`).
#[test]
fn every_test_file_is_a_module_of_this_binary() {
    use std::collections::BTreeSet;
    let declared: BTreeSet<String> = include_str!("all.rs")
        .lines()
        .filter_map(|l| l.strip_prefix("mod ")?.strip_suffix(';'))
        .map(str::to_owned)
        .collect();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let files: BTreeSet<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .map(|p| p.file_stem().unwrap().to_string_lossy().into_owned())
        .filter(|name| name != "all")
        .collect();
    assert_eq!(
        declared, files,
        "crates/engine/tests/all.rs und die Dateien in crates/engine/tests/ stimmen nicht überein"
    );
}
