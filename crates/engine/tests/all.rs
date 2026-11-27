//! Single test file of the integration tests (#43). Every file in `crates/engine/tests/` is a module here.
//!
//! Reason: otherwise Cargo builds a program of its own per file. In a mutation run 25 programs were thus
//! recompiled, linked and started to list the tests per mutant; linking and the first start (macOS checks every
//! new program) cost a multiple of the actual tests. With one program the tests stay the same, only
//! they are called `<file>::<test>` (example: `cargo nextest run -p vivaplan-engine -E 'test(/^golden::/)'`).
//!
//! A new test file must be registered here. The test at the end fails if a file is missing.

// #116: Memory limit per test process, so a mutant with an endless loop does not paralyse the machine.
#[global_allocator]
static TEST_ALLOC: engine::test_alloc::Limited = engine::test_alloc::Limited;

mod bench;
mod buckets_guardrails;
mod buckets_need;
mod cancel;
mod care_home;
mod cash_housing;
mod command_sweep;
mod compare_strategies;
mod data;
mod describe;
mod digest;
mod example_neu;
mod experiment;
mod files;
mod gain_location;
mod golden;
mod household;
mod income_deductions;
mod income_ends;
mod income_indexed;
mod indexation;
mod inflow_strategy;
mod invariants;
mod investment_yield;
mod jst;
mod large_pillar3a;
mod legacy_format;
mod legacy_import;
mod library;
mod mandatory_inputs;
mod measure;
mod milestones_limits;
mod never_sell;
mod no_pension_fund;
mod ods_reference;
mod optimization_settings;
mod optimize_goal;
mod optimize_ranges;
mod parametric_fit;
mod path_reuse;
mod pillar3a;
mod pillar3a_plan;
mod pillar3a_tranches;
mod plan_builder;
mod property_gain_models;
mod property_tax_value;
mod proptest_parsers;
mod proptest_plans;
mod proptest_tax;
mod proptest_upgrade;
mod qa_matrix;
mod reference;
mod relocation;
mod rerun_paths;
mod reserve_standard_only;
mod robustness;
mod series_forms;
mod sim_rules;
mod simulation;
mod state_curves;
mod statistics;
mod strategy;
mod tax_bill;
mod tax_capital_location;
mod tax_gain_cantons;
mod tax_indexed;
mod tax_location;
mod tax_parity;
mod tax_simple;
mod taxable_income;
mod upgrade_api;
mod validate_extra;
mod wealth_deductions;

/// Test plans that do not check #51 partly use ages outside the limits for the milestones
/// «AHV-Bezug» and «Bezug Pensionskasse» (for example a payout at 75 years). Open limits apply to them; the limits themselves are checked by the
/// tests in `validate_extra.rs` and `settings.rs`.
pub fn any_age(mut v: serde_json::Value) -> serde_json::Value {
    v["limits"] = serde_json::json!({ "ahvMin": 0, "ahvMax": 130, "pkMin": 0, "pkMax": 130 });
    v
}

/// #43: No test file may drop out of the run unnoticed (`autotests = false` in `crates/engine/Cargo.toml`).
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
