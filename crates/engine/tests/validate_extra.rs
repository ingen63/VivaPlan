//! Additional checks of the validation: person 2, Monte Carlo limits, warnings about strategy and variations.

use engine::api;
use engine::validate::{Issue, Severity};
use serde_json::{json, Value};

fn base() -> Value {
    json!({
        "project": { "name": "Test" },
        "household": { "persons": [ { "id": "A", "name": "Anna", "birth": "1965-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2030-01" },
            { "id": "retire", "name": "Pensionierung", "kind": "date", "date": "2040-01" },
            { "id": "ahv", "name": "AHV", "kind": "date", "date": "2040-01" },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2031-12" }
        ],
        "pension": {
            "pk": { "capital": 100_000, "at": "retire", "lumpsum": 0.5, "conversion": 0.05 },
            "ahv": { "monthly": 2_000, "at": "ahv" }
        },
        "flows": { "income": 5_000, "spending": 4_000 },
        "wealth": { "stocks": 1_000, "bonds": 1_000, "cash": 1_000, "stockShare": 0.5, "reserveMonths": 3 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0.04, "bonds": 0.01, "inflation": 0.01 } },
        "taxes": { "income": [], "wealth": [], "capital": [] }
    })
}

fn has(v: &Value, severity: Severity, section: &str, text: &str) -> bool {
    let all: Vec<Issue> = api::validate(v);
    all.iter()
        .any(|i| i.severity == severity && i.section == section && i.message.contains(text))
}

fn with_partner(lumpsum: f64, conversion: f64) -> Value {
    let mut v = base();
    v["household"]["persons"] = json!([
        { "id": "A", "name": "Anna", "birth": "1965-01" },
        { "id": "B", "name": "Ben", "birth": "1965-01" }
    ]);
    v["partner"] = json!({ "income": 0, "pk": { "capital": 0, "at": "retire", "lumpsum": lumpsum, "conversion": conversion }, "ahv": { "monthly": 0, "at": "ahv" } });
    v
}

#[test]
fn partner_pension_fund_inputs_are_checked() {
    assert!(has(
        &with_partner(1.5, 0.05),
        Severity::Error,
        "pension",
        "Person 2: the lump sum"
    ));
    assert!(has(
        &with_partner(0.5, 0.2),
        Severity::Warning,
        "pension",
        "Person 2: conversion rate"
    ));
    assert!(!has(&with_partner(0.5, 0.05), Severity::Error, "pension", "Person 2"));
}

#[test]
fn second_person_without_data_is_only_a_warning() {
    let mut v = base();
    v["household"]["persons"] = json!([
        { "id": "A", "name": "Anna", "birth": "1965-01" },
        { "id": "B", "name": "Ben", "birth": "1965-01" }
    ]);
    assert!(has(
        &v,
        Severity::Warning,
        "general",
        "second person has neither income"
    ));
}

#[test]
fn monte_carlo_paths_and_series_are_checked() {
    let mut v = base();
    v["returns"]["source"] = json!("bootstrap");
    v["returns"]["bootstrap"] = json!({ "paths": 10, "block": 12, "seed": 1 });
    assert!(has(&v, Severity::Error, "returns", "requires a time series"));
    v["returns"]["bootstrap"]["paths"] = json!(1_000_001);
    assert!(has(&v, Severity::Error, "returns", "between 1 and 1'000'000"));
    v["returns"]["source"] = json!("parametric");
    v["returns"]["parametric"] = json!({ "paths": 1_000_001, "mu": 0.07, "sigma": 0.15, "df": 0, "jumpLambda": 0, "jumpMu": 0, "jumpSigma": 0, "seed": 1 });
    assert!(has(&v, Severity::Error, "returns", "between 1 and 1'000'000"));
    v["returns"]["parametric"]["paths"] = json!(1_000_000);
    assert!(!has(&v, Severity::Error, "returns", "between 1 and 1'000'000"));
}

#[test]
fn three_buckets_covering_more_than_fifteen_years_warn() {
    let mut v = base();
    v["withdrawal"] = json!({ "strategy": "three_buckets", "buckets": { "cashYears": 8, "bondYears": 8 } });
    assert!(has(&v, Severity::Warning, "withdrawal", "more than 15 years"));
}

#[test]
fn sweeps_that_have_no_effect_are_reported() {
    let mut v = base();
    v["withdrawal"] = json!({ "strategy": "three_buckets", "buckets": { "cashYears": 2, "bondYears": 3 } });
    v["scenarios"] = json!({ "sweeps": [ { "param": "wealth.stockShare", "from": 0.0, "to": 0.5, "step": 0.5 } ] });
    assert!(has(
        &v,
        Severity::Warning,
        "scenarios",
        "3-bucket strategy, the stock allocation only applies"
    ));
}

#[test]
fn too_many_combinations_are_an_error_in_the_plan() {
    let mut v = base();
    let sweep = |param: &str, from: f64, to: f64| json!({ "param": param, "from": from, "to": to, "step": 0.01 });
    v["scenarios"] = json!({ "sweeps": [
        sweep("wealth.stockShare", 0.0, 1.0),
        sweep("pension.pk.lumpsum", 0.0, 0.99),
        sweep("flows.spendingFactor", 1.0, 1.99)
    ] });
    assert!(has(&v, Severity::Error, "scenarios", "1'010'000 combinations"));
}

// #134/AK4: The validation reports a rate table in which the tax (amount × rate) falls with a rising amount as a
// warning referring to the row; rising average rates and the example plan stay without a message.
#[test]
fn issue_134_ak4_falling_tax_table_is_a_warning_at_the_row() {
    let issues = |v: &Value| -> Vec<Issue> { api::validate(v) };
    let falling = |i: &Issue| i.message.contains("the tax falls");

    let mut v = base();
    v["taxes"]["income"] = json!([[0, 0.0], [100_000, 0.10], [200_000, 0.15]]);
    assert!(!issues(&v).iter().any(falling), "steigende Sätze");
    assert!(!issues(&api::example_plan()).iter().any(falling), "Beispielplanung");

    // [[0, 10 %], [300'000, 1 %]]: 200'000 → 8'000, 300'000 → 3'000
    for (key, path) in [("wealth", "taxes.wealth.1.1"), ("capital", "taxes.capital.1.1")] {
        let mut v = base();
        v["taxes"][key] = json!([[0, 0.10], [300_000, 0.01]]);
        let found: Vec<Issue> = issues(&v).into_iter().filter(falling).collect();
        assert_eq!(found.len(), 1, "{key}: {found:?}");
        assert_eq!(found[0].severity, Severity::Warning, "Warnung, kein Fehler");
        assert_eq!((found[0].section, found[0].field.as_deref()), ("taxes", Some(path)));
    }
    // Tables for married couples, row in the entered order
    let mut v = base();
    v["taxes"]["couple"] = json!({ "income": [[300_000, 0.01], [0, 0.10]], "wealth": [], "capital": [] });
    let found: Vec<Issue> = issues(&v).into_iter().filter(falling).collect();
    assert_eq!(found[0].field.as_deref(), Some("taxes.couple.income.0.1"));
    assert!(found[0].message.starts_with("Income tax (married), bracket 1"));
    // Invalid rates are already an error: no additional warning
    let mut v = base();
    v["taxes"]["income"] = json!([[0, 0.10], [300_000, -0.5]]);
    assert!(!issues(&v).iter().any(falling));
    assert!(has(&v, Severity::Error, "taxes", "tax rate must be between"));
}

/// #139: Monte Carlo plan over `years` years from January 2030 with `paths` paths.
fn mc(source: &str, paths: u64, years: i32) -> Value {
    let mut v = base();
    v["milestones"][3]["date"] = json!(format!("{}-12", 2029 + years));
    v["returns"]["source"] = json!(source);
    v["returns"]["strategy"] = json!({ "stocks": "sp500", "bonds": "usa" });
    v["returns"][source] = if source == "bootstrap" {
        json!({ "paths": paths, "block": 12, "seed": 1 })
    } else {
        json!({ "paths": paths, "mu": 0.07, "sigma": 0.15, "df": 6, "seed": 1 })
    };
    v
}

fn too_many(v: &Value) -> Option<Issue> {
    api::validate(v)
        .into_iter()
        .find(|i| i.message.starts_with("Too many paths"))
}

// #139/AK1, AK2: 1'000'000 paths over 100 years are an error at the field of the paths, with the highest possible number
// (50'000'000 / 1'200 = 41'666).
#[test]
fn issue_139_ak2_one_million_paths_over_100_years_are_an_error() {
    for source in ["bootstrap", "parametric"] {
        let i = too_many(&mc(source, 1_000_000, 100)).expect(source);
        assert_eq!(i.severity, Severity::Error);
        assert_eq!(i.field.as_deref(), Some(format!("returns.{source}.paths").as_str()));
        assert_eq!(
            i.message,
            "Too many paths for the memory: 1'000'000 paths × 1200 months. With this planning period, at most 41'666 paths are possible."
        );
    }
}

// #139/AK3: 50'000 paths over 30 years (a usual computation today) stay allowed; exactly at the limit too, one path
// more not. Fixed and historical returns are not affected.
#[test]
fn issue_139_ak3_usual_plans_stay_allowed_and_the_limit_is_exact() {
    assert!(too_many(&mc("bootstrap", 50_000, 30)).is_none());
    assert!(too_many(&mc("parametric", 50_000, 30)).is_none());
    // 100 years = 1'200 months: 41'666 × 1'200 = 49'999'200 ≤ 50 million, 41'667 × 1'200 = 50'000'400
    assert!(too_many(&mc("bootstrap", 41_666, 100)).is_none());
    assert!(too_many(&mc("bootstrap", 41_667, 100)).is_some());
    let mut fixed = mc("bootstrap", 1_000_000, 100);
    fixed["returns"]["source"] = json!("fixed");
    assert!(too_many(&fixed).is_none());
    assert_eq!(engine::validate::MAX_PATH_MONTHS, 50_000_000);
}

// #162/AK1: Exactly 50 million path-months are allowed (40 months × 1'250'000 paths), one path more not. The number of paths
// in the message has thousands separators, also with a number of digits divisible by 3 (100'000, no leading apostrophe).
#[test]
fn issue_162_ak1_exact_path_limit_and_thousands_separator() {
    let forty = |paths| {
        let mut v = mc("bootstrap", paths, 4);
        v["milestones"][3]["date"] = json!("2033-04");
        v
    };
    assert!(too_many(&forty(1_250_000)).is_none(), "genau 50 Mio. Pfad-Monate");
    assert!(too_many(&forty(1_250_001)).is_some());
    let i = too_many(&mc("parametric", 100_000, 100)).unwrap();
    assert!(
        i.message
            .starts_with("Too many paths for the memory: 100'000 paths × 1200 months."),
        "{}",
        i.message
    );
}
